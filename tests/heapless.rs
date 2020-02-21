#![cfg(feature = "heapless")]

extern crate crc_any;

use crc_any::{CRCu16, CRCu32, CRCu64, CRC};

#[test]
fn crc() {
    let mut crc = CRC::crc64();

    crc.digest(b"https://magiclen.org");

    assert_eq!(vec![46, 219, 104, 85, 36, 10, 96, 248], crc.get_crc_heapless_vec_be().to_vec());
    assert_eq!(vec![248, 96, 10, 36, 85, 104, 219, 46], crc.get_crc_heapless_vec_le().to_vec());

    let mut crc = CRC::crc32();

    crc.digest(b"https://magiclen.org");

    assert_eq!(vec![157, 140, 116, 114], crc.get_crc_heapless_vec_be().to_vec());
    assert_eq!(vec![114, 116, 140, 157], crc.get_crc_heapless_vec_le().to_vec());

    let mut crc = CRC::crc16();

    crc.digest(b"https://magiclen.org");

    assert_eq!(vec![77, 150], crc.get_crc_heapless_vec_be().to_vec());
    assert_eq!(vec![150, 77], crc.get_crc_heapless_vec_le().to_vec());

    let mut crc = CRC::crc8();

    crc.digest(b"https://magiclen.org");

    assert_eq!(vec![45], crc.get_crc_heapless_vec_be().to_vec());
    assert_eq!(vec![45], crc.get_crc_heapless_vec_le().to_vec());
}

#[test]
fn crc_u16() {
    let mut crc = CRCu16::crc16();

    crc.digest(b"https://magiclen.org");

    assert_eq!(vec![77, 150], crc.get_crc_heapless_vec_be().to_vec());
    assert_eq!(vec![150, 77], crc.get_crc_heapless_vec_le().to_vec());
}

#[test]
fn crc_u32() {
    let mut crc = CRCu32::crc32();

    crc.digest(b"https://magiclen.org");

    assert_eq!(vec![157, 140, 116, 114], crc.get_crc_heapless_vec_be().to_vec());
    assert_eq!(vec![114, 116, 140, 157], crc.get_crc_heapless_vec_le().to_vec());

    let mut crc = CRCu32::crc24();

    crc.digest(b"https://magiclen.org");

    assert_eq!(vec![59, 98, 20], crc.get_crc_heapless_vec_be().to_vec());
    assert_eq!(vec![20, 98, 59], crc.get_crc_heapless_vec_le().to_vec());
}

#[test]
fn crc_u64() {
    let mut crc = CRCu64::crc64();

    crc.digest(b"https://magiclen.org");

    assert_eq!(vec![46, 219, 104, 85, 36, 10, 96, 248], crc.get_crc_heapless_vec_be().to_vec());
    assert_eq!(vec![248, 96, 10, 36, 85, 104, 219, 46], crc.get_crc_heapless_vec_le().to_vec());

    let mut crc = CRCu64::crc40gsm();

    crc.digest(b"https://magiclen.org");

    assert_eq!(vec![90, 94, 5, 195, 152], crc.get_crc_heapless_vec_be().to_vec());
    assert_eq!(vec![152, 195, 5, 94, 90], crc.get_crc_heapless_vec_le().to_vec());
}
