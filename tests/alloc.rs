#![cfg(feature = "alloc")]

use crc_any::{CRCu16, CRCu32, CRCu64, CRC};

#[test]
fn crc() {
    let mut crc = CRC::crc64we();

    crc.digest(b"https://magiclen.org");

    assert_eq!(vec![19, 252, 181, 71, 9, 46, 100, 182], crc.get_crc_vec_be());
    assert_eq!(vec![182, 100, 46, 9, 71, 181, 252, 19], crc.get_crc_vec_le());

    let mut crc = CRC::crc32c();

    crc.digest(b"https://magiclen.org");

    assert_eq!(vec![62, 60, 217, 231], crc.get_crc_vec_be());
    assert_eq!(vec![231, 217, 60, 62], crc.get_crc_vec_le());

    let mut crc = CRC::crc16cdma2000();

    crc.digest(b"https://magiclen.org");

    assert_eq!(vec![191, 108], crc.get_crc_vec_be());
    assert_eq!(vec![108, 191], crc.get_crc_vec_le());

    let mut crc = CRC::crc8cdma2000();

    crc.digest(b"https://magiclen.org");

    assert_eq!(vec![169], crc.get_crc_vec_be());
    assert_eq!(vec![169], crc.get_crc_vec_le());
}

#[test]
fn crc_u16() {
    let mut crc = CRCu16::crc16();

    crc.digest(b"https://magiclen.org");

    assert_eq!(vec![77, 150], crc.get_crc_vec_be());
    assert_eq!(vec![150, 77], crc.get_crc_vec_le());
}

#[test]
fn crc_u32() {
    let mut crc = CRCu32::crc32();

    crc.digest(b"https://magiclen.org");

    assert_eq!(vec![157, 140, 116, 114], crc.get_crc_vec_be());
    assert_eq!(vec![114, 116, 140, 157], crc.get_crc_vec_le());

    let mut crc = CRCu32::crc24();

    crc.digest(b"https://magiclen.org");

    assert_eq!(vec![59, 98, 20], crc.get_crc_vec_be());
    assert_eq!(vec![20, 98, 59], crc.get_crc_vec_le());
}

#[test]
fn crc_u64() {
    let mut crc = CRCu64::crc64();

    crc.digest(b"https://magiclen.org");

    assert_eq!(vec![46, 219, 104, 85, 36, 10, 96, 248], crc.get_crc_vec_be());
    assert_eq!(vec![248, 96, 10, 36, 85, 104, 219, 46], crc.get_crc_vec_le());

    let mut crc = CRCu64::crc40gsm();

    crc.digest(b"https://magiclen.org");

    assert_eq!(vec![90, 94, 5, 195, 152], crc.get_crc_vec_be());
    assert_eq!(vec![152, 195, 5, 94, 90], crc.get_crc_vec_le());
}
