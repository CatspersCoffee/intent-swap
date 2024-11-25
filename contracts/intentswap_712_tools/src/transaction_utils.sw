library;

use std::{
    inputs::{
        GTF_INPUT_COIN_AMOUNT,
        GTF_INPUT_COIN_ASSET_ID,
        input_type,
        Input,
    },
    outputs::{
        GTF_OUTPUT_TYPE,
        GTF_OUTPUT_COIN_AMOUNT,
        GTF_OUTPUT_COIN_ASSET_ID,
        GTF_OUTPUT_COIN_TO,
        output_type,
        Output,
    },
    tx::{
        GTF_SCRIPT_GAS_LIMIT,
        GTF_POLICY_TYPES,
        GTF_POLICY_TIP,
        GTF_SCRIPT_INPUTS_COUNT,
        GTF_SCRIPT_OUTPUTS_COUNT,
        GTF_CREATE_OUTPUT_AT_INDEX,
        GTF_SCRIPT_OUTPUT_AT_INDEX,
        Transaction,
        tx_type,
    },
};

// TODO: replace GTF consts with direct references to tx.sw, inputs.sw, and outputs.sw from std lib
const GTF_INPUT_TYPE = 0x200;   // sway 0.63.1

const GTF_INPUT_COIN_TX_ID = 0x201;

const OUTPUT_TYPE_COIN = 0u64;
const OUTPUT_TYPE_CHANGE = 3u64;

const INPUT_TYPE_COIN = 0u64;
const INPUT_TYPE_CONTRACT = 1u64;


fn policies() -> u32 {
    __gtf::<u32>(0, GTF_POLICY_TYPES)
}

// changed to policy_tip in sway 0.60.0
const TIP_POLICY: u32 = 1u32 << 0;

pub fn tx_tip() -> Option<u64> {
    let bits = policies();
    if bits & TIP_POLICY > 0 {
        Some(__gtf::<u64>(0, GTF_POLICY_TIP))
    } else {
        None
    }
}

/// Get the transaction gas price
pub fn tx_gas_limit() -> u64 {
    __gtf::<u64>(0, GTF_SCRIPT_GAS_LIMIT)
}

/// Get the transaction inputs count
pub fn input_count() -> u64 {
    __gtf::<u64>(0, GTF_SCRIPT_INPUTS_COUNT)
}

/// Verifies an input at the given index is a Coin input
pub fn verify_input_coin(index: u64) -> bool {
    match input_type(index) {
        Some(Input::Coin) => return true,
        _ => false,
    }
}
/// Verifies an input at the given index is a Contract input
pub fn verify_input_contract(index: u64) -> bool {
    match input_type(index) {
        Some(Input::Contract) => true,
        _ => false,
    }
}

/// Get the asset ID of a coin input
pub fn input_coin_asset_id(index: u64) -> b256 {
    __gtf::<b256>(index, GTF_INPUT_COIN_ASSET_ID)
}


/// Get the amount of a coin input
pub fn input_coin_amount(index: u64) -> u64 {
    __gtf::<u64>(index, GTF_INPUT_COIN_AMOUNT)
}

/// Get the transaction outputs count
pub fn output_count() -> u64 {
    __gtf::<u64>(0, GTF_SCRIPT_OUTPUTS_COUNT)
}

/// Verifies an output at the given index is a coin output
pub fn verify_output_coin(index: u64) -> bool {
    match output_type(index) {
        Some(Output::Coin) => return true,
        _ => false,
    }
}

pub fn verify_output_change(index: u64) -> Option<bool> {
    if index >= output_count() {
        return None
    }

    match __gtf::<u8>(index, GTF_OUTPUT_TYPE) {
        0u8 => Some(false),
        1u8 => Some(false),
        2u8 => Some(true),
        3u8 => Some(false),
        4u8 => Some(false),
        _ => None,
    }
}

pub fn output_coin_asset_id(index: u64) -> Option<b256> {
    match output_type(index) {
        Some(Output::Coin) => Some(__gtf::<b256>(index, GTF_OUTPUT_COIN_ASSET_ID)),
        _ => None,
    }
}

/// Get the amount of a coin input
pub fn output_coin_amount(index: u64) -> u64 {
    __gtf::<u64>(index, GTF_OUTPUT_COIN_AMOUNT)
}

/// Get the receiver of a coin input
pub fn output_coin_to(index: u64) -> b256 {
    __gtf::<b256>(index, GTF_OUTPUT_COIN_TO)
}

/// return the utxo id from the coin input at index.
pub fn input_txn_hash(index: u64) -> b256 {
     __gtf::<b256>(index, GTF_INPUT_COIN_TX_ID)
}