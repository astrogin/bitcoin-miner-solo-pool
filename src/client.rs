use std::collections::HashMap;
use std::fs::read;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, Error, BufReader, BufStream, BufWriter};
use tokio::net::TcpStream;
use std::{io, thread};
use std::borrow::BorrowMut;
use std::io::ErrorKind;
use std::ops::DerefMut;
use std::str::from_utf8;
use std::sync::{Arc, Mutex};
use clap::builder::Str;
use serde_json::Value;
use serde::{Deserialize, Serialize};
use tokio::net::tcp::{ReadHalf, WriteHalf};
use tokio::sync::{mpsc};
use tokio::sync::mpsc::error::TryRecvError;
use tokio::sync::mpsc::Sender;
use tokio::sync::mpsc::Receiver;
use crate::message::{Message, Notification, Response};
use crate::miner::Miner;

#[derive(Serialize, Deserialize)]
struct Request {
    id: String,
    method: String,
    params: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Client {
    client_sender: Arc<Sender<String>>,
    miner_sender: Arc<Sender<String>>,
    wallet: String,
    extra_nonce1: Arc<Mutex<String>>,
}

impl Client {
    pub async fn connect(address: String, wallet: String) -> Result<(), Error> {
        println!("{:?}", address);
        let (mut reader, mut writer) = TcpStream::connect(address.clone()).await.unwrap().into_split();
        let mut buf_reader = BufReader::new(reader);
        let (pool_sender, mut pool_receiver) = mpsc::channel(32);
        let (mut miner_sender, mut miner_receiver) = mpsc::channel(32);
        let mut miner_sender = Arc::new(miner_sender);
        let (cs, mut client_receiver) = mpsc::channel(32);
        let client_sender = Arc::new(cs);
        let client_sender2 = Arc::clone(&client_sender);
        let extra_nonce_1 = Arc::new(Mutex::new(String::new()));
        let mut client = Client {
            client_sender: Arc::clone(&client_sender),
            wallet,
            extra_nonce1: extra_nonce_1.clone(),
            miner_sender,
        };
        let subscribe_request = Request {
            id: String::from("mining.subscribe"),
            method: String::from("mining.subscribe"),
            params: vec![],
        };

        let subscribe_message = format!("{}\n", serde_json::to_string(&subscribe_request).unwrap());
        client_sender.send(subscribe_message).await.expect("TODO: panic message");

        tokio::spawn(async move {
            loop {
                let message = client_receiver.recv().await.unwrap();
                println!("Message sent {:?}", message);
                writer.write_all(message.as_bytes()).await.unwrap();
            }
        });

        tokio::spawn(async move {
            loop {
                let message = pool_receiver.recv().await.unwrap();
                client.handle_pool_message(message).await;
            }
        });

        thread::spawn(move || {
            let mut miner: Option<Miner> = None;
            loop {
                match miner_receiver.try_recv() {
                    Ok(msg) => {
                        let nonce1: &str = &*extra_nonce_1.lock().unwrap();
                        let message: Notification = serde_json::from_str(&msg).unwrap();
                        miner = Some(Miner::new(message, nonce1.to_string()));
                    }
                    Err(err) => {
                        println!("err IN THREAD {:?}", err)
                    }
                }
                if let Some(m) = &miner {
                    match m.run() {
                        None => {}
                        Some(nonce) => {
                            println!("FOUND CONGRATS");
                            let submit_block_request = Request {
                                id: String::from("1"),
                                method: String::from("mining.submit"),
                                params: vec![
                                    address.clone(),
                                    m.job_id.clone(),
                                    m.extra_nonce_2.clone(),
                                    m.ntime.clone(),
                                    nonce
                                ],
                            };
                            let submit_block_message = format!("{}\n", serde_json::to_string(&submit_block_request).unwrap());
                            client_sender2.try_send(submit_block_message).expect("TODO: panic message");
                        }
                    }
                }
            }
        });

        loop {
            let mut line = String::new();
            buf_reader.read_line(&mut line).await.expect("TODO: panic message");
            println!("Received message: {:?}", line);
            pool_sender.send(line).await.expect("TODO: panic message");
        }
    }

    async fn handle_pool_message(&mut self, raw_message: String) {
        let message: Message = serde_json::from_str(&raw_message).unwrap();
        match message {
            Message::OkResponse(msg) => {
                match &msg.id[..] {
                    "mining.subscribe" => {
                        self.extra_nonce1.lock().expect("CANT BLOCK").push_str(&*msg.result[1].to_string().replace("\"", ""));
                        self.authorize().await;
                        println!("subscribed");
                    }
                    _ => {}
                }
                println!("OkResponse {:?}", msg);
            }
            Message::StandardRequest(msg) => {
                println!("StandardRequest {:?}", msg);
            }
            Message::Notification(msg) => {
                match &msg.method[..] {
                    "mining.notify" => {
                        self.miner_sender.send(raw_message).await.expect("TODO: panic message");
                        println!("mining.notify");
                    }
                    _ => {}
                }
            }
            _ => {
                println!("Unknown message");
            }
        }
    }

    async fn authorize(&self) {
        let auth_request = Request {
            id: String::from("2"),
            method: String::from("mining.authorize"),
            params: vec![self.wallet.clone(), "x".parse().unwrap()],
        };

        let auth_message = format!("{}\n", serde_json::to_string(&auth_request).unwrap());
        self.client_sender.send(auth_message).await.expect("TODO: panic message");
    }
}