pub type U256 = ethers::types::U256;
pub type Token = ethers::core::abi::ethabi::Token;

#[derive(Debug)]
pub struct SchemaItem {
    pub name: String,
    pub kind: ParamType,
}

#[derive(Debug)]
pub enum ParamType {
    Address,
    String,
    Bool,
    Bytes32,
    Bytes,
    Uint8,
    Uint16,
    Uint32,
    Uint64,
    Uint128,
    Uint256,
}

#[derive(Debug)]
pub struct SchemaItemWithSignature {
    pub name: String,
    pub kind: String,
    pub signature: String,
}

pub struct SchemaDecodedItem {
    pub name: String,
    pub kind: String,
    pub value: SchemaItem,
    pub signature: String,
}
