extern crate crc_any;

use crc_any::CRC;

#[test]
fn crc64iso() {
    let mut crc = CRC::crc64iso();

    crc.digest(b"https://magiclen.org");

    assert_eq!([100, 157, 176, 104, 236, 165, 185, 219].to_vec(), crc.get_crc_vec());
}

#[test]
fn crc64ecma() {
    let mut crc = CRC::crc64ecma();

    crc.digest(b"https://magiclen.org");

    assert_eq!([19, 252, 181, 71, 9, 46, 100, 182].to_vec(), crc.get_crc_vec());
}

#[test]
fn crc32ieee() {
    let mut crc = CRC::crc32ieee();

    crc.digest(b"https://magiclen.org");

    assert_eq!([157, 140, 116, 114].to_vec(), crc.get_crc_vec());
}

#[test]
fn crc32mhash() {
    let mut crc = CRC::crc32mhash();

    crc.digest(b"https://magiclen.org");

    assert_eq!([43, 69, 125, 214].to_vec(), crc.get_crc_vec());
}

#[test]
fn crc32c() {
    let mut crc = CRC::crc32c();

    crc.digest(b"https://magiclen.org");

    assert_eq!([62, 60, 217, 231].to_vec(), crc.get_crc_vec());
}

#[test]
fn crc16ibm() {
    let mut crc = CRC::crc16ibm();

    crc.digest(b"https://magiclen.org");

    assert_eq!([77, 150].to_vec(), crc.get_crc_vec());
}

#[test]
fn crc16ccitt() {
    let mut crc = CRC::crc16ccitt();

    crc.digest(b"https://magiclen.org");

    assert_eq!([62, 70].to_vec(), crc.get_crc_vec());
}

#[test]
fn crc8atm() {
    let mut crc = CRC::crc8atm();

    crc.digest(b"https://magiclen.org");

    assert_eq!([45].to_vec(), crc.get_crc_vec());
}

#[test]
fn crc8cdma() {
    let mut crc = CRC::crc8cdma();

    crc.digest(b"https://magiclen.org");

    assert_eq!([169].to_vec(), crc.get_crc_vec());
}