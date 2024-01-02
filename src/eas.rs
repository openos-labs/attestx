use crate::utils::{decode_id, from_hex, to_hex};
use ethers::{
    contract::abigen,
    middleware::SignerMiddleware,
    prelude::{Abigen, Address, TransactionReceipt},
    providers::{Http, Middleware, Provider},
    signers::Signer,
    types::U256,
};

use crate::consts;
use eyre::Result;
use std::sync::Arc;

pub type MyWallet = ethers::signers::Wallet<ethers::core::k256::ecdsa::SigningKey>;

abigen!(EASContract, "contracts/abi/EAS.json",);
abigen!(SchemaRegistry, "contracts/abi/SchemaRegistry.json",);

#[derive(Debug)]
pub struct EAS {
    pub chain_id: u64,
    pub eas_contract: Address,
    pub registry_contract: Address,
    pub client: Arc<SignerMiddleware<Provider<Http>, MyWallet>>,
}

impl Clone for EAS {
    fn clone(&self) -> Self {
        Self {
            chain_id: self.chain_id,
            eas_contract: self.eas_contract,
            registry_contract: self.registry_contract,
            client: self.client.clone(),
        }
    }
}

impl EAS {
    pub async fn new(url: String, private_key: String) -> Result<Self> {
        let provider =
            Provider::<Http>::try_from(url).expect("could not instantiate HTTP Provider");

        let chain_id = provider.get_chainid().await?;

        let wallet: MyWallet = private_key.parse()?;

        Ok(Self {
            chain_id: chain_id.as_u64(),
            eas_contract: Address::zero(),
            registry_contract: Address::zero(),
            client: Arc::new(SignerMiddleware::new(
                provider,
                wallet.with_chain_id(chain_id.as_u64()),
            )),
        })
    }

    pub fn with_eas_contract(&mut self, eas_contract: String) -> Result<&mut Self> {
        self.eas_contract = eas_contract.parse::<Address>()?;
        Ok(self)
    }

    pub fn with_registry_contract(&mut self, registry_contract: String) -> Result<&mut Self> {
        self.registry_contract = registry_contract.parse::<Address>()?;
        Ok(self)
    }

    pub fn with_wallet(&mut self, private_key: String) -> Result<&mut Self> {
        let wallet: MyWallet = private_key.parse()?;
        self.client = Arc::new(self.client.with_signer(wallet.with_chain_id(self.chain_id)));
        Ok(self)
    }

    pub async fn deploy_schema_registry(&mut self) -> Result<String> {
        let registry_contract = SchemaRegistry::deploy(self.client.clone(), ())?
            .send()
            .await?;

        self.registry_contract = registry_contract.address();

        let registry_address = to_hex(registry_contract.address().as_bytes());

        Ok(registry_address)
    }

    pub async fn deploy_eas(&mut self, registry_address: String) -> Result<String> {
        let registry_address = registry_address.parse::<Address>()?;
        let eas_contract = EASContract::deploy(self.client.clone(), registry_address)?
            .send()
            .await?;

        self.eas_contract = eas_contract.address().clone();

        let eas_address = to_hex(eas_contract.address().as_bytes());

        Ok(eas_address)
    }

    pub async fn get_version(&self) -> Result<String> {
        let eas_contract = EASContract::new(self.eas_contract, self.client.clone());

        let version = eas_contract.version().call().await?;

        Ok(version)
    }

    pub async fn get_attestation(&self, uid: String) -> Result<Attestation> {
        let eas_contract = EASContract::new(self.eas_contract, self.client.clone());

        let uid = decode_id(uid)?;

        let attestation = eas_contract.get_attestation(uid).call().await?;

        Ok(attestation)
    }

    pub async fn is_attestation_valid(&self, uid: String) -> Result<bool> {
        let eas_contract = EASContract::new(self.eas_contract, self.client.clone());

        let uid = decode_id(uid)?;

        let valid = eas_contract.is_attestation_valid(uid).call().await?;

        Ok(valid)
    }

    pub async fn new_schema(
        &self,
        schema: String,
        resolver_address: String,
        revocable: bool,
    ) -> Result<String> {
        let registry_contract = SchemaRegistry::new(self.registry_contract, self.client.clone());

        let resolver_address = resolver_address.parse::<Address>()?;

        let receipt: Option<TransactionReceipt> = registry_contract
            .register(schema, resolver_address, revocable)
            .send()
            .await?
            .await?;

        match receipt {
            Some(receipt) => {
                let logs = receipt.logs;
                if logs.len() > 0 {
                    let log = &logs[0];
                    let topics = &log.topics;
                    if topics.len() == 3 && hex::encode(topics[0]) == consts::TOPIC_SCHEMA {
                        let uid = to_hex(topics[1].as_bytes());
                        return Ok(uid);
                    }
                }
            }
            None => {}
        }
        Err(eyre::format_err!("error to register schema"))
    }

    pub async fn get_schema(&self, uid: String) -> Result<SchemaRecord> {
        let registry_contract = SchemaRegistry::new(self.registry_contract, self.client.clone());

        let uid = decode_id(uid)?;

        let schema = registry_contract.get_schema(uid).call().await?;

        Ok(schema)
    }

    pub async fn new_attestation(
        &self,
        schema_id: String,
        recipient: String,
        expiration_time: u64,
        revocable: bool,
        ref_uid: Option<String>,
        data: String,
    ) -> Result<String> {
        let eas_contract = EASContract::new(self.eas_contract, self.client.clone());

        let schema = decode_id(schema_id)?;

        let ref_uid = match ref_uid {
            Some(ref_uid) => decode_id(ref_uid)?,
            None => [0; 32],
        };

        let req_data = AttestationRequestData {
            recipient: recipient.parse::<Address>().unwrap(),
            expiration_time: expiration_time,
            revocable: revocable,
            ref_uid: ref_uid,
            value: U256::zero(),
            data: from_hex(data.as_str())?.into(),
        };

        let request = AttestationRequest {
            schema: schema,
            data: req_data,
        };

        let receipt: Option<TransactionReceipt> =
            eas_contract.attest(request).send().await?.await?;

        match receipt {
            Some(receipt) => {
                let logs = receipt.logs;
                if logs.len() > 0 {
                    let log = &logs[0];
                    let topics = log.topics.clone();
                    if topics.len() > 2 && to_hex(topics[0].as_bytes()) == consts::TOPIC_ATTESTATION
                    {
                        let uid = format!("0x{}", hex::encode(log.data.clone()));
                        return Ok(uid);
                    }
                }
            }
            None => {}
        }
        Err(eyre::format_err!("error to attest"))
    }
}

pub async fn new_attestation_offchain() -> Result<()> {
    Ok(())
}

pub fn generate_bindings() -> Result<()> {
    Abigen::new("SchemaRegistry", "contracts/abi/SchemaRegistry.json")?
        .generate()?
        .write_to_file("contracts/abi/schema_registry.rs")?;

    Abigen::new("EAS", "contracts/abi/EAS.json")?
        .generate()?
        .write_to_file("contracts/abi/eas.rs")?;

    Ok(())
}

#[cfg(test)]
mod tests {

    use super::*;

    #[tokio::test]
    async fn test_deploy() {
        let url = "https://rpc.ankr.com/eth_goerli/".to_string();
        let private_key = "".to_string();

        let mut eas = EAS::new(url, private_key).await.unwrap();

        let registry_address = eas.deploy_schema_registry().await.unwrap();
        let eas_address = eas.deploy_eas(registry_address.clone()).await.unwrap();

        println!("registry_address: {}\n", registry_address);
        println!("eas_address: {}\n", eas_address);
        println!("len: {}", registry_address.len());

        // turn the rlp bytes encoding into a rlp stream and check that the decoding returns the
        // same struct

        // We compare the sighash rather than the specific struct
        assert!(registry_address.len() == 42 && eas_address.len() == 42);
    }
}
