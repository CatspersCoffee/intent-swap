use sha2::{Sha256, Digest};
use tokio::io::Error;
use fuels::{
    prelude::*,
    prelude::{
        Address, ContractId,
    },
    types::{
        Bits256, Bytes32,
        transaction::TxPolicies,
    },
    accounts::wallet::WalletUnlocked,
};


pub mod setup_case_01 {
    use super::*;
    use crate::interfaces::{
        dummy_pwallet::get_dummypwallet_info,
        generalized_swap_verifier_interface::contract_verifier_instance,
        tokenminter,
    };

    pub async fn setup_predicate_and_assets_sdk_provider() -> std::result::Result<(
        Provider,
        ContractId,     // SwapVerifier contractid.
        Vec<u8>,        // dummypwallet predicate bytecode.
        Bech32Address,  // dummypwallet address.
        Vec<u8>,        // dummypsolver predicate bytecode.
        Bech32Address,  // dummypsolver address.
        Bytes32,        // token in asset id.
        Bytes32,        // token out asset id.
    ), Error> {

        let mut node_config = NodeConfig::default();
        node_config.starting_gas_price = 1;

        let wallets_config = WalletsConfig::new(
                Some(1),             /* Single wallet */
                Some(1),             /* Single coin (UTXO) */
                Some(1_000_000_000_000), /* Amount per coin */
        );

        let mut wallets = launch_custom_provider_and_get_wallets(
            wallets_config,
            Some(node_config),
            None,
        )
        .await
        .unwrap();
        let wallet = wallets.pop().unwrap();
        let provider = wallet.provider().clone().unwrap();

        let (_swapverifier_instance, swapverifier_cid) = contract_verifier_instance(wallet.clone()).await;

        // create a dummy wallet that only holds the asset going in to the swap.
        let (dummypwallet_bytecode, _, _) = get_dummypwallet_info(0u8);

        let dummypwallet_predicate: Predicate =
            Predicate::from_code(dummypwallet_bytecode.clone())
                .with_provider(provider.clone());
        let dummypwallet_predicate_address: Address = dummypwallet_predicate.address().into();

        // create a dummy solver wallet that holds the desired swap asset (the out asset) and some gas
        let (dummypsolver_bytecode, _, _) = get_dummypwallet_info(1u8);

        let dummypsolver_predicate: Predicate =
            Predicate::from_code(dummypsolver_bytecode.clone())
                .with_provider(provider.clone());
        let dummypsolver_predicate_address: Address = dummypsolver_predicate.address().into();

        // wallet transfers base asset (for gas) to solver predicate. Solver will pay the gas for the tx.
        wallet
            .transfer(
                &dummypsolver_predicate.address(),
                5000001,
                AssetId::BASE,
                TxPolicies::default()
            )
            .await
            .unwrap();

        // deploy a token minter contract, and send some amount different types of assets to the two
        // dummy predicate wallet and solver addresses.
        let pwals: Vec<Address> = vec![
            dummypwallet_predicate_address,
            dummypsolver_predicate_address
        ];
        let (assetin_assetid, assetout_assetid) = deploy_token_minter_mint_to(
            &wallet,
            pwals,
        ).await;

        println!("\n------------------------------------------------------------------- (addresses):");
        println!("SwapVerifier: {}", hex::encode(swapverifier_cid));
        println!(" ");
        println!("DummyPredicateWallet address: {}", hex::encode(dummypwallet_predicate_address));
        println!(" ");
        println!("DummyPredicateSolver address: {}", hex::encode(dummypsolver_predicate_address));

        Ok((
            provider.clone(),
            swapverifier_cid,
            dummypwallet_bytecode,
            dummypwallet_predicate.address().clone(),
            dummypsolver_bytecode,
            dummypsolver_predicate.address().clone(),
            assetin_assetid,
            assetout_assetid,
        ))
    }

    /// Deploy a fresh TokenMinter contract.
    pub async fn deploy_token_minter_mint_to(
        wallet_with_gas: &WalletUnlocked,
        address_to: Vec<Address>,
    ) -> (Bytes32, Bytes32) {

        // Deploying TokenMinter contract (uses sdk wallet
        let tokenminter_cid = tokenminter::deploy_tokenminter(wallet_with_gas).await;

        let assetin_sub_id = Bits256::from_hex_str(&u64_to_bits256_hex(0u64)).unwrap();
        let assetout_sub_id = Bits256::from_hex_str(&u64_to_bits256_hex(1u64)).unwrap();

        // mint the first 1_000_000_000 and send to address idx 0
        tokenminter::mint_to(
            tokenminter_cid,
            wallet_with_gas,
            *address_to.get(0).unwrap(),
            1_000_000_000u64,
            assetin_sub_id,
        ).await;

        // mint the second 500_000_000 and send to address idx 0
        tokenminter::mint_to(
            tokenminter_cid,
            wallet_with_gas,
            *address_to.get(0).unwrap(),
            520_000_000u64,
            assetin_sub_id,
        ).await;

        // mint the third 2_000_000_000 and send to address idx 1
        tokenminter::mint_to(
            tokenminter_cid,
            wallet_with_gas,
            *address_to.get(1).unwrap(),
            2_000_000_000u64,
            assetout_sub_id,
        ).await;

        println!(" ");
        let tmcid_hex = tokenminter_cid.as_slice();
        println!("TokenMinter contractid hex: {}", hex::encode(tmcid_hex));

        let assetin_assetid = get_assetid_for_subid_and_cid(
            Bytes32::from(assetin_sub_id.0), tokenminter_cid);
        println!("Asset In: ");
        println!("sub_id  : {}", hex::encode(assetin_sub_id.0));
        println!("assetid : {}", hex::encode(assetin_assetid));

        let assetout_assetid = get_assetid_for_subid_and_cid(
            Bytes32::from(assetout_sub_id.0), tokenminter_cid);
        println!("Asset Out: ");
        println!("sub_id  : {}", hex::encode(assetout_sub_id.0));
        println!("assetid : {}", hex::encode(assetout_assetid));

        (assetin_assetid, assetout_assetid)

    }

}

fn u64_to_bits256_hex(value: u64) -> String {
    format!("0x{:064x}", value)
}

fn get_assetid_for_subid_and_cid(
    sub_id: Bytes32,
    contract: ContractId
) -> Bytes32 {
    let mut hasher = Sha256::new();
    hasher.update(*contract);
    hasher.update(*sub_id);
    Bytes32::from(<[u8; 32]>::from(hasher.finalize()))
}
