predicate;

use std::{
    b512::B512,
    tx::{
        tx_id,
        tx_witness_data,
    },
    inputs::{
        input_predicate,
        input_count,
        input_coin_owner,
    },
    outputs::{
        output_type,
        output_asset_id,
        output_asset_to,
        output_amount,
        output_count,
    },
    vm::evm::{
        ecr::ec_recover_evm_address,
        evm_address::EvmAddress,
    },
    bytes::Bytes,
    math::*,
    option::Option,
    string::String,
};
use std::*;
use std::bytes_conversions::{b256::*, u64::*};
use std::primitive_conversions::{u16::*, u32::*, u64::*};

use helpers::{
    general_helpers::{
        hex_string_to_bytes,
        string_to_bytes,
        bytes_read_b256,
    },
    hex::b256_to_hex,
    numeric_utils::*,
};
use intentswap_712_tools::{
    generalized_swap::{
        GenIO, Intent, EIP712Domain, Eip712,
    },
    transaction_utils::{
        input_coin_amount,
        input_coin_asset_id,
        output_coin_asset_id,
        verify_input_coin,
        verify_output_coin,
        verify_output_change,
        input_txn_hash,
    },
    io_utils::{
        InpOut,
        process_assets,
        check_utxos,
        find_input_assets_owner,
        verify_change_output,
        reconstruct_intent_lite, ReconstructIntentResult,
    },
};

const TEST_CONST_EVM_SINGER: b256 = 0x000000000000000000000000222227f7e08997ee7457a0a772e417ca5462906d;

//FIXME - Untested.
fn main( intent: Intent ) -> bool {

    let in_count: u64 = input_count().into();
    let out_count: u64 = output_count().as_u64();
    let mut tx_inputs : Vec<InpOut> = Vec::new();
    let mut tx_outputs : Vec<InpOut> = Vec::new();
    let mut tx_change_assetid: Vec<b256> = Vec::new();
    let mut tx_change_to: Vec<b256> = Vec::new();

    let mut change_ok = false;
    let mut sender = b256::zero();
    let mut recovered_signer = b256::zero();
    let mut signed_by_sender: bool = false;

    let mut i = 0;
    while i < in_count {
        // collect all the input coins
        if verify_input_coin(i) {
            let inp = InpOut::new(
                input_coin_asset_id(i),
                Some(input_coin_amount(i)),
                None,
                Some(input_txn_hash(i)),
                input_coin_owner(i)
            );
            tx_inputs.push(inp);

        }
        i += 1;
    }
    let mut j = 0;
    while j < out_count {
        // collect all the output coins
        if verify_output_coin(j) {
            let outp = InpOut::new(
                output_coin_asset_id(j).unwrap(),   // from tx_utls, return Option<b256>
                Some(output_amount(j).unwrap()),
                None,
                None,
                None
            );
            tx_outputs.push(outp);
        }

        // collect all the change outputs assetid's and receivers.
        match verify_output_change(j) {
            Some(is_change) => {
                if is_change {
                    tx_change_assetid.push(output_asset_id(j).unwrap().into());
                    tx_change_to.push(output_asset_to(j).unwrap().into());
                }
            },
            _ => {},
        }


        j += 1;
    }

    // copy tolerance from tx intent data:
    let tolerance_bps: u64 = b256_to_u64(intent.io.tolerance);

    //reorder the inputs to match the expected utxo set:
    // reorder_inputs(tx_inputs, intent.io.inpututxoids);

    let input_result = match process_assets(
        tx_inputs,
        intent.io.inputassets,      // intent input assets
        intent.io.inputamounts,     // intent input amounts
        intent.io.outputasset,      // intent output assets
        intent.io.outputamount,     // intent output amounts
        tolerance_bps,              // tollerance bps as a u64
        true                        // is inputs?
    ) {
        Ok(result) => {
            result
        },
        Err(error_code) => {
            // revert(error_code);
            return false;
        },
    };
    let output_result = match process_assets(
        tx_outputs,
        intent.io.inputassets,      // intent input assets
        intent.io.inputamounts,     // intent input amounts
        intent.io.outputasset,      // intent output assets
        intent.io.outputamount,     // intent output amounts
        tolerance_bps,              // tollerance bps as a u64
        false
    ) {
        Ok(result) => {
            result
        },
        Err(error_code) => {
            // Output processing failed with error code
            // revert(error_code);
            return false;
        },
    };
    // check utxos are ok
    let (utxo_check_result, ordered_utxos, utxo_indices) = check_utxos(tx_inputs, intent.io.inpututxoids);

    // check there is a change output of the asset_in to the sender
    //
    // find the tx_input that matches the assetid of input_result.match_asset, record the idx (k)
    // as you iterate through tx_inputs.
    // query the tx inputs at idx (k), and find the owner.
    //
    match find_input_assets_owner(tx_inputs, input_result.match_asset) {
        Some(owner) => {
            sender = owner.into(); //returned as Option<Address>

            // see Note 1 below.
            /*
            change_ok = verify_change_output( // use v4
                tx_change_assetid,
                tx_change_to,
                input_result.match_asset,
                sender,
            );
            */
        },
        None => {
            return false;
        },
    }

    //-------------------------------------------

    if (input_result.amounts_match &&
        output_result.amounts_match &&
        utxo_check_result &&
        change_ok
        ) {

        // Note 1:
        // The predicate validator has run into max bytecode length issues, so for
        // demonstraction we are simply reconstructing the intent from mostly the
        // data passed in using reconstruct_intent_lite(). We only use the tx_inputs
        // as tx specific data here. See the contract validator and reconstruct_intent()
        // for true reconstruction mechanism, that takes all (* see Note 2 below) the data obtained from
        // transaction introspection.
        //
        match reconstruct_intent_lite(
            tx_inputs,
            input_result.agg_assets,
            intent.io.inputamounts,
            intent.io.outputasset,
            intent.io.outputamount,
            ordered_utxos,
            utxo_indices,

        ) {
            ReconstructIntentResult::Success(recon_intent) => {
                // Note 2:
                // Use the reconstructed intent data to rebuild the struct.
                // Copy the tolerance directly from sent in intent data, if this
                // value was sent in with a value that was not the same as the sender
                // set, then ecr will fail anyway.
                let payload = (
                    EIP712Domain::new(),
                    GenIO::new(
                        recon_intent.input_assets,
                        recon_intent.input_utxos,
                        recon_intent.input_amounts,
                        recon_intent.output_asset,
                        recon_intent.output_amount,
                        intent.io.tolerance,
                    )
                );
                let encoded_hash = match payload.encode_eip712() {
                    Some(hash) => hash,
                    None => { return false; },
                };

                let mut ptr: u64 = 0;
                let (cs_lhs, ptr) = bytes_read_b256(intent.compsig, ptr, 32);
                let (cs_rhs, ptr) = bytes_read_b256(intent.compsig, ptr, 32);
                let compactsig = B512::from((cs_lhs, cs_rhs));

                recovered_signer = ec_recover_evm_address(
                    compactsig, encoded_hash
                ).unwrap().into();
            },
            _ => {
                // Failed to reconstruct intent
            }
        }

    } else {
        // input or outputs or utxos are not correct
        // revert code for input failure
        // revert code for output failure
        // revert code for utxo failure
        // revert code for change failure
        return false;
    }


    if (recovered_signer == TEST_CONST_EVM_SINGER) {
        signed_by_sender = true;
    }

    return signed_by_sender;
}