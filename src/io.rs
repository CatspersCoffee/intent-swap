use std::result::Result as StdResult;
use std::fmt;
use fuels::{
    prelude::*,
    accounts::{
        predicate::Predicate, Account,
    },
    types::{
        input::Input as SdkInput,
        ChainId, Bytes32,
        coin_type::CoinType,
        coin::Coin,
    },
};


#[derive(Debug)]
pub enum UTXOToolsError {
    NoUTXOFound,
    UnexpectedInputType,
}

impl fmt::Display for UTXOToolsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            UTXOToolsError::NoUTXOFound => write!(f, "No UTXO found in the input"),
            UTXOToolsError::UnexpectedInputType => write!(f, "Unexpected input type"),
        }
    }
}
impl std::error::Error for UTXOToolsError {}



pub mod utxo_input_utils {
    use super::*;
    use fuels::types::coin_type::CoinType;

    pub async fn get_predi_input(
        provider: &Provider,
        predi_bytecode: &Vec<u8>,
        predi_data: Option<Vec<u8>>,
        asset_id: AssetId,
        target_amount: u64,
    ) -> StdResult<(Vec<SdkInput>, Bech32Address, Address), UTXOToolsError> {

        let mut predicate = Predicate::from_code(predi_bytecode.clone())
            .with_provider(provider.clone());

        if let Some(data) = predi_data {
            predicate = predicate.with_data(data);
        }

        println!("get_predi_input:");
        println!("\tasset_id       : {}", hex::encode(asset_id));
        println!("\ttarget_amount  : {}", target_amount);
        println!(" ");

        let predicate_b32addr = predicate.address().clone();
        let predicate_address: Address = predicate.address().into();
        let mut predicate_inputs: Vec<SdkInput> = Vec::new();      // target asset inputs
        predicate_inputs.append(
            &mut predicate
                .get_asset_inputs_for_amount(
                    asset_id,
                    target_amount,
                    None
                )
                .await
                .unwrap(),
        );

        Ok((predicate_inputs, predicate_b32addr, predicate_address))
    }

    pub async fn find_inputs_by_resource_filter(
        provider: &Provider,
        predi_bytecode: &Vec<u8>,
        predi_data: Option<Vec<u8>>,
        asset_id: AssetId,
        target_amount: u64,
    ) -> StdResult<(Vec<SdkInput>, Bech32Address), UTXOToolsError> {

        let mut predicate = Predicate::from_code(predi_bytecode.clone())
            .with_provider(provider.clone());

        let mut pdata: Vec<u8> = Vec::new();
        if let Some(data) = predi_data {
            predicate = predicate.with_data(data.clone());
            pdata.extend_from_slice(&data);
        }
        let predicate_b32addr = predicate.address().clone();
        let filter = ResourceFilter {
            from: predicate_b32addr.clone(),
            asset_id: Some(asset_id),
            amount: target_amount,
            ..Default::default()
        };

        let utxo_predicate_hash = provider
            .get_spendable_resources(filter)
            .await
            .unwrap();

        let mut inputs = vec![];
        // let mut total_amount_in_predicate: u64 = 0;

        for coin in utxo_predicate_hash {
            inputs.push(SdkInput::resource_predicate(
                coin.clone(),
                predi_bytecode.clone(),
                pdata.clone(),
            ));
            // total_amount_in_predicate += coin.amount();
        };

        Ok((inputs, predicate_b32addr))

    }

