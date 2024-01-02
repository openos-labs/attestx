pub mod consts;
pub mod eas;
pub mod models;
pub mod utils;

use crate::eas::EAS;
use crate::utils::encode_data;
use models::{Token, U256};

use eyre::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let url = "https://data-seed-prebsc-1-s1.bnbchain.org:8545".to_string();

    let private_key = "".to_string();
    let mut eas = EAS::new(url, private_key).await?;

    let registry_address = "0x08C8b8417313fF130526862f90cd822B55002D72".to_string();
    let eas_address = "0x6c2270298b1e6046898a322acB3Cbad6F99f7CBD".to_string();
    let resolver_address = "0x0000000000000000000000000000000000000000".to_string();
    // let eip712_address = "0x3b32B97092f09Ad34E5766e239e4C2F76b0DEe43".to_string();
    // let indexer_address = "0x10E0a439F2A96FB58F1800C495C91cf86a2b9411".to_string();

    // deploy attestation service
    // let registry_address = eas.deploy_schema_registry().await?;
    // let eas_address = eas.deploy_eas(registry_address.clone()).await?;

    println!("registry_address: {}\n", registry_address);
    println!("eas_address: {}\n", eas_address);

    let eas = eas
        .with_registry_contract(registry_address.clone())?
        .with_eas_contract(eas_address)?;

    let schema = "uint64 twitterId, string name, uint64 followers, uint64 posts".to_string();
    let revocable = true;

    // create schema
    let schema_id = eas
        .new_schema(schema.clone(), resolver_address, revocable)
        .await?;

    //let schema_id =
    // "0xa1d3c7a85714564a3817eaafd31d14714d1c4144bc528c3d1b0aa46984939c82".to_string();
    println!("schema_id: {}\n", schema_id);

    // create attestation
    let recipient = "0x000000ff41e81e2ed086931cddef697f2b5529bf".to_string();
    let expiration_time = 0;
    let revocable = true;
    let ref_uid = None;

    let mut params: Vec<Token> = vec![];
    params.push(Token::Uint(U256::from(100000)));
    params.push(Token::String("bitneo".to_string()));
    params.push(Token::Uint(U256::from(1888)));
    params.push(Token::Uint(U256::from(10)));

    let data = encode_data(params)?;

    let att_id = eas
        .new_attestation(
            schema_id.clone(),
            recipient,
            expiration_time,
            revocable,
            ref_uid,
            data,
        )
        .await?;

    println!("attestation_id: {}\n", att_id);
    Ok(())
}
