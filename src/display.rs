use std::str::FromStr;
use std::collections::HashMap;
use serde_json::Value;
use tracing::{info, error};
use fuels::{
    accounts::wallet::WalletUnlocked,
    prelude::*,
    types::coin::CoinStatus,
    accounts::predicate::Predicate,
};
use fuel_types::Address;

//----------------------------

#[derive(Clone, Debug)]
pub enum FuelAddress {
    Bech32(Bech32Address),
    Raw(Address),
}

impl FuelAddress {
    pub fn to_bech32(&self) -> Bech32Address {
        match self {
            FuelAddress::Bech32(bech32) => bech32.clone(),
            FuelAddress::Raw(address) => Bech32Address::from(*address),
        }
    }

    pub fn to_raw(&self) -> Address {
        match self {
            FuelAddress::Bech32(bech32) => {
                let h = bech32.hash();
                let a = Address::from_bytes_ref(&h);
                *a
            },
            FuelAddress::Raw(address) => *address,
        }
    }
}

impl From<Bech32Address> for FuelAddress {
    fn from(bech32: Bech32Address) -> Self {
        FuelAddress::Bech32(bech32)
    }
}

impl From<Address> for FuelAddress {
    fn from(address: Address) -> Self {
        FuelAddress::Raw(address)
    }
}

//----------------------------

// use fuels_accounts::ViewOnlyAccount;



pub mod display {
    use super::*;

    pub async fn print_utxos_for_address(
        provider: &Provider,
        bech32addr: &Bech32Address
    ) {
        println!("get utxos for address:");

        let some_address: Address = Address::from(bech32addr);

        println!("\naddress {} UTXOs:", hex::encode(some_address));

        let balances = provider.get_balances(bech32addr).await.unwrap();
        print_balances(&balances);

        // println!("--------------------------------");
        for (key, value) in balances.iter() {
            println!("Asset(key): {},\tTotalValue: {}", key, value);
            // print!("\n");
            println!("Index:\tUTXO_txid (tx_id):\t\t\t\t\t\t\tAmount:\t\tStatus:");



            let ass_id = AssetId::from_str(key).unwrap();

            let x_max_utxos: usize = 25;
            for i in 0..x_max_utxos {
                let coins = provider.get_coins(&bech32addr, ass_id).await.unwrap();
                let c_len = coins.len();
                // println!("coins length = {}", c_len);

                if i < coins.len() {
                    //index:
                    print!("{}", i);
                    let blah_coin = &coins[i];

                    let c_utxo_id = blah_coin.utxo_id;
                    // Serialize UtxoId to JSON
                    let json_str = serde_json::to_string(&c_utxo_id).unwrap();
                    //println!("Serialized UtxoId: {}", json_str);

                    // Parse JSON string into a serde_json::Value
                    let json_value: Value = serde_json::from_str(&json_str).unwrap();

                    // Access the tx_id field from the JSON value
                    let tx_id_from_json: Option<String> = serde_json::from_value(json_value["tx_id"].clone()).unwrap();
                    //.map(|bytes: Bytes32| hex::encode(&bytes.bytes()));

                    // tx_id from within UtxoId { tx_id: ---, output_index: --- }
                    match tx_id_from_json {
                        Some(tx_id) => print!("\t{}", tx_id),
                        None => println!("Failed to extract tx_id"),
                    }

                    let c_amount = blah_coin.amount;
                    print!("\t{}", c_amount);

                    let c_status = blah_coin.status.clone();
                    match c_status {
                        CoinStatus::Unspent => print!("\t\tUnspent"),
                        CoinStatus::Spent => print!("\t\tSpent"),
                    }

                    // println!("\n---");
                    // println!("blah_coin[{}]: {:?}", i, blah_coin);

                    print!("\n");
                } else {
                    // println!("Index {} is out of bounds for x_max_utxos ({}).", i, x_max_utxos);
                    print!("\n");
                    break; // Exit the loop if there are less than x Coins for particular AssetId owned.
                }
            }

        }

        println!("\nAddress balances:");
        let balances = provider.get_balances(bech32addr).await.unwrap();
        println!("{:#?}\n", balances);
    }

/*
    /// Take predicate Bech32Address or Address & print out (max 25) UTXO's at the predicate address.
    pub async fn print_utxos_for_bech32address(
        provider: &Provider,
        // script_bech32address: Bech32Address,
        // script_address: Address,
        fuel_address: FuelAddress,
    ) {
        info!("--------------------------------");
        info!("get utxos for script Bech32Address:");
        // let predicate = Predicate::from_code(this_predicate_bytecode)
        //     .with_provider(provider.clone());
        // let predicate_address: Address = predicate.address().into();
        // let predicate_address_bech32 = Bech32Address::from(predicate_address);

        // let h = script_bech32address.hash();
        // let script_address = Address::from_bytes_ref(&h);

        /*
        let bech32_address = match fuel_address {
            FuelAddress::Bech32(bech32) => {
                println!("Bech32 address provided directly");
                bech32
            },
            FuelAddress::Raw(raw) => {
                println!("Raw address provided, converting to Bech32");
                Bech32Address::from(raw)
            }
        };
        */


