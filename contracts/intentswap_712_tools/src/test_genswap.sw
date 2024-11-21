library;

use std::{
    b512::B512,
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
        extend,
        bytes_read_b256,
    },
    numeric_utils::*,
};

use ::generalized_swap::{
    GenIO, Intent, EIP712Domain, Eip712,
};


//-----------------------------------------------------------
// DEBUG:
use helpers::hex::b256_to_hex;
//-----------------------------------------------------------


const TEST_CONST_DOMAIN_SEP_HASH: b256 = 0x47f9d229f5cbfdd9148072eb4928dc4f52479d3ea9d4e578743f6c51b959f445;
const TEST_CONST_TYPE_HASH: b256 = 0x2aba27466153a0e63c25c715bd2cd4807b7f9c8ca6938eb1c37f849c9c6c9fa9;
const TEST_CONST_STRUCT_HASH: b256 = 0xf265f6e2e330157d3a26de8f4fc1bb89065890cea9a1e868ed0ffa0b0f085df4;
const TEST_CONST_ENCODED_HASH: b256 = 0xa73aaefb000a09cb89eecc08a6d17cc69a6741d6a8996aff2c48d4475030b2a1;
const TEST_CONST_EVM_SINGER: b256 = 0x000000000000000000000000333339d42a89028ee29a9e9f4822e651bac7ba14;


// forc test domain_hash --logs
// test the calculation of domain_hash
#[test]
fn domain_hash(){
    // test domain_hash ... ok (55.699888ms, 39234 gas)
    let eip712_domain_type_hash = EIP712Domain::new().domain_separator_hash();
    log(eip712_domain_type_hash);
    let expected_domain_hash = TEST_CONST_DOMAIN_SEP_HASH;
    assert(eip712_domain_type_hash == expected_domain_hash );
    /*
        8b73c3c69bb8fe3d512ecc4cf759cc79239f7b179b0ffacaa9a75d522b39400f --> EIP712_DOMAIN_TYPE_HASH
        b5071facaa60230ac769f911446fb1de75bd4cd98c66efdd0825a0dc7af3d3dc --> Name Hash
        c89efdaa54c0f20c7adf612882df0950f5a951637e0307cdcb4c672f298b8bc6 --> Version Hash
        00000000000000000000000000000000000000000000000000000000000026a1 --> Chain ID
        0000000000000000000000000000000000000000000000000000000000000001 --> Verifying Contract
        47f9d229f5cbfdd9148072eb4928dc4f52479d3ea9d4e578743f6c51b959f445 --> final hash
    */
}

// forc test domain_hash_precalc --logs
#[test]
fn domain_hash_precalc(){
    // test domain_hash_precalc ... ok (1.544836ms, 883 gas)
    let eip712_domain_type_hash = EIP712Domain::new().domin_separator_hash_precalc();
    log(eip712_domain_type_hash);
    let expected_domain_hash = TEST_CONST_DOMAIN_SEP_HASH;
    assert(eip712_domain_type_hash == expected_domain_hash );
}

// forc test type_precalc --logs
#[test]
fn type_precalc(){
    // test domain_hash_precalc ... ok (1.544836ms, 883 gas)
    let eip712_type_hash = GenIO::type_hash();
    log(eip712_type_hash);
    let expected_type_hash = TEST_CONST_TYPE_HASH;
    assert(eip712_type_hash == expected_type_hash );
}

// forc test struct_hash --logs
// test tx struct hash with params.
#[test]
fn struct_hash(){

    let (some_asset_in, some_amount_in, utxoid_in, asset_out, amount_out, tolerance) = get_setup_tx_params();
    let tx = GenIO::new(
        [some_asset_in, some_asset_in, some_asset_in, some_asset_in, some_asset_in],
        [utxoid_in, utxoid_in, utxoid_in, utxoid_in, utxoid_in],
        [some_amount_in, some_amount_in, some_amount_in, some_amount_in, some_amount_in],
        asset_out,
        amount_out,
        tolerance,
    );

    let struct_hash = tx.struct_hash();
    // encoded struct: 2aba27466153a0e63c25c715bd2cd4807b7f9c8ca6938eb1c37f849c9c6c9fa9dfded4ed5ac76ba7379cfe7b3b0f53e768dca8d45a34854e649cfc3c18cbd9cda15d0f6fa074184fb0be73a3e2bda1b9463822845a24ab337720ad45fcf83e62fb660c034c9014d94d76f73c3a59458efd09c6bccfc3e6496bd9d98794dce3b50202020202020202020202020202020202020202020202020202020202020202000000000000000000000000000000000000000000000000000000007735940000000000000000000000000000000000000000000000000000000000000000fa

    log(struct_hash);
    let expected_struct_hash = TEST_CONST_STRUCT_HASH;
    assert(struct_hash == expected_struct_hash );
}

// forc test hash_encode712 --logs
// test the encoding and hashing domain and sruct according to EIP-712 spec.
#[test]
fn hash_encode712(){

    let (some_asset_in, some_amount_in, utxoid_in, asset_out, amount_out, tolerance) = get_setup_tx_params();
    let payload = (
        EIP712Domain::new(),
        GenIO::new(
            [some_asset_in, some_asset_in, some_asset_in, some_asset_in, some_asset_in],
            [utxoid_in, utxoid_in, utxoid_in, utxoid_in, utxoid_in],
            [some_amount_in, some_amount_in, some_amount_in, some_amount_in, some_amount_in],
            asset_out,
            amount_out,
            tolerance
        )
    );
    let encoded_hash = match payload.encode_eip712() {
        Some(hash) => hash,
        None => revert(0),
    };
    log(encoded_hash);

    let expected_encoded_hash = TEST_CONST_ENCODED_HASH;
    assert(encoded_hash == expected_encoded_hash);
}

