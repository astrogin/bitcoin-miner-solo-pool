use std::collections::HashMap;
use std::time::SystemTime;
use clap::builder::Str;
use primitive_types::U256;
use rand::Rng;
use serde_json::Value;
use sha256::digest;
use crate::client::MinerDataMessage;
use crate::message::Notification;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Miner {
    pub extra_nonce_1: String,
    pub extra_nonce_2: String,
    pub nbits: String,
    pub target: U256,
    pub merkle_root: String,
    pub job_id: String,
    pub ntime: String,
    pub version: String,
    pub prev_hash: String,
}

impl Miner {
    pub fn new(msg: MinerDataMessage, extra_nonce_1: String) -> Miner {
        let nbits: String = msg.params[6].to_string().replace("\"", "");
        // TODO make random
        let extra_nonce_2: String = "37f0cca00000".parse().unwrap();

        let target = format!(
            "{}{}",
            &nbits[2..],
            "00".repeat(usize::from_str_radix(&nbits[..2], 16).unwrap() - 3)
        )
            .parse::<String>()
            .unwrap();
        let target = format!("{}{}", "0".repeat(64 - target.len()), target);
        let target = U256::from_str_radix(target.as_str(), 16).unwrap();

        let job_id = msg.params[0].to_string().replace("\"", "");

        let now: isize = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs() as isize;
        let server_time =
            isize::from_str_radix(msg.params[7].to_string().replace("\"", "").as_str(), 16)
                .unwrap();
        let mut time_delta = server_time - now;
        let ntime = format!("{:x}", now + time_delta);

        let mut raw_coinbase = String::new();
        raw_coinbase.push_str(&msg.params[2].to_string().replace("\"", "")); //coinb1
        raw_coinbase.push_str(&extra_nonce_1.replace("\"", ""));
        raw_coinbase.push_str(&extra_nonce_2.replace("\"", ""));
        raw_coinbase.push_str(&msg.params[3].to_string().replace("\"", "")); //coinb2

        let coinbase_transaction = digest(digest(raw_coinbase));

        let mut merkle_root = coinbase_transaction;

        for txid in msg.params[4].as_array().unwrap() {
            merkle_root.push_str(txid.to_string().as_str());
            merkle_root = digest(digest(merkle_root));
        }

        let mut merkle_root_le =
            hex::encode(&Miner::sha256_string_to_le_bytes(merkle_root.as_str()).unwrap());

        return Miner {
            extra_nonce_1,
            extra_nonce_2,
            nbits,
            target,
            merkle_root: merkle_root_le,
            job_id,
            ntime,
            version: msg.params[5].to_string().replace("\"", ""),
            prev_hash: msg.params[1].to_string().replace("\"", ""),
        };
    }

    fn sha256_string_to_le_bytes(input: &str) -> Result<[u8; 32], String> {
        // Parse the input string as hexadecimal bytes
        let bytes = match hex::decode(input) {
            Ok(bytes) => bytes,
            Err(_) => return Err(String::from("Invalid input string")),
        };

        if bytes.len() != 32 {
            return Err(String::from("Input string must represent 32 bytes"));
        }

        // Reverse the byte order to convert to little-endian
        let mut le_bytes = [0u8; 32];
        for i in 0..32 {
            le_bytes[i] = bytes[31 - i];
        }

        Ok(le_bytes)
    }

    fn build_header(
        version: &String,
        prev_hash: &String,
        nonce: &String,
        ntime: &String,
        merkle_root_le: &String,
        nbits: &String,
    ) -> U256 {
        let mut header = String::new();
        header.push_str(version); //version
        header.push_str(prev_hash); //prev hash
        header.push_str(merkle_root_le); //merkle
        header.push_str(ntime); //time
        header.push_str(nbits); // bits
        header.push_str(nonce); // nonce
        header.push_str(&"000000800000000000000000000000000000000000000000000000000000000000000000000000000000000080020000"); // padding

        let hash = U256::from_str_radix(digest(digest(header)).as_str(), 16).unwrap();

        hash
    }

    pub fn run(&self) -> Option<String> {
        let mut rng = rand::thread_rng();
        let nonce: String = (0..4).map(|_| format!("{:02x}", rng.gen::<u8>())).collect();
        let hash = Miner::build_header(
            &self.version,
            &self.prev_hash,
            &nonce,
            &self.ntime,
            &self.merkle_root,
            &self.nbits,
        );

        if self.target > hash {
            return Some(nonce);
        }

        None
    }
}