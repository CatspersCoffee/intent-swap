use std::str::FromStr;
use ethers::types::Address as EthAddress;
use ethers_signers::{LocalWallet, Signer as EthSigner};

use fuels::prelude::*;
use fuel_crypto::SecretKey;

use crate::consts::*;


pub fn get_evm_addr() -> EthAddress {
    let eth_wallet = create_account(SENDER_EVM_SK);
    eth_wallet.address()
}

/// Creates an EVM account from a private key
fn create_account(private_key: &str) -> LocalWallet {
    private_key
        .parse::<LocalWallet>()
        .unwrap()
        .with_chain_id(CHAIN_ID_FUEL)
}


pub fn get_fuel_wallet(provider: &Provider) -> WalletUnlocked {
    let fuel_sk = FUEL_EMPTY_WALLET_SK.to_string();
    let wallet = WalletUnlocked::new_from_private_key(
        SecretKey::from_str(&fuel_sk).unwrap(),
        Some(provider.clone())
    );
    wallet
}