library;

use std::{
    bytes::Bytes,
    math::*,
    option::Option,
    string::String,
};
use std::*;
use std::bytes_conversions::{b256::*, u64::*};
use std::primitive_conversions::{u16::*, u32::*, u64::*};

use ::numeric_utils::*;


//----------------------------------------------------------------------------------
// TEST psudo-bignum operations

// forc test test_u64_add_overflow --logs
// test
#[test]
fn test_u64_add_overflow() {

    let ovf = check_u64_addition_overflow(0xFFFFFFFFFFFFFFFE, 0x01);
    log(ovf);   // should be true
    assert(ovf == true);

    let ovf = check_u64_addition_overflow(0xFFFFFFFFFFFFFFFE, 0x02);
    log(ovf);   // should be false
    assert(ovf == false);
}

// forc test test_b256_operations --logs
#[test]
fn test_b256_operations() {

    // Simple addition:
    // 520000000 --> 1EFE9200
    // 1000000000 --> 3B9ACA00
    // 1520000000 --> 5A995C00
    let a: b256 = 0x000000000000000000000000000000000000000000000000000000001EFE9200;
    let b: b256 = 0x000000000000000000000000000000000000000000000000000000003B9ACA00;
    let c: b256 = 0x000000000000000000000000000000000000000000000000000000005a995c00;
    let result = add_b256(a, b).unwrap();
    // log(result);
    assert(result == c);

    // add 0 to 200 decimal.
    let a: b256 = 0x00000000000000000000000000000000000000000000000000000000000000C8; // decimal 200
    let b: b256 = 0x0000000000000000000000000000000000000000000000000000000000000000;
    let c: b256 = 0x00000000000000000000000000000000000000000000000000000000000000C8;
    let result = add_b256(a, b).unwrap();
    // log(result);
    assert(result == c);

    // test u64 max
    let max_u64 = to_b256((0, 0, 0, 0xFFFFFFFFFFFFFFFE));
    let one = to_b256((0, 0, 0, 0x01));
    let result = add_b256(max_u64, one);
    // assert(result.is_none());
    match result {
        Some(v) => assert(v == to_b256((0, 0, 0, 0xFFFFFFFFFFFFFFFF))),
        None => revert(2335u64),
    }

    // test u64 overflow
    let max_u64 = to_b256((0, 0, 0, 0xFFFFFFFFFFFFFFFF));
    let one = to_b256((0, 0, 0, 0x01));
    let result = add_b256(max_u64, one);
    // assert(result.is_none());
    let mut r = b256::zero();
    match result {
        Some(v) => {
            r = v;
        },
        None => {
            r = to_b256((0, 0, 0, 0xaa)); // should handle the None case, revert or return Error.
        },
    }
    assert(r == to_b256((0, 0, 0, 0xaa)))

}

// forc test test_b256_to_u64 --logs
#[test]
fn test_b256_to_u64() {
    // let x: b256 = 0x00000000000000000000000000000000000000000000000000000000000000fa;
    let x: b256 = 0x000000000000000000000000000000000000000000000000FFFFFFFFFFFFFFFF; // u64 max
    let y = b256_to_u64(x);
    log(y);
    assert(y == 0xFFFFFFFFFFFFFFFFu64);
}

// forc test test_b256_sub --logs
#[test]
fn test_b256_sub() {
    let x: b256 = 0x000000000000000000000000000000000000000000000000FFFFFFFFFFFFFFFF; // u64 max
    let y: b256 = 0x00000000000000000000000000000000000000000000000000000000000000fa; // 250 decimal
    let z = sub_b256_packed_u64(x, y);
    log(z); // should be Some(18446744073709551365)
    assert(z == Some(18446744073709551365));

    let x: b256 = 0x000000000000000000000000000000000000000000000000FFFFFFFFFFFFFFFF; // u64 max
    let y: b256 = 0x000000000000000000000000000000000000000000000000FFFFFFFFFFFFFFFF; // u64 max
    let z = sub_b256_packed_u64(x, y);
    log(z); // should be Some(0)
    assert(z == Some(0));

    // return None, y > x
    let x: b256 = 0x0000000000000000000000000000000000000000000000000000000000000001; // 1 decimal
    let y: b256 = 0x000000000000000000000000000000000000000000000000FFFFFFFFFFFFFFFF; // u64 max
    let z = sub_b256_packed_u64(x, y);
    log(z); // should be None
    assert(z == None);
}
