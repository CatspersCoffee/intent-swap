predicate;

use std::{
    bytes::Bytes,
    string::String,
};

use helpers::general_helpers::{
    hex_string_to_bytes,
    char_to_hex,
};

configurable {
    RANDOMBYTE: u8 = 0,
}

fn main() -> bool {
    // Dummy code for unique predicate root/addr.
    let mut bytes = Bytes::new();
    bytes.push(0x64);
    bytes.push(0x75);
    bytes.push(0x6d);
    bytes.push(0x6d);
    bytes.push(0x79);
    bytes.push(0x5f);
    bytes.push(0x77);
    bytes.push(0x61);
    bytes.push(0x6c);
    bytes.push(0x6c);
    bytes.push(0x65);
    bytes.push(0x74);
    bytes.push(RANDOMBYTE);

    let mut ascii_hex_string = String::from_ascii_str("64756d6d795f77616c6c6574"); // "dummy_wallet"
    let mut namebytes = hex_string_to_bytes(ascii_hex_string).unwrap();
    namebytes.push(RANDOMBYTE);

    let mut j = 0;
    while j < namebytes.capacity() {
        if (bytes.get(j) != namebytes.get(j)) {
            return false;
        }
        j += 1;
    }
    return true;
}