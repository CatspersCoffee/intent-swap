
use std::str::FromStr;
use ethers::{
    signers::LocalWallet,
    types::{
        U256, Signature,
    },
};
use crate::consts::*;

pub mod build_eip712_genio_ethers {

    use super::*;
    use ethers::core::types::{U256, H256};
    use ethers_core::types::transaction::eip712::Eip712;
    use ethers::prelude::*;
    use ethers_signers::Signer;

    #[derive(Eip712, Clone, Debug, EthAbiType)]
    #[eip712(
        name = "ZapGeneralizedIO",
        version = "1",
        chain_id = 9889,
        verifying_contract = "0x0000000000000000000000000000000000000001"
    )]
    struct GenIO {
        inputassets: [H256; 5],
        inpututxoids: [H256; 5],
        inputamounts: [U256; 5],
        outputasset: H256,
        outputamount: U256,
        tolerance: U256,
    }


    /// Converts an array of [u8; 32] to an array of H256
    fn convert_to_h256_array(input: [[u8; 32]; 5]) -> [H256; 5] {
        let mut result = [H256::zero(); 5];
        for (i, item) in input.iter().enumerate() {
            result[i] = H256::from_slice(item);
        }
        result
    }

    /// Converts an array of [u8; 32] to an array of U256
    fn convert_to_u256_array(input: [[u8; 32]; 5]) -> [U256; 5] {
        let mut result = [U256::zero(); 5];
        for (i, item) in input.iter().enumerate() {
            result[i] = U256::from_big_endian(item);
        }
        result
    }

    pub async fn get_sig_eip712_by_ethers_genio(
        assets_in: [[u8; 32]; 5],
        utxoids_in:[[u8; 32]; 5],
        amounts_in: [[u8; 32]; 5],
        asset_out: [u8; 32],
        amount_out: [u8; 32],
        tolerance: [u8; 32],
    )  -> Vec<u8> {

        println!("Input Amounts:");
        for (i, amount) in amounts_in.iter().enumerate() {
            let amount_u256 = U256::from_big_endian(amount);
            println!("  Amount {} : {}", i, amount_u256);
        }
        println!("\nOutput Amount:");
        let out_amount_u256 = U256::from_big_endian(&amount_out);
        println!("  Amount: {}", out_amount_u256);


        let tx = GenIO {
            inputassets: convert_to_h256_array(assets_in),
            inpututxoids: convert_to_h256_array(utxoids_in),
            inputamounts: convert_to_u256_array(amounts_in),
            outputasset: H256::from_slice(&asset_out),
            outputamount: U256::from_big_endian(&amount_out),
            tolerance: U256::from(tolerance),
        };

        let wallet_from_key = LocalWallet::from_str(SENDER_EVM_SK).unwrap();
        let wallet = wallet_from_key.with_chain_id(CHAIN_ID_FUEL);

        // Sign the transaction
        let sig = wallet.sign_typed_data(&tx).await.expect("failed to sign typed data");
        let compact_sig = compact(&sig);

        println!("Signature: {:?}", sig);
        println!("Compact Sig: {}", hex::encode(compact_sig));

        // Extract r, s, v from the signature
        let mut r = [0u8; 32];
        let mut s = [0u8; 32];
        sig.r.to_big_endian(&mut r);
        sig.s.to_big_endian(&mut s);
        let v = sig.v as u8;

        // Verify the components of the EIP-712 structure
        let domain_separator = tx.domain().unwrap().separator();
        let type_hash = GenIO::type_hash().unwrap();
        let struct_hash = tx.struct_hash().unwrap();
        let encoded = tx.encode_eip712().unwrap();


        // Print out the results for verification
        println!("Wallet Address: {}", hex::encode(wallet.address()));
        println!(" ");
        println!("Domain Separator: 0x{}", hex::encode(domain_separator));
        println!("Type Hash: 0x{}", hex::encode(type_hash));
        println!("Struct Hash: 0x{}", hex::encode(struct_hash));
        println!("Encoded EIP-712: 0x{}", hex::encode(encoded));
        println!(" ");
        println!("Signature (r): 0x{}", hex::encode(r));
        println!("Signature (s): 0x{}", hex::encode(s));
        println!("Signature (v): {}", v);

        // Verify the signature
        let signer = sig.recover(encoded).expect("failed to recover signer");
        println!("Signer: {}", hex::encode(signer));

        compact_sig.to_vec()
    }

}

// credit: fuel-labs
fn compact(signature: &Signature) -> [u8; 64] {
    let shifted_parity = U256::from(signature.v - 27) << 255;

    let r = signature.r;
    let y_parity_and_s = shifted_parity | signature.s;

    let mut sig = [0u8; 64];
    let mut r_bytes = [0u8; 32];
    let mut s_bytes = [0u8; 32];
    r.to_big_endian(&mut r_bytes);
    y_parity_and_s.to_big_endian(&mut s_bytes);
    sig[..32].copy_from_slice(&r_bytes);
    sig[32..64].copy_from_slice(&s_bytes);

    sig
}