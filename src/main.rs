use fuels::{
    prelude::*,
    prelude::Address,
    types::{
        Bits256,
        input::Input as SdkInput,
        output::Output as SdkOutput,
    },
};

pub mod receipt_show;
use receipt_show::log_shower;

pub mod helpers;
pub mod setup_01;

pub mod io;
use io::{
    utxo_input_utils,
    tools::get_tx_id_from_script,
};

pub mod interfaces;
use interfaces::generalized_swap_verifier_interface::*;
use crate::interfaces::generalized_swap_verifier_interface;

pub mod wallets;
use wallets::*;

pub mod evm_712_sign;
use crate::receipt_show::receipt_sniffer::{
    TxStatusResult, TxPollError,
    wait_for_tx_success,
};

pub mod consts;

//--------------------------------------------------------------------------------------------------------------------------


#[tokio::main]
async fn main() {
    println!("Run swap with contract validator.");

    let _f = run_contract_validated_intent_swap().await;

}

//--------------------------------------------------------------------------------------------------------------------------

async fn run_contract_validated_intent_swap() {
    println!("run_contract_validated_intent_swap");

    // pick a known EVM wallet used for testing
    let eth_address = get_evm_addr();
    println!("EVM SENDER ADDRESS: 0x{}", hex::encode(eth_address));

    // setup assets from blank slate.
    let (
        provider,
        giov_cid,
        predicate_owner_bytecode,
        predicate_owner_b32addr,
        predicate_solver_bytecode,
        predicate_solver_b32addr,
        asset_in_id,
        asset_out_id
    ) = setup_01::setup_case_01::setup_predicate_and_assets_sdk_provider().await.unwrap();

    // get an empty fuel wallet to use in the contract instance.
    let empty_wallet = get_fuel_wallet(&provider);

    let swapverifier_instance = SwapVerifier::new(
        giov_cid.clone(),
        empty_wallet.clone()
    );

    //-------------------------------------------------------------------------

    let amount_in_1_decimal: u64 = 1_000_000_000; // 1 ETH, as U256 on Fuel, 9 decimals.
    let amount_in_2_decimal: u64 = 520_000_000; // 0.52 ETH, as U256 on Fuel, 9 decimals.

    // let amount_in_1_decimal: u64 = 600_000_000; // 1 ETH, as U256 on Fuel, 9 decimals.
    // let amount_in_2_decimal: u64 = 400_000_000; // 0.52 ETH, as U256 on Fuel, 9 decimals.



    let amount_out_decimal: u64 = 2_000_000_000; // 2x, make the price 0.50 ETH per
    // let amount_out_decimal: u64 = 1_940_000_000; // 2x, make the price 0.76 ETH per

    let amount_to_sender = amount_out_decimal;
    let amount_to_solver = amount_in_1_decimal + amount_in_2_decimal;

    let tolerance_bps: u64 = 250;


    helpers::display::print_separator_line(170, "(INPUTS):"); //-------------------------------------------------------------------------

    // get the asset_in 1st Input and UTXOid, (which will go to the Solver).
    let dwal_asset_in_bal = io::tools::get_asset_balance_for_address(
        &provider,
        &predicate_owner_b32addr,
        *AssetId::from_bytes_ref(&asset_in_id),
    ).await;
    println!("DummyPWallet asset_in balance = {}", dwal_asset_in_bal);

    /*
    println!("get DummyPWallet asset_in 1st input");
    let utxo_collector = UTXOllector::new();

    let (dwal_asset_in_1_input, _, _) = utxo_collector.find_single_bytecode_predicate_input_for_amount(
        &provider,
        &predicate_owner_bytecode.clone(),
        None,
        *AssetId::from_bytes_ref(&asset_in_id),
        amount_in_1_decimal,
    ).await.unwrap();

    let (asset_in_1_utxo_txid_bytes, utxo_1_txid_idx) = get_input_txid(&dwal_asset_in_1_input.clone()).unwrap();
    println!("Asset In 1 UTXO ID:");
    println!("utxo txid : {}", hex::encode(asset_in_1_utxo_txid_bytes));
    println!("index     : {}", utxo_1_txid_idx);

    // get the asset_in 2nd Input and UTXOid, (which will go to the Solver).
    println!("get DummyPWallet asset_in 2nd input");
    let utxo_collector = UTXOllector::new();

    let (dwal_asset_in_2_input, _, _) = utxo_collector.find_single_bytecode_predicate_input_for_amount(
        &provider,
        &predicate_owner_bytecode.clone(),
        None,
        *AssetId::from_bytes_ref(&asset_in_id),
        amount_in_2_decimal,
    ).await.unwrap();

    let (asset_in_2_utxo_txid_bytes, utxo_2_txid_idx) = get_input_txid(&dwal_asset_in_2_input.clone()).unwrap();
    println!("Asset In 2 UTXO ID:");
    println!("utxo txid : {}", hex::encode(asset_in_2_utxo_txid_bytes));
    println!("index     : {}", utxo_2_txid_idx);
    */

    println!("get DummyPWallet asset_in 1st, 2nd inputs:");

    let (dwal_asset_in_inputs, _) = utxo_input_utils::find_inputs_by_resource_filter(
        &provider,
        &predicate_owner_bytecode.clone(),
        None,
        *AssetId::from_bytes_ref(&asset_in_id),
        amount_to_solver,
    ).await.unwrap();
    let dwal_asset_in_1_input = dwal_asset_in_inputs.get(0).unwrap().to_owned();
    let dwal_asset_in_2_input = dwal_asset_in_inputs.get(1).unwrap().to_owned();

    /*
    let (asset_in_1_utxo_txid_bytes, utxo_1_txid_idx) = utxo_input_utils::get_input_txid(&dwal_asset_in_1_input.clone()).unwrap();
    println!("Asset In 1 UTXO ID:");
    println!("utxo txid : {}", hex::encode(asset_in_1_utxo_txid_bytes));
    println!("index     : {}", utxo_1_txid_idx);

    let (asset_in_2_utxo_txid_bytes, utxo_2_txid_idx) = utxo_input_utils::get_input_txid(&dwal_asset_in_2_input.clone()).unwrap();
    println!("Asset In 2 UTXO ID:");
    println!("utxo txid : {}", hex::encode(asset_in_2_utxo_txid_bytes));
    println!("index     : {}", utxo_2_txid_idx);
    */

    let (
        asset_in_1_utxo_txid_bytes,
        utxo_1_txid_idx,
        utxo_1_amount_idx
    ) = utxo_input_utils::get_input_txid_and_amount(&dwal_asset_in_1_input.clone()).unwrap();
    println!("Asset In 1 UTXO ID:");
    println!("utxo txid : {}", hex::encode(asset_in_1_utxo_txid_bytes));
    println!("amount    : {}", utxo_1_amount_idx.unwrap());
    println!("index     : {}", utxo_1_txid_idx);

    let (
        asset_in_2_utxo_txid_bytes,
        utxo_2_txid_idx,
        utxo_2_amount_idx,
    ) = utxo_input_utils::get_input_txid_and_amount(&dwal_asset_in_2_input.clone()).unwrap();
    println!("Asset In 2 UTXO ID:");
    println!("utxo txid : {}", hex::encode(asset_in_2_utxo_txid_bytes));
    println!("amount    : {}", utxo_2_amount_idx.unwrap());
    println!("index     : {}", utxo_2_txid_idx);







    // get the gas input for the solver, who pays the gas.
    println!("get DummyPSolver gas input:");
    let dsolver_baseasset_bal = io::tools::get_asset_balance_for_address(
        &provider,
        &predicate_solver_b32addr,
        AssetId::default(),
    ).await;
    println!("DummyPSolver base asset balance = {}", dsolver_baseasset_bal);

    let (dsolver_gas_input, _, _) = utxo_input_utils::find_single_bytecode_predicate_input_for_amount(
        &provider,
        &predicate_solver_bytecode.clone(),
        None,
        AssetId::default(),
        100000u64,
    ).await.unwrap();

    // get the asset_out asset id, from the the solver (which will do to the swapper pwallet)
    println!("get DummyPSolver asset_out input:");
    let dsolver_asset_out_bal = io::tools::get_asset_balance_for_address(
        &provider,
        &predicate_solver_b32addr,
        *AssetId::from_bytes_ref(&asset_out_id),
    ).await;
    println!("DummyPSolver asset_in balance = {}", dsolver_asset_out_bal);

    let (dsolver_asset_out_input, _, _) = utxo_input_utils::find_single_bytecode_predicate_input_for_amount(
        &provider,
        &predicate_solver_bytecode.clone(),
        None,
        *AssetId::from_bytes_ref(&asset_out_id),
        amount_out_decimal,
    ).await.unwrap();


    helpers::display::print_separator_line(170, "(OUTPUTS):");    //-------------------------------------------------------------------------


    // create an Output of asset id asset_out to send to the Owner (i.e., the sender receives the desired asset).
    println!("create an Output of asset id asset_out to send to DummyPWallet (the swapped asset received by sender)");
    let dwallet_swap_output = SdkOutput::Coin {
        to: Address::from(&predicate_owner_b32addr),
        amount: amount_to_sender,
        asset_id: *AssetId::from_bytes_ref(&asset_out_id),
    };

    // create an Output of asset id asset_in to send to the Solver (i.e., the solver receives the assets from the owner).
    println!("create an Output of asset id asset_in to send to DummyPSolver (the swapped asset received by solver)");
    let dsolver_swap_output = SdkOutput::Coin {
        to: Address::from(&predicate_solver_b32addr),
        amount: amount_to_solver,
        asset_id: *AssetId::from_bytes_ref(&asset_in_id),
    };

    // create an output for the change and return it to the solver.
    println!("gas change back to DummyPSolver");
    let dsolver_gas_change = SdkOutput::change(
        Address::from(&predicate_solver_b32addr),
        0,
        AssetId::default()
    );

    // create an output for the asset_in change and return it to the sender.
    println!("asset_in change back to DummyPSender");
    let dwallet_assetin_change = SdkOutput::change(
        Address::from(&predicate_owner_b32addr),
        0,
        *AssetId::from_bytes_ref(&asset_in_id),
    );


    // collect all inputs and outputs
    let tx_inputs: Vec<SdkInput> = vec![
        dwal_asset_in_1_input,
        dwal_asset_in_2_input,
        dsolver_asset_out_input,
        dsolver_gas_input,
    ];
    let tx_outputs: Vec<SdkOutput> = vec![
        dwallet_swap_output,
        dsolver_swap_output,
        dsolver_gas_change,
        dwallet_assetin_change,
    ];


    // println!("INPUTS:\n{:#?}", tx_inputs);
    // println!(" ");
    // println!("OUTPUTS:\n{:#?}", tx_outputs);


    helpers::display::print_separator_line(170, "(TX SWAP DETAILS):"); //----------------------------------------------------------------
    // This is what the sender signs. The details here must match what gets constructed into the transaction.

    let total_amount_in = amount_in_1_decimal + amount_in_2_decimal;

    // let total_amount_out = amount_out_decimal;  // copy from utxo values
    let total_amount_out: u64 = 2_000_000_000;    // create a value that is not the same as the utxo (for testing)

    let amount_in_hex_string = u64_to_bits256_hex(total_amount_in);
    println!("total_amount_in :");
    println!(" (decimal) : {}", total_amount_in);
    println!(" (hex)     : {}", amount_in_hex_string);

    let amount_out_hex_string = u64_to_bits256_hex(total_amount_out);
    println!("total_amount_out:");
    println!(" (decimal) : {}", total_amount_out);
    println!(" (hex)     : {}", amount_out_hex_string);

    println!(" ");
    println!("asset_in   : {}", hex::encode(asset_in_id));
    println!("utxo_in 1  : {}", hex::encode(asset_in_1_utxo_txid_bytes));
    println!("utxo_in 2  : {}", hex::encode(asset_in_2_utxo_txid_bytes));
    println!("asset_out  : {}", hex::encode(asset_out_id));

    println!(" ");
    println!("tolerance_bps: {}", tolerance_bps);



    helpers::display::print_separator_line(170, "(SETUP EIP-712 tx struct):"); //---------------------------------------------------------

    let (asset_1_in, amount_1_in, utxoid_1_in,
        asset_2_in, amount_2_in, utxoid_2_in
    ) = (
            helpers::conversions::bytes32_to_bits256(asset_in_id),
            Bits256::from_hex_str(&u64_to_bits256_hex(utxo_1_amount_idx.unwrap())).unwrap(),
            Bits256(asset_in_1_utxo_txid_bytes),
            helpers::conversions::bytes32_to_bits256(asset_in_id),
            Bits256::from_hex_str(&u64_to_bits256_hex(utxo_2_amount_idx.unwrap())).unwrap(),
            Bits256(asset_in_2_utxo_txid_bytes)
        );

    let asset_out = helpers::conversions::bytes32_to_bits256(asset_out_id);
    let amount_out = Bits256::from_hex_str(&amount_out_hex_string).unwrap();

    let gio_tx = populate_genio(
        [asset_1_in, asset_2_in, Bits256::zeroed(), Bits256::zeroed(), Bits256::zeroed()],
        [utxoid_1_in, utxoid_2_in, Bits256::zeroed(), Bits256::zeroed(), Bits256::zeroed()],
        [amount_1_in, amount_2_in, Bits256::zeroed(), Bits256::zeroed(), Bits256::zeroed()],
        asset_out,
        amount_out,
        helpers::conversions::u64_to_bits256(tolerance_bps),
    ).unwrap();

    // sign the GenIO tx data struct with ethers to obtain a compact signature.
    let compact_sig = evm_712_sign::build_eip712_genio_ethers::get_sig_eip712_by_ethers_genio(
        convert_to_u8_32_array(gio_tx.inputassets),
        convert_to_u8_32_array(gio_tx.inpututxoids),
        convert_to_u8_32_array(gio_tx.inputamounts),
        gio_tx.outputasset.0,
        gio_tx.outputamount.0,
        helpers::conversions::u64_to_bits256(tolerance_bps).0,
    ).await;

    assert_eq!(compact_sig.len(), 64);

    helpers::display::print_separator_line(170, "(BUILD validate_solution call tx):");

    let stb = call_validate_solution(
        swapverifier_instance.clone(),
        helpers::conversions::bytes32_to_bits256(predicate_owner_b32addr.hash),
        gio_tx,
        compact_sig,
        tx_inputs,
        tx_outputs,
    ).await;
    let tx = stb.build(&provider).await.unwrap();

    helpers::display::print_separator_line(170, "(TX):");

    let chainid = provider.chain_id();
    let tx_id = get_tx_id_from_script(&tx, chainid);

    println!("tx_id: {}", hex::encode(tx_id));
    println!(" ");
    println!("tx:\n{:#?}", tx);

    println!("send the transaction...\n");
    let tx_id = provider.send_transaction(tx).await.unwrap();
    // println!("tx_id: {}", hex::encode(tx_id));

    helpers::display::print_separator_line(170, "(SHOW RECEIPTS):");

    // get the receipts from the call:
    // let receipts = provider
    //     .tx_status(&tx_id)
    //     .await
    //     .unwrap().take_receipts();

    match wait_for_tx_success(&provider, &tx_id, 100, 12).await {
        TxStatusResult::Ok { receipts, elapsed_time } => {
            println!("Transaction successful after {:.3}s", elapsed_time.as_secs_f64());
            println!("Process receipts...");

            // println!("receipts:\n{:?}", receipts);
            // println!("");

            let dummy_empty_gio = GenIO {
                inputassets: [Bits256::zeroed(); 5],
                inpututxoids: [Bits256::zeroed(); 5],
                inputamounts: [Bits256::zeroed(); 5],
                outputasset: Bits256::zeroed(),
                outputamount: Bits256::zeroed(),
                tolerance: Bits256::zeroed(),
            };
            let dummy_swap_intent = generalized_swap_verifier_interface::Intent {
                sender: Bits256::zeroed(),
                compsig: Bytes([0x00; 64].to_vec()),
                io: dummy_empty_gio,
            };
            let fch = swapverifier_instance
                .methods()
                .validate_solution(
                    dummy_swap_intent,
                );
            let fcr = fch.get_response(receipts).unwrap();

            println!("CallResponse (bool), was the transaction valid solution?: {:?}", fcr.value);

            log_shower(fcr);
        }
        TxStatusResult::Err { error, status, elapsed_time } => {
            match error {
                TxPollError::Timeout { tx_id, duration } => {
                    println!("Transaction {:?} timed out after {:?}", tx_id, duration);
                }
                TxPollError::TransactionError(e) => {
                    println!(
                        "Transaction failed after {:?}! Error: {:?}, Last status: {:?}",
                        elapsed_time, e, status
                    );
                }
            }
        }
    }




}


fn u64_to_bits256_hex(value: u64) -> String {
    format!("0x{:064x}", value)
}


/// Converts an array of [Bits256; 5] to an array of [u8; 32]
fn convert_to_u8_32_array(input: [Bits256; 5]) -> [[u8; 32]; 5] {
    let mut result = [[0x00; 32]; 5];
    for (i, item) in input.iter().enumerate() {
        result[i] = item.0;
    }
    result
}



