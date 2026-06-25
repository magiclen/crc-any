use crc_any::{CRC, CRCu16, CRCu32, CRCu64};

const CHECK_INPUT: &[u8] = b"123456789";

#[test]
fn update_matches_digest() {
    let mut digest_crc = CRCu32::crc32c();
    digest_crc.digest(CHECK_INPUT);

    let mut update_crc = CRCu32::crc32c();
    update_crc.update(CHECK_INPUT);

    assert_eq!(digest_crc.get_crc(), update_crc.get_crc());

    let mut digest_crc = CRC::crc64();
    digest_crc.digest(CHECK_INPUT);

    let mut update_crc = CRC::crc64();
    update_crc.update(CHECK_INPUT);

    assert_eq!(digest_crc.get_crc(), update_crc.get_crc());
}

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
