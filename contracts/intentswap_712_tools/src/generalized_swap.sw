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
        hash_bytes,
        bytes_read_b256,
    },
    numeric_utils::to_b256,
};


use helpers::hex::b256_to_hex;

// A Simple Swap based Intent, validated through EIP-712
//
// EIP-712 Typed Data
//   |
//   |------------------- EIP-712 prefix ----------------->|
//   |                                                     |
//   |----> Domain Separator                               |
//   |        |                                            |
//   |        |----> Domain Hash ------------------------->|
//   |                                                     |
//   |                                                     |
//   |        |----> Type Hash ---->|                      |
//   |        |                     |                      |
//   |------->|                     +----> Struct Hash --->|
//            |                     |                      |
//            |---- Struct Data --->|                      |
//                                                         |
//                                                         |
//          Encode EIP-712 <-------------------------------+
//                |
//                |
//                v
//          Sign keccak256(Encode EIP-712).
//


const EIP712_DOMAIN_TYPE_HASH: b256 = 0x8b73c3c69bb8fe3d512ecc4cf759cc79239f7b179b0ffacaa9a75d522b39400f;


pub struct Intent {
    pub sender: b256,
    pub compsig: Bytes,
    pub io: GenIO,
}


pub struct GenIO {
    pub inputassets: [b256; 5],
    pub inpututxoids: [b256; 5],
    pub inputamounts: [b256; 5],
    pub outputasset: b256,
    pub outputamount: b256,
    pub tolerance: b256,
}

impl GenIO {

    pub fn new(
        assets_in: [b256; 5],
        utxoids_in: [b256; 5],
        amounts_in: [b256; 5],
        asset_out: b256,
        amount_out: b256,
        tolerance_bps: b256,
    ) -> GenIO {
        GenIO {
            inputassets: [assets_in[0], assets_in[1], assets_in[2], assets_in[3], assets_in[4]],
            inpututxoids: [utxoids_in[0], utxoids_in[1], utxoids_in[2], utxoids_in[3], utxoids_in[4]],
            inputamounts: [amounts_in[0], amounts_in[1], amounts_in[2], amounts_in[3], amounts_in[4]],
            outputasset: asset_out,
            outputamount: amount_out,
            tolerance: tolerance_bps,
        }
    }

    /// hard coded type hash for GenIO
    pub fn type_hash() -> b256 {
        // keccak256("GenIO(bytes32[5] inputassets,bytes32[5] inpututxoids,uint256[5] inputamounts,bytes32 outputasset,uint256 outputamount,uint256 tolerance)")
        let type_hash: b256 = 0x2aba27466153a0e63c25c715bd2cd4807b7f9c8ca6938eb1c37f849c9c6c9fa9;
        type_hash
    }

    /// encode each array of b256's, by concatenating each element.
    fn encode_b256_array(array: [b256; 5]) -> b256 {
        let mut encoded = Bytes::new();
        let mut i: u64 = 0;
        while i < 5 {
            extend(encoded, array[i].to_be_bytes(), 32);
            i += 1;
        }
        hash_bytes(encoded)
    }

    /// calculate tx truct hash with tx specific params.
    pub fn struct_hash(self) -> b256 {
        let mut encoded = Bytes::new();
        extend(encoded, self::type_hash().to_be_bytes(), 32);
        extend(encoded, self::encode_b256_array(self.inputassets).to_be_bytes(), 32);
        extend(encoded, self::encode_b256_array(self.inpututxoids).to_be_bytes(), 32);
        extend(encoded, self::encode_b256_array(self.inputamounts).to_be_bytes(), 32);
        extend(encoded, self.outputasset.to_be_bytes(), 32);
        extend(encoded, self.outputamount.to_be_bytes(), 32);
        extend(encoded, self.tolerance.to_be_bytes(), 32);
        hash_bytes(encoded)
    }
}


pub struct EIP712Domain {
    name: String,
    version: String,
    chain_id: u64,
    verifying_contract: b256,
}

impl EIP712Domain {

    pub fn new() -> EIP712Domain {
        EIP712Domain {
            name: String::from_ascii_str("ZapGeneralizedIO"),
            version: String::from_ascii_str("1"),
            chain_id: 9889,
            verifying_contract: 0x0000000000000000000000000000000000000000000000000000000000000001,
        }
    }

    // precalculated domain separator hash
    pub fn domin_separator_hash_precalc(self) -> b256 {
        let dsh: b256 = 0x47f9d229f5cbfdd9148072eb4928dc4f52479d3ea9d4e578743f6c51b959f445;
        dsh
    }

    pub fn domain_separator_hash(self) -> b256 {
        let mut encoded = Bytes::new();

        // 1. Add EIP712_DOMAIN_TYPE_HASH
        extend(encoded, EIP712_DOMAIN_TYPE_HASH.to_be_bytes(), 32);

        // 2. Add hash of name
        let name_hash = hash_bytes(string_to_bytes(self.name).unwrap());
        extend(encoded, name_hash.to_be_bytes(), 32);

        // 3. Add hash of version
        let version_hash = hash_bytes(string_to_bytes(self.version).unwrap());
        extend(encoded, version_hash.to_be_bytes(), 32);

        // 4. Add chainId (as 32-byte big-endian)
        // let mut chianid_tuple: (u64, u64, u64, u64) = (0, 0, 0, self.chain_id);
        let chainid = to_b256((0, 0, 0, self.chain_id));
        extend(encoded, chainid.to_be_bytes(), 32);

        // 5. Add verifyingContract
        extend(encoded, self.verifying_contract.to_be_bytes(), 32);

        // 6. Compute final hash
        let final_hash = hash_bytes(encoded);
        final_hash
    }

}


pub trait Eip712 {
    fn encode_eip712(self) -> Option<b256>;
}

impl Eip712 for (EIP712Domain, GenIO) {

    /// Calculate the encoded EIP-712 hash by concatenating the
    /// components as per EIP-712 specification.
    /// capacity of byte aray sould be 2 + 32 + 32 = 66 bytes total.
    /// --> digest_input = \x19\x01 + domain_separator + struct_hash
    /// --> domain separator hash = keccak256(digest_input);
    fn encode_eip712(self) -> Option<b256> {

        let (mut domain, tx) = self;
        let domain_separator = domain.domin_separator_hash_precalc();

        let dsh_bytes = domain_separator.to_be_bytes();
        let sh_bytes = tx.struct_hash().to_be_bytes();

        let mut digest_input = Bytes::with_capacity(66);
        // add prefix
        digest_input.push(0x19);
        digest_input.push(0x01);
        // add domain_separator then struct_hash
        extend(digest_input, dsh_bytes, 32);
        extend(digest_input, sh_bytes, 32);

        let hash = hash_bytes(digest_input);
        Some(hash)
    }
}