    /// Finds a single predicate input for a specified amount of a given asset.
    ///
    /// This function attempts to locate a single coin (UTXO) that matches or exceeds
    /// the target amount for a specified asset, associated with a given predicate.
    ///
    /// # Arguments
    /// * `provider` - A reference to the Fuel provider.
    /// * `predi_bytecode` - The bytecode of the predicate.
    /// * `predi_data` - Optional data for the predicate.
    /// * `asset_id` - The AssetId of the asset to search for.
    /// * `target_amount` - The minimum amount of the asset to find.
    ///
    /// # Returns
    /// A `Result` containing:
    /// - On success: A tuple with the found input (`SdkInput`), the predicate's Bech32 address,
    ///   and the predicate's regular address.
    /// - On failure: A `UTXOToolsError`, typically `NoUTXOFound` if no matching coin is found.
    ///
    /// # Process
    /// 1. Creates a predicate from the provided bytecode and optional data.
    /// 2. Retrieves all coins for the specified asset associated with the predicate.
    /// 3. Filters coins based on the target amount.
    /// 4. Selects the oldest matching coin (lowest `block_created`).
    /// 5. Creates a predicate input from the selected coin.
    /// 6. Uses `find_coin_by_amount` to double-check and potentially optimize the selection.
    ///
    /// # Notes
    /// - This is "general" for a predicate defined by bytecode. See below for Sponsor
    /// predicate only.
    /// - This function prioritizes older coins when multiple matches are found.
    ///
    pub async fn find_single_bytecode_predicate_input_for_amount(
        provider: &Provider,
        predi_bytecode: &Vec<u8>,
        predi_data: Option<Vec<u8>>,
        asset_id: AssetId,
        target_amount: u64,
    ) -> StdResult<(SdkInput, Bech32Address, Address), UTXOToolsError> {

        let mut predicate = Predicate::from_code(predi_bytecode.clone())
            .with_provider(provider.clone());
        let mut some_predi_data: Vec<u8> = Vec::new();
        if let Some(data) = predi_data {
            predicate = predicate.with_data(data.clone());
            some_predi_data.extend_from_slice(&data);
        }
        let predicate_b32addr = predicate.address().clone();
        let predicate_address: Address = predicate.address().into();

        let predicate_coins = predicate.get_coins(asset_id).await.unwrap();
        let matching_indices = find_coins_by_amount(&predicate_coins, target_amount);

        let mut matching_coins: Vec<Coin> = matching_indices.iter()
            .map(|&index| predicate_coins[index].clone())
            .collect();

        let pick_a_coin = matching_coins.iter()
            .min_by(|a, b| a.block_created.cmp(&b.block_created))
            .cloned()
            .ok_or_else(|| UTXOToolsError::NoUTXOFound)?;

        matching_coins.retain(|coin| coin.block_created != pick_a_coin.block_created);

        match find_coin_by_amount(
            predicate.clone(),
            asset_id,
            target_amount,
            some_predi_data,
        ).await {
            Ok(input) => {
                Ok((input, predicate_b32addr, predicate_address))
            },
            Err(e) => Err(e)
        }
    }

    async fn find_coin_by_amount(
        predicate: Predicate,
        asset_id: AssetId,
        target_amount: u64,
        predi_data: Vec<u8>,
    ) -> StdResult<SdkInput, UTXOToolsError> {

        let predicate_coins = predicate.get_coins(asset_id).await.unwrap();
        let predi_bytecode = predicate.code().to_vec();

        let matching_indices: Vec<usize> = predicate_coins.iter()
            .enumerate()
            .filter(|(_, coin)| coin.amount >= target_amount)
            .map(|(index, _)| index)
            .collect();

        // Create a new vector with the matching coins
        let mut matching_coins: Vec<Coin> = matching_indices.iter()
            .map(|&index| predicate_coins[index].clone())
            .collect();
        let pick_a_coin = matching_coins.iter()
            .min_by(|a, b| a.block_created.cmp(&b.block_created))
            .cloned()
            .ok_or_else(|| UTXOToolsError::NoUTXOFound)?;
        // Remove the selected coin from matching_coins
        matching_coins.retain(|coin| coin.block_created != pick_a_coin.block_created);

        // println!("\nSelected Coin (oldest):");
        // print_coins(&vec![pick_a_coin.clone()]);

        // let mut the_predi_data: Vec<u8> = Vec::new();
        let the_inp = create_predicate_input_from_coin(
                &pick_a_coin,
                predi_bytecode.clone(),
                predi_data );

        Ok(the_inp)
    }

    pub async fn get_asset_balance_for_address(
        provider: &Provider,
        bech32addr: &Bech32Address,
        target_assetid: AssetId,
    ) -> u64 {
        println!("get balance:");
        let some_address: Address = Address::from(bech32addr);
        println!("owner: {} UTXOs:", hex::encode(some_address));

        let balances = provider.get_balances(bech32addr).await.unwrap();
        // print_balances(&balances);

        let target_key: String = target_assetid.iter().map(|byte| format!("{:02x}", byte)).collect();

        let mut target_asset_balance = 0;
        match balances.get(&target_key) {
            Some(balance) => {
                // println!("The balance for {} is {}", print_truncated_32byte_hex(hex::decode(target_key).unwrap().as_slice()), balance);
                target_asset_balance = *balance;
            },
            None => {
                println!("No balance found for {}", target_key);
            },
        }
        target_asset_balance
    }

