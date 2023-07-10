use std::collections::HashMap;
use std::fs::read;
use tokio::io::{ AsyncBufReadExt, AsyncWriteExt, Error, BufReader, BufStream, BufWriter};
use tokio::net::TcpStream;
use std::io;
use std::io::ErrorKind;
use std::str::from_utf8;
use std::sync::Arc;
use jsonrpsee::types::error::ErrorCode;
use serde_json::Value;
use serde::{Deserialize, Serialize};
use tokio::net::tcp::{ReadHalf, WriteHalf};
use tokio::sync::{mpsc, Mutex};
use tokio::sync::mpsc::Sender;
use crate::message::{Message, Response};
use crate::miner::Miner;

#[derive(Serialize, Deserialize)]
struct SubscribeRequest {
    id: String,
    method: String,
    params: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Client {
    //pub stream: Arc<TcpStream>,
    client_sender: Arc<Sender<String>>
}

impl Client {
    pub async fn mine(address: String, threads: i8) {}
    pub async fn connect(address: String) -> Result<(), Error> {
        println!("{:?}", address);
        let (mut reader, mut writer) = TcpStream::connect(address).await.unwrap().into_split();
        let mut buf_reader = BufReader::new(reader);
        let (pool_sender, mut pool_receiver) = mpsc::channel(32);
        let (cs, mut client_receiver) = mpsc::channel(32);
        let client_sender = Arc::new(cs);
        let client = Client {
            client_sender: Arc::clone(&client_sender)
        };
        let subscribe_request = SubscribeRequest {
            id: String::from("mining.subscribe"),
            method: String::from("mining.subscribe"),
            params: vec![],
        };

        let subscribe_message = format!("{}\n", serde_json::to_string(&subscribe_request).unwrap());
        client_sender.send(subscribe_message).await.expect("TODO: panic message");

        tokio::spawn(async move {
            loop {
                let message = client_receiver.recv().await.unwrap();
                writer.write_all(message.as_bytes()).await.unwrap();
            }
        });

        tokio::spawn(async move {
            loop {
                let message = pool_receiver.recv().await.unwrap();
                client.handle_pool_message(message);
            }
        });

        loop {
            let mut line = String::new();
            buf_reader.read_line(&mut line).await.expect("TODO: panic message");
            println!("Received message: {:?}", line);
            pool_sender.send(line).await.expect("TODO: panic message");
        }

        Ok(())
    }

    fn handle_pool_message(&self, raw_message: String) {
        println!("Message to parse {}", raw_message);
        let message: Message = serde_json::from_str(&raw_message).unwrap();
        match message {
            Message::OkResponse(msg) => {
                println!("MESSAG {:?}", msg)
            }
            _ => {
                println!("Unknown message")
            }
        }
        //println!("{:?}", message);
    }
}