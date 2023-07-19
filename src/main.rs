#![allow(warnings)]
mod client;
mod miner;
mod message;

use std::io::Error;
use client::Client;
use clap::Parser;

//struct representation of command line arguments
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub(crate) struct Args {
    #[arg(short, long)]
    pub address: String,

    #[arg(short, long)]
    pub wallet: String,
}

// async runtime marcos
#[tokio::main]
async fn main() {
    // parse arguments
    let args = Args::try_parse().unwrap();

    Client::mine(args.address, args.wallet).await.unwrap();
}