    /// extract the utxo txid and index for the Input::Coin
    pub fn get_input_txid(some_input: &SdkInput) -> StdResult<([u8; 32], u16), UTXOToolsError> {
        let utxo_id = match some_input {
            SdkInput::ResourcePredicate { resource, .. } | SdkInput::ResourceSigned { resource } => {
                match resource {
                    CoinType::Coin(coin) => Some(&coin.utxo_id),
                    CoinType::Message(_) => None,
                }
            },
            SdkInput::Contract { utxo_id, .. } => Some(utxo_id),
        };

        if let Some(utxo_id) = utxo_id {
            let mut utxo_txid_bytes: [u8; 32] = [0x00; 32];
            utxo_txid_bytes.copy_from_slice(utxo_id.tx_id().as_ref());
            let utxo_txid_idx = utxo_id.output_index();

            // println!("UTXO ID:");
            // println!("utxo txid : {}", hex::encode(utxo_txid_bytes));
            // println!("index     : {}", utxo_txid_idx);

            Ok((utxo_txid_bytes, utxo_txid_idx))
        } else {
            Err(UTXOToolsError::NoUTXOFound)
        }
    }


    pub fn get_input_txid_and_amount(some_input: &SdkInput) -> StdResult<([u8; 32], u16, Option<u64>), UTXOToolsError> {
        let (utxo_id, amount) = match some_input {
            SdkInput::ResourcePredicate { resource, .. } | SdkInput::ResourceSigned { resource } => {
                match resource {
                    CoinType::Coin(coin) => (Some(&coin.utxo_id), Some(coin.amount)),
                    CoinType::Message(_) => (None, None),
                }
            },
            SdkInput::Contract { utxo_id, .. } => (Some(utxo_id), None),
        };

        if let Some(utxo_id) = utxo_id {
            let mut utxo_txid_bytes: [u8; 32] = [0x00; 32];
            utxo_txid_bytes.copy_from_slice(utxo_id.tx_id().as_ref());
            let utxo_txid_idx = utxo_id.output_index();

            // println!("UTXO ID:");
            // println!("utxo txid : {}", hex::encode(utxo_txid_bytes));
            // println!("index     : {}", utxo_txid_idx);
            // if let Some(amt) = amount {
            //     println!("amount    : {}", amt);
            // }

            Ok((utxo_txid_bytes, utxo_txid_idx, amount))
        } else {
            Err(UTXOToolsError::NoUTXOFound)
        }
    }





}

pub mod tools {
    use super::*;

    pub async fn get_asset_balance_for_address(
        provider: &Provider,
        bech32addr: &Bech32Address,
        target_assetid: AssetId,
    ) -> u64 {
        let balances = provider.get_balances(bech32addr).await.unwrap();
        let target_key: String = target_assetid.iter().map(|byte| format!("{:02x}", byte)).collect();
        let mut target_asset_balance = 0;
        match balances.get(&target_key) {
            Some(balance) => {
                target_asset_balance = *balance;
            },
            None => {
                println!("No balance found for {}", target_key);
            },
        }
        target_asset_balance
    }

    pub fn get_tx_id_from_script(
        tx: &ScriptTransaction,
        chainid: ChainId,
    ) -> Bytes32 {
        let tx_id = tx.id(chainid);
        tx_id
    }


}

fn create_predicate_input_from_coin(
    coin: &Coin,
    predicate_code: Vec<u8>,
    predicate_data: Vec<u8>
) -> SdkInput {
    let a_coin = coin.clone();
    let coin_type = CoinType::Coin(a_coin);
    let inp = SdkInput::resource_predicate(coin_type, predicate_code, predicate_data);
    inp
}

fn find_coins_by_amount(coins: &Vec<Coin>, target_amount: u64) -> Vec<usize> {
    coins.iter()
        .enumerate()
        .filter(|(_, coin)| coin.amount >= target_amount)
        .map(|(index, _)| index)
        .collect()
}