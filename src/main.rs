#![allow(warnings)]

mod client;
mod miner;
mod message;

use core::ops::Not;
use primitive_types::U256;
use rand::Rng;
use serde_json::{from_str, to_vec, Value};
use sha256::digest;
use std::collections::HashMap;
use std::fmt::format;
use std::io::{BufRead, BufReader, Error, ErrorKind, Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::str::{from_utf8, FromStr};
use std::time::SystemTime;
use std::vec;

use client::Client;
use clap::Parser;
use tokio::sync::mpsc;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub(crate) struct Args {
    /// Name of the person to greet
    #[arg(short, long)]
    pub address: String,

    #[arg(short, long)]
    pub wallet: String,
}


#[tokio::main]
async fn main() -> Result<(), Error> {
    let args = Args::try_parse().expect("Wrong arguments");
    /*let (tx, mut rx) = mpsc::channel(32);

    let task = tokio::spawn(async move {
        loop {
            println!("FIRST");
            let message = rx.recv().await.unwrap();
            println!("GOT = {}", message);
        }
    });

    let (tx1, mut rx1) = mpsc::channel(32);

    let task2 = tokio::spawn(async move {
        loop {
            println!("SECOND");
            let message = rx1.recv().await.unwrap();
            println!("GOT1 = {}", message);
        }
    });

    tx.send("sending from first handle").await;
    task.await;
    tx1.send("sending from first handle1").await;
    task2.await;*/
    let client = Client::connect(args.address, args.wallet).await;
    println!("client {:?}", client);
    //let mut stream = TcpStream::connect("btc.zsolo.bid:6057")?;
    /*let mut stream = TcpStream::connect("solo.ckpool.org:3333")?;
    let message =
        "{\"id\": \"mining.subscribe\", \"method\": \"mining.subscribe\", \"params\": []}\n";
    stream.write_all(message.as_bytes())?;
    let mut buf = [0; 1024];
    let mut reader = BufReader::new(stream.try_clone()?);
    let mut buf_line = String::new();
    match reader.read_line(&mut buf_line) {
        Err(e) => panic!("Got an error: {}", e),
        Ok(0) => return Err(Error::new(ErrorKind::BrokenPipe, "Connection closed")),
        Ok(_) => (),
    };

    let subscribe_data: HashMap<String, Value> = serde_json::from_str(&buf_line).unwrap();
    let extra_nonce1 = subscribe_data["result"][1].to_string().replace("\"", "");
    let extra_nonce2 = "37f0cca00000";
    println!("CONNECT: {:?}", subscribe_data);
    let auth_message = "{\"params\": [\"bc1qwjuuut8cd23ws2ks5ppt5e59573d8c83hpql2s\", \"x\"], \"id\": 2, \"method\": \"mining.authorize\"}\n";

    stream.write_all(auth_message.as_bytes())?;

    loop {
        let mut buf_line2 = String::new();
        if reader
            .read_line(&mut buf_line2)
            .expect("Cannot read new line")
            == 0
        {
            break;
        }
        let job: HashMap<String, Value> = serde_json::from_str(&buf_line2).unwrap();
        if job.contains_key("method") && job["method"].eq(&String::from("mining.notify")) {
            let nbits: String = job["params"][6].to_string().replace("\"", "");
            let target = format!(
                "{}{}",
                &nbits[2..],
                "00".repeat(usize::from_str_radix(&nbits[..2], 16).unwrap() - 3)
            )
            .parse::<String>()
            .unwrap();

            let target = format!("{}{}", "0".repeat(64 - target.len()), target);

            println!("TARGET HASH: {:?}", target);

            let target = U256::from_str_radix(target.as_str(), 16).unwrap();


            let job_id = job["params"][0].to_string().replace("\"", "");

            let now: isize = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs() as isize;
            let server_time =
                isize::from_str_radix(job["params"][7].to_string().replace("\"", "").as_str(), 16)
                    .unwrap();
            let mut time_delta = server_time - now;
            let ntime = format!("{:x}", now + time_delta);

            let mut raw_coinbase = String::new();
            raw_coinbase.push_str(&job["params"][2].to_string().replace("\"", "")); //coinb1
            raw_coinbase.push_str(&extra_nonce1.replace("\"", ""));
            raw_coinbase.push_str(&extra_nonce2.replace("\"", ""));
            raw_coinbase.push_str(&job["params"][3].to_string().replace("\"", "")); //coinb2

            let coinbase_transaction = digest(digest(raw_coinbase));

            let mut merkle_root = coinbase_transaction;

            for txid in job["params"][4].as_array().unwrap() {
                merkle_root.push_str(txid.to_string().as_str());
                merkle_root = digest(digest(merkle_root));
            }

            let mut merkle_root_le =
                hex::encode(&sha256_string_to_le_bytes(merkle_root.as_str()).unwrap());

            let nbits: String = job["params"][6].to_string().replace("\"", "");
            let mut nonce = 0;
            /*loop {
                let mut rng = rand::thread_rng();
                let nonce: String = (0..4).map(|_| format!("{:02x}", rng.gen::<u8>())).collect();
                let hash = build_header(&job, &nonce, &ntime, &merkle_root_le, &nbits);

                if (target > hash) {
                    println!("FOUND CONGRATS");
                    let submit_message = format!(
                    "{{\"params\": [\"bc1qwjuuut8cd23ws2ks5ppt5e59573d8c83hpql2s\", \"{}\", \"{}\", \"{}\", \"{}\"], \"id\": 1, \"method\": \"mining.submit\"}}\n",
                    job_id,
                    extra_nonce2,
                    ntime,
                    nonce
                );
                    println!("CONGRATS MESSAGE: {:?}", submit_message);

                    stream.write_all(submit_message.as_bytes())?;
                    break;
                }
            }*/
        }
        println!("message received: {}", buf_line2);
    }*/

    Ok(())
}

fn build_header(
    job: &HashMap<String, Value>,
    nonce: &String,
    ntime: &String,
    merkle_root_le: &String,
    nbits: &String,
) -> U256 {
    let mut header = String::new();
    header.push_str(&job["params"][5].to_string().replace("\"", "")); //version
    header.push_str(&job["params"][1].to_string().replace("\"", "")); //prev hash
    header.push_str(&merkle_root_le); //merkle
    header.push_str(&ntime); //time
    header.push_str(&nbits); // bits
    header.push_str(&nonce); // nonce
    header.push_str(&"000000800000000000000000000000000000000000000000000000000000000000000000000000000000000080020000"); // padding

    let hash = U256::from_str_radix(digest(digest(header)).as_str(), 16).unwrap();

    hash
}
