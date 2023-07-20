use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufStream, BufWriter};
use tokio::net::TcpStream;
use std::{io, thread};
use std::error::Error;
use std::sync::{Arc, Mutex};
use tokio::sync::{mpsc};
use tokio::sync::mpsc::Sender;
use tokio::sync::mpsc::Receiver;
use crate::message::{Message, MinerDataMessage, Notification, Request, Response};
use crate::miner::Miner;

#[derive(Debug, Clone)]
pub struct Client {
    client_sender: Arc<Sender<String>>,
    miner_sender: Sender<String>,
    wallet: String,
    extra_nonce1: String,
}

impl Client {
    pub async fn mine(address: String, wallet: String) -> Result<(), Box<dyn Error>> {
        //connect to pool
        let (mut reader, mut writer) = TcpStream::connect(address).await?.into_split();
        //create read buffer
        let mut buf_reader = BufReader::new(reader);
        //this channel we are going to use for receive messages from stream pool
        let (pool_sender, mut pool_receiver) = mpsc::channel(32);
        //this channel we are going to use for send/receive message from/to miner thread
        let (mut miner_sender, mut miner_receiver) = mpsc::channel(32);
        //this channel we are going to send message to stream pool from our client
        let (cs, mut client_receiver) = mpsc::channel(32);
        let client_sender = Arc::new(cs);
        //this sender we are going to use in miner thread
        let client_sender2 = Arc::clone(&client_sender);
        //initialize client with data
        let mut client = Client {
            client_sender: Arc::clone(&client_sender),
            wallet: wallet.clone(),
            extra_nonce1: String::new(),
            miner_sender,
        };

        //create subscribe message
        let subscribe_request = Request {
            id: String::from("mining.subscribe"),
            method: String::from("mining.subscribe"),
            params: vec![],
        };
        //serialize it to json
        let subscribe_message = format!("{}\n", serde_json::to_string(&subscribe_request).unwrap());
        //send the message to channel
        client_sender.send(subscribe_message).await?;
        //this task will send messages from client to mining pool
        tokio::spawn(async move {
            loop {
                let message = client_receiver.recv().await.unwrap();
                println!("Message sent {:?}", message);
                writer.write_all(message.as_bytes()).await.unwrap();
            }
        });
        // that task receive and handle messages from pool
        tokio::spawn(async move {
            loop {
                let message = pool_receiver.recv().await.unwrap();
                client.handle_pool_message(message).await;
            }
        });

        // this thread using for mining work
        thread::spawn(move || {
            let mut miner: Option<Miner> = None;
            loop {
                //receive new data from pool
                match miner_receiver.try_recv() {
                    Ok(msg) => {
                        //parse message
                        let message: MinerDataMessage = serde_json::from_str(&msg).unwrap();
                        let nonce1= message.nonce1.clone().to_string();
                        //create miner with new data
                        miner = Some(Miner::new(message, nonce1));
                    }
                    Err(err) => {
                        //println!("err IN THREAD {:?}", err)
                    }
                }
                // if miner exists we mine.
                if let Some(m) = &miner {
                    //this function return nonce if solution found
                    match m.run() {
                        None => {}
                        Some(nonce) => {
                            println!("FOUND CONGRATS");
                            // create submit block message for found block
                            let submit_block_request = Request {
                                id: String::from("1"),
                                method: String::from("mining.submit"),
                                params: vec![
                                    wallet.clone(),
                                    m.job_id.clone(),
                                    m.extra_nonce_2.clone(),
                                    m.ntime.clone(),
                                    nonce,
                                ],
                            };
                            let submit_block_message = format!("{}\n", serde_json::to_string(&submit_block_request).unwrap());
                            //send the message to pool task
                            client_sender2.try_send(submit_block_message).unwrap();
                            break;
                        }
                    }
                }
            }
        });
        // in that loop we listen new messages from pool and read them from buffer
        loop {
            let mut line = String::new();
            buf_reader.read_line(&mut line).await.unwrap();
            println!("Received message: {:?}", line);
            pool_sender.send(line).await.unwrap();
        }
    }

    async fn handle_pool_message(&mut self, raw_message: String) {
        //parse message from json
        let message: Message = serde_json::from_str(&raw_message).unwrap();
        //check type of the message
        match message {
            Message::OkResponse(msg) => {
                match &msg.id[..] {
                    "mining.subscribe" => {
                        //in subscription response we need to store extra_nonce_1 for mining
                        self.extra_nonce1.push_str(&*msg.result[1].to_string().replace("\"", ""));
                        //after subscription we need to authorize for mining work to receive new blocks
                        self.authorize().await;
                        println!("subscribed");
                    }
                    _ => {}
                }
            }
            Message::Notification(msg) => {
                match &msg.method[..] {
                    "mining.notify" => {
                        //put all data which need for mining together
                        let message: MinerDataMessage = MinerDataMessage {
                            method: msg.method,
                            params: msg.params,
                            nonce1: self.extra_nonce1.clone(),
                        };
                        //send to mining thread
                        self.miner_sender.send(serde_json::to_string(&message).unwrap()).await.unwrap();
                    }
                    _ => {}
                }
            }
            _ => {
                println!("Unknown message");
            }
        }
    }
    // using for authorization in pool
    async fn authorize(&self) {
        //create authorization message
        let auth_request = Request {
            id: String::from("2"),
            method: String::from("mining.authorize"),
            params: vec![self.wallet.clone(), "x".parse().unwrap()],
        };

        let auth_message = format!("{}\n", serde_json::to_string(&auth_request).unwrap());
        //send to pool task
        self.client_sender.send(auth_message).await.unwrap();
    }
}