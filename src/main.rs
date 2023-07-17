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

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub(crate) struct Args {
    #[arg(short, long)]
    pub address: String,

    #[arg(short, long)]
    pub wallet: String,
}


#[tokio::main]
async fn main() -> Result<(), Error> {
    let args = Args::try_parse().expect("Wrong arguments");
    Client::connect(args.address, args.wallet).await.expect("TODO: panic message");
    Ok(())
}