        info!("predicate_address (hex)   : {}", hex::encode(predicate_address));
        info!("predicate_address (bech32): {:?}", predicate_address_bech32.to_string());
        info!("Script/Predicate balances:");
        let balances = predicate.get_balances().await.unwrap();

        let g = bech32_address.


        for (key, value) in balances.iter() {
            info!("Asset(key): {},\tTotalValue: {}", key, value);
            info!("Index:\tUTXO_txid (tx_id):\t\t\t\t\t\t\tAmount:\t\tStatus:");
            info!(" ");
            let ass_id = AssetId::from_str(key).unwrap();
            let x_max_utxos: usize = 25;

            for i in 0..x_max_utxos {
                let coins = provider.get_coins(&predicate_address_bech32, ass_id).await.unwrap();
                let _c_len = coins.len();
                if i < coins.len() {
                    let some_coin = &coins[i];
                    let c_utxo_id = some_coin.utxo_id;
                    let json_str = serde_json::to_string(&c_utxo_id).unwrap();
                    let json_value: Value = serde_json::from_str(&json_str).unwrap();
                    let tx_id_from_json: Option<String> = serde_json::from_value(json_value["tx_id"].clone()).unwrap();
                    let tx_id_info = match tx_id_from_json {
                        Some(tx_id) => format!("{}", tx_id),
                        None => "Failed to extract tx_id".to_string(),
                    };
                    let c_amount = some_coin.amount;
                    let status_info = match some_coin.status {
                        CoinStatus::Unspent => "Unspent",
                        CoinStatus::Spent => "Spent",
                    };
                    info!("{}\t {}\t{}\t\t{}", i, tx_id_info, c_amount, status_info);
                } else {
                    info!(" ");
                    break;
                }
            }


        }
        info!(" ");
        let balances = predicate.get_balances().await.unwrap();
        if balances.is_empty() {
            info!("No balances found.");
        } else {
            info!("Predicate Balances:");
            for (asset_id, balance) in balances.iter() {
                info!("  AssetId: {}", asset_id);
                info!("  Balance: {} ", balance);
                info!(""); // Empty line for readability between entries
            }
        }
        info!("--------------------------------");
    }
