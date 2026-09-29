use crc_any::{CRC, CRCu8, CRCu16, CRCu32, CRCu64};

const INPUT: &[u8] = b"123456789";

#[test]
fn crc() {
    let mut crc = CRC::crc3gsm();
    crc.update(INPUT);
    assert_eq!("0x4", format!("{crc}"));

    let mut crc = CRC::crc32c();
    crc.update(INPUT);
    assert_eq!("0xE3069283", format!("{crc}"));
}

#[test]
fn crc_types() {
    let mut crc = CRCu8::crc5epc();
    crc.update(INPUT);
    assert_eq!("0x00", format!("{crc}"));

    let mut crc = CRCu16::crc13bbc();
    crc.update(INPUT);
    assert_eq!("0x04FA", format!("{crc}"));

    let mut crc = CRCu32::crc17can();
    crc.update(INPUT);
    assert_eq!("0x04F03", format!("{crc}"));

    let mut crc = CRCu64::crc40gsm();
    crc.update(INPUT);
    assert_eq!("0xD4164FC646", format!("{crc}"));
}
