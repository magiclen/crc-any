use crc_any::{CRCu16, CRCu32, CRCu64, CRC};

const CHECK_INPUT: &[u8] = b"123456789";

#[test]
fn reset_matches_fresh_for_reflected_crc16() {
    let mut fresh = CRCu16::crc16riello();
    fresh.digest(CHECK_INPUT);
    let expected = fresh.get_crc();

    let mut reused = CRCu16::crc16riello();
    reused.digest(CHECK_INPUT);
    reused.reset();
    reused.digest(CHECK_INPUT);

    assert_eq!(0x63D0, expected);
    assert_eq!(expected, reused.get_crc());
}

#[test]
fn reset_matches_fresh_for_reflected_crc24() {
    let mut fresh = CRCu32::crc24ble();
    fresh.digest(CHECK_INPUT);
    let expected = fresh.get_crc();

    let mut reused = CRCu32::crc24ble();
    reused.digest(CHECK_INPUT);
    reused.reset();
    reused.digest(CHECK_INPUT);

    assert_eq!(0xC25A56, expected);
    assert_eq!(expected, reused.get_crc());
}

#[test]
fn reset_matches_fresh_for_reflected_crc64() {
    let mut fresh = CRCu64::crc64iso();
    fresh.digest(CHECK_INPUT);
    let expected = fresh.get_crc();

    let mut reused = CRCu64::crc64iso();
    reused.digest(CHECK_INPUT);
    reused.reset();
    reused.digest(CHECK_INPUT);

    assert_eq!(0xB90956C775A41001, expected);
    assert_eq!(expected, reused.get_crc());
}

#[test]
fn enum_reset_matches_fresh_for_reflected_crc() {
    let mut fresh = CRC::crc16riello();
    fresh.digest(CHECK_INPUT);
    let expected = fresh.get_crc();

    let mut reused = CRC::crc16riello();
    reused.digest(CHECK_INPUT);
    reused.reset();
    reused.digest(CHECK_INPUT);

    assert_eq!(0x63D0, expected);
    assert_eq!(expected, reused.get_crc());
}
