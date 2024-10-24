use fuels::types::{
    Bits256, Bytes32,
};

pub mod conversions {
    use super::*;

    pub fn bytes32_to_bits256(bytes: Bytes32) -> Bits256 {
        let mut array = [0u8; 32];
        array.copy_from_slice(&bytes.as_ref());
        Bits256(array)
    }

    pub fn bits256_to_vecu8(a: Bits256) -> Vec<u8> {
        let mut v_bytes: [u8; 32] = [0; 32];
        v_bytes.copy_from_slice(&a.0);
        //let vec: Vec<u8> = Vec::new();
        let vec = v_bytes.into();
        vec
    }

    // Convert a u64 into a Bits256
    pub fn u64_to_bits256(a: u64) -> Bits256 {
        let mut bytes32arr: [u8; 32] = [0; 32];
        for i in 0..8 {
            bytes32arr[31 - i] = ((a >> (i * 8)) & 0xFF) as u8;
        }
        let mut b: Bits256 = Bits256::from_hex_str(
            "0x0000000000000000000000000000000000000000000000000000000000000000"
            ).unwrap();
        b.0.copy_from_slice(&bytes32arr);
        b
    }

}

pub mod display {

    pub fn print_separator_line(length: usize, suffix: &str) {
        // Calculate the length of the dashes
        let dash_length = if length > suffix.len() {
            length - suffix.len()
        } else {
            0
        };
        let dashes = "-".repeat(dash_length);
        println!("\n{} {}", dashes, suffix);
    }
}