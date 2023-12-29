use ethers::{core::abi::ethabi, core::utils::keccak256};
use eyre::Result;

use crate::models::Token;

pub fn to_hex(bytes: &[u8]) -> String {
    format!("0x{}", hex::encode(bytes))
}

pub fn from_hex(hex: &str) -> Result<Vec<u8>> {
    let hex = hex.trim_start_matches("0x");
    let bytes = hex::decode(hex)?;
    Ok(bytes)
}

pub fn encode_data(data: Vec<Token>) -> Result<String> {
    let bytes = ethabi::encode(&data);
    let encoded_data = to_hex(&bytes);

    Ok(encoded_data)
}

pub fn keccak256_hash(bytes: &[u8]) -> String {
    let hash = keccak256(bytes);
    format!("0x{}", hex::encode(hash))
}

pub fn decode_id(id: String) -> Result<[u8; 32]> {
    let id_bytes = hex::decode(id.trim_start_matches("0x"))?;
    let id: [u8; 32] = id_bytes[0..32].try_into().unwrap();
    Ok(id)
}