// forc test recover_signer_from_712tx --logs
// test recovery from a mock transaction input of amount, receiver, compact signature (added as witness).
#[test]
fn recover_signer_from_712tx(){
    // receive as params:
    // amount, receiver, compact signature.
    let (some_asset_in, some_amount_in, utxoid_in, asset_out, amount_out, tolerance) = get_setup_tx_params();
    let mut compactsig_hex_string = String::from_ascii_str("13caf5acee7537d7c791c5854d4011257952d96d47f52f397ee9b90576e35449294c0a9c53fbeccf7319ef4ed1470d2dfddf9b803cb9a67e885e5a3851cfd9e0");

    let compactsig_bytes = hex_string_to_bytes(compactsig_hex_string).unwrap();
    let mut ptr: u64 = 0;
    let (cs_lhs, ptr) = bytes_read_b256(compactsig_bytes, ptr, 32);
    let (cs_rhs, _ptr) = bytes_read_b256(compactsig_bytes, ptr, 32);

    log(cs_lhs);
    log(cs_rhs);

    // let cs_lhs_cs_rhs = (cs_lhs, cs_rhs);
    let compactsig = B512::from((cs_lhs, cs_rhs));

    let payload = (
        EIP712Domain::new(),
        GenIO::new(
            [some_asset_in, some_asset_in, some_asset_in, some_asset_in, some_asset_in],
            [utxoid_in, utxoid_in, utxoid_in, utxoid_in, utxoid_in],
            [some_amount_in, some_amount_in, some_amount_in, some_amount_in, some_amount_in],
            asset_out,
            amount_out,
            tolerance,
        )
    );
    let encoded_hash = match payload.encode_eip712() {
        Some(hash) => hash,
        None => revert(0),
    };

    let recovered_signer: b256 = ec_recover_evm_address(compactsig, encoded_hash).unwrap().into();
    log(recovered_signer);

    let expected_signer = TEST_CONST_EVM_SINGER;
    assert(recovered_signer == expected_signer);
}

fn get_setup_tx_params() -> (b256, b256, b256, b256, b256, b256) {
    let asset_in: b256 = 0x0000000000000000000000000000000000000000000000000000000000000000;
    let utxoid_in: b256 = 0x0101010101010101010101010101010101010101010101010101010101010101;
    let amount_in: b256 = 0x000000000000000000000000000000000000000000000000000000003b9aca00; // 1 ETH, as U256 on Fuel, 9 decimals.
    let asset_out: b256 = 0x0202020202020202020202020202020202020202020202020202020202020202;
    let amount_out: b256 = 0x0000000000000000000000000000000000000000000000000000000077359400;  // 2x, make the price 0.5 ETH per
    let tolerance_bps: u64 = 250;  // 2.5% tolerance, convert to b256 big-endian
    (asset_in, amount_in, utxoid_in, asset_out, amount_out, to_b256((0, 0, 0, tolerance_bps)))
}




// forc test struct_hash_blah1 --logs
// test tx struct hash with params.
#[test]
fn struct_hash_blah1(){

    let sender: b256 = 0x1937bc8f1733a188a5cf63b93f1fa3bc229b6f9fad49c7d3514f2eb91e593966;

    let asset1: b256 = 0x648318bcc430e79e7e9f0d2087a00353912505ac8beb18661b8a1e6907f800a9;
    let asset2: b256 = 0x7701ab82f40441a90b39eaebdade54a4464f9c070678766bd2eda3f89e44a7c6;
    let asset3: b256 = 0x0000000000000000000000000000000000000000000000000000000000000000;

    // test UTXO id's
    let utxo1: b256 = 0x585a36b425eb9afcfadce25944486ac80a6ed5090aa1e76ca4176b3a8f230956;
    let utxo2: b256 = 0xff22400efac198dfa4a4a29d816a089d1f18c166663077a06ed7317b6c044551;
    let utxo3: b256 = 0x3333333333333333333333333333333333333333333333333333333333333333;
    let utxo4: b256 = 0x4444444444444444444444444444444444444444444444444444444444444444;
    let utxo5: b256 = 0x5555555555555555555555555555555555555555555555555555555555555555;


    let intent_input_assets: [b256; 5] = [
        asset1, asset1, b256::zero(), b256::zero(), b256::zero(),
    ];
    // let intent_input_amounts: [b256; 5] = [
    //     to_b256((0, 0, 0, 520000000)),     // for asset1
    //     to_b256((0, 0, 0, 1000000000)),    // for asset1
    //     to_b256((0, 0, 0, 0)),
    //     b256::zero(), b256::zero()
    // ];
    let intent_input_amounts: [b256; 5] = [
        to_b256((0, 0, 0, 1000000000)),    // for asset1
        to_b256((0, 0, 0, 520000000)),     // for asset1
        to_b256((0, 0, 0, 0)),
        b256::zero(), b256::zero()
    ];
    let intent_utxos = [utxo1, utxo2, b256::zero(), b256::zero(), b256::zero()];

    //
    let intent_output_asset = asset2;
    let intent_output_amount: b256 = to_b256((0, 0, 0, 2000000000));    // 2000000000 for asset2

    let tolerance_bps: b256 = to_b256((0, 0, 0, 250));  // 2.5% tolerance







    let tx = GenIO::new(
        intent_input_assets,
        intent_utxos,
        intent_input_amounts,
        intent_output_asset,
        intent_output_amount,
        tolerance_bps,
    );

    let struct_hash = tx.struct_hash();


    log(String::from_ascii_str("Struct hash:"));
    log(b256_to_hex(struct_hash));




    // let expected_struct_hash = TEST_CONST_STRUCT_HASH;
    // assert(struct_hash == expected_struct_hash );
}