*/
    /// Take predicate bytecode & print out (max 25) UTXO's at the predicate address.
    pub async fn print_utxos_for_script_bytecode(
        provider: &Provider,
        this_predicate_bytecode: Vec<u8>
    ) {
        info!("--------------------------------");
        info!("get utxos for address:");
        let predicate = Predicate::from_code(this_predicate_bytecode)
            .with_provider(provider.clone());
        let predicate_address: Address = predicate.address().into();
        let predicate_address_bech32 = Bech32Address::from(predicate_address);
        info!("predicate_address (hex)   : {}", hex::encode(predicate_address));
        info!("predicate_address (bech32): {:?}", predicate_address_bech32.to_string());
        info!("Script/Predicate balances:");
        let balances = predicate.get_balances().await.unwrap();
        for (key, value) in balances.iter() {
            info!("Asset(key): {},\tTotalValue: {}", key, value);
            info!("Index:\tUTXO_txid (tx_id):\t\t\t\t\t\t\tAmount:\t\tStatus:");
            info!(" ");
            let ass_id = AssetId::from_str(key).unwrap();
            let x_max_utxos: usize = 25;

            for i in 0..x_max_utxos {
                let coins = provider.get_coins(&predicate_address_bech32, ass_id).await.unwrap();
                let _c_len = coins.len();
                if i < coins.len() {
                    let some_coin = &coins[i];
                    let c_utxo_id = some_coin.utxo_id;
                    let json_str = serde_json::to_string(&c_utxo_id).unwrap();
                    let json_value: Value = serde_json::from_str(&json_str).unwrap();
                    let tx_id_from_json: Option<String> = serde_json::from_value(json_value["tx_id"].clone()).unwrap();
                    let tx_id_info = match tx_id_from_json {
                        Some(tx_id) => format!("{}", tx_id),
                        None => "Failed to extract tx_id".to_string(),
                    };
                    let c_amount = some_coin.amount;
                    let status_info = match some_coin.status {
                        CoinStatus::Unspent => "Unspent",
                        CoinStatus::Spent => "Spent",
                    };
                    info!("{}\t {}\t{}\t\t{}", i, tx_id_info, c_amount, status_info);
                } else {
                    info!(" ");
                    break;
                }
            }


        }
        info!(" ");
        let balances = predicate.get_balances().await.unwrap();
        if balances.is_empty() {
            info!("No balances found.");
        } else {
            info!("Predicate Balances:");
            for (asset_id, balance) in balances.iter() {
                info!("  AssetId: {}", asset_id);
                info!("  Balance: {} ", balance);
                info!(""); // Empty line for readability between entries
            }
        }
        info!("--------------------------------");
    }


    pub async fn print_utxos_for_wallet_address(
        provider: &Provider,
        wallet: &WalletUnlocked
    ) {
        info!("--------------------------------");
        info!("get utxos for address:");
        let wallet_address: Address = Address::from(wallet.address());
        let wallet_address_bech32 = Bech32Address::from(wallet_address);
        info!("wallet address (hex)   : {}", hex::encode(wallet_address));
        info!("wallet address (bech32): {:?}", wallet_address_bech32);
        info!("Wallet balances:");
        let balances = wallet.get_balances().await.unwrap();
        for (key, value) in balances.iter() {
            info!("Asset(key): {},\tTotalValue: {}", key, value);
            info!("Index:\tUTXO_txid (tx_id):\t\t\t\t\t\t\tAmount:\t\tStatus:");
            let ass_id = AssetId::from_str(key).unwrap();
            let x_max_utxos: usize = 25;
            for i in 0..x_max_utxos {
                let coins = provider.get_coins(&wallet_address_bech32, ass_id).await.unwrap();
                let _c_len = coins.len();

                for i in 0..x_max_utxos {
                    let coins = provider.get_coins(&wallet_address_bech32, ass_id).await.unwrap();
                    let _c_len = coins.len();
                    if i < coins.len() {
                        let some_coin = &coins[i];
                        let c_utxo_id = some_coin.utxo_id;
                        let json_str = serde_json::to_string(&c_utxo_id).unwrap();
                        let json_value: Value = serde_json::from_str(&json_str).unwrap();
                        let tx_id_from_json: Option<String> = serde_json::from_value(json_value["tx_id"].clone()).unwrap();
                        let tx_id_info = match tx_id_from_json {
                            Some(tx_id) => format!("{}", tx_id),
                            None => "Failed to extract tx_id".to_string(),
                        };
                        let c_amount = some_coin.amount;
                        let status_info = match some_coin.status {
                            CoinStatus::Unspent => "Unspent",
                            CoinStatus::Spent => "Spent",
                        };
                        info!("{}\t {}\t{}\t\t{}", i, tx_id_info, c_amount, status_info);
                    } else {
                        info!(" ");
                        break;
                    }
                }
            }
        }
        info!(" ");
        let balances = wallet.get_balances().await.unwrap();
        if balances.is_empty() {
            info!("No balances found.");
        } else {
            info!("Predicate Balances:");
            for (asset_id, balance) in balances.iter() {
                info!("  AssetId: {}", asset_id);
                info!("  Balance: {} ", balance);
                info!(""); // Empty line for readability between entries
            }
        }
        info!("--------------------------------");
    }




}



pub fn print_balances(balances: &HashMap<String, u64>) {
    println!("Asset: \t\t\tBalance:");
    for (key, value) in balances.iter() {
        println!("{} \t{}", print_truncated_32byte_hex(hex::decode(key).unwrap().as_slice()), value);
    }
}

pub fn print_truncated_32byte_hex(hex32bytes: &[u8]) -> String {
    let hex_string = hex::encode(hex32bytes);
    let truncated = if hex_string.len() > 12 {
        format!("{}...{}", &hex_string[..6], &hex_string[hex_string.len() - 6..])
    } else {
        hex_string
    };
    truncated
}
