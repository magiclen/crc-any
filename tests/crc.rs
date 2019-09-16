#![cfg(feature = "alloc")]

extern crate crc_any;

use crc_any::CRC;

// TODO: CRC-3

#[test]
fn crc3gsm() {
    let mut crc = CRC::crc3gsm();

    crc.digest(b"https://magiclen.org");

    assert_eq!("0x7", &crc.to_string());
}

// TODO: CRC-4

#[test]
fn crc4itu() {
    let mut crc = CRC::crc4itu();

    crc.digest(b"https://magiclen.org");

    assert_eq!("0x3", &crc.to_string());
}

#[test]
fn crc4interlaken() {
    let mut crc = CRC::crc4interlaken();

    crc.digest(b"https://magiclen.org");

    assert_eq!("0xF", &crc.to_string());
}

// TODO: CRC-5

#[test]
fn crc5epc() {
    let mut crc = CRC::crc5epc();

    crc.digest(b"https://magiclen.org");

    assert_eq!("0x1C", &crc.to_string());
}

#[test]
fn crc5itu() {
    let mut crc = CRC::crc5itu();

    crc.digest(b"https://magiclen.org");

    assert_eq!("0x03", &crc.to_string());
}

#[test]
fn crc5usb() {
    let mut crc = CRC::crc5usb();

    crc.digest(b"https://magiclen.org");

    assert_eq!("0x16", &crc.to_string());
}

// TODO: CRC-6

#[test]
fn crc6cdma2000_a() {
    let mut crc = CRC::crc6cdma2000_a();

    crc.digest(b"https://magiclen.org");

    assert_eq!("0x15", &crc.to_string());
}

#[test]
fn crc6cdma2000_b() {
    let mut crc = CRC::crc6cdma2000_b();

    crc.digest(b"https://magiclen.org");

    assert_eq!("0x36", &crc.to_string());
}

#[test]
fn crc6darc() {
    let mut crc = CRC::crc6darc();

    crc.digest(b"https://magiclen.org");

    assert_eq!("0x09", &crc.to_string());
}

#[test]
fn crc6gsm() {
    let mut crc = CRC::crc6gsm();

    crc.digest(b"https://magiclen.org");

    assert_eq!("0x27", &crc.to_string());
}

#[test]
fn crc6itu() {
    let mut crc = CRC::crc6itu();

    crc.digest(b"https://magiclen.org");

    assert_eq!("0x2E", &crc.to_string());
}

// TODO: CRC-7

#[test]
fn crc7() {
    let mut crc = CRC::crc7();

    crc.digest(b"https://magiclen.org");

    assert_eq!("0x36", &crc.to_string());
}

#[test]
fn crc7umts() {
    let mut crc = CRC::crc7umts();

    crc.digest(b"https://magiclen.org");

    assert_eq!("0x65", &crc.to_string());
}

// TODO: CRC-8

#[test]
fn crc8() {
    let mut crc = CRC::crc8();

    crc.digest(b"https://magiclen.org");

    assert_eq!("0x2D", &crc.to_string());
}

#[test]
fn crc8cdma2000() {
    let mut crc = CRC::crc8cdma2000();

    crc.digest(b"https://magiclen.org");

    assert_eq!("0xA9", &crc.to_string());
}

#[test]
fn crc8darc() {
    let mut crc = CRC::crc8darc();

    crc.digest(b"https://magiclen.org");

    assert_eq!("0xB0", &crc.to_string());
}

#[test]
fn crc8dvb_s2() {
    let mut crc = CRC::crc8dvb_s2();

    crc.digest(b"https://magiclen.org");

    assert_eq!("0xBE", &crc.to_string());
}

#[test]
fn crc8ebu() {
    let mut crc = CRC::crc8ebu();

    crc.digest(b"https://magiclen.org");

    assert_eq!("0x39", &crc.to_string());
}

#[test]
fn crc8icode() {
    let mut crc = CRC::crc8icode();

    crc.digest(b"https://magiclen.org");

    assert_eq!("0xA6", &crc.to_string());
}

#[test]
fn crc8itu() {
    let mut crc = CRC::crc8itu();

    crc.digest(b"https://magiclen.org");

    assert_eq!("0x78", &crc.to_string());
}

#[test]
fn crc8maxim() {
    let mut crc = CRC::crc8maxim();

    crc.digest(b"https://magiclen.org");

    assert_eq!("0x35", &crc.to_string());
}

#[test]
fn crc8rohc() {
    let mut crc = CRC::crc8rohc();

    crc.digest(b"https://magiclen.org");

    assert_eq!("0x18", &crc.to_string());
}

#[test]
fn crc8wcdma() {
    let mut crc = CRC::crc8wcdma();

    crc.digest(b"https://magiclen.org");

    assert_eq!("0x05", &crc.to_string());
}

// TODO: CRC-10

#[test]
fn crc10() {
    let mut crc = CRC::crc10();

    crc.digest(b"https://magiclen.org");

    assert_eq!("0x254", &crc.to_string());
}

#[test]
fn crc10cdma2000() {
    let mut crc = CRC::crc10cdma2000();

    crc.digest(b"https://magiclen.org");

    assert_eq!("0x3EE", &crc.to_string());
}

#[test]
fn crc10gsm() {
    let mut crc = CRC::crc10gsm();

    crc.digest(b"https://magiclen.org");

    assert_eq!("0x2FD", &crc.to_string());
}

// TODO: CRC-11

#[test]
fn crc11() {
    let mut crc = CRC::crc11();

    crc.digest(b"https://magiclen.org");

    assert_eq!("0x3D9", &crc.to_string());
}

// TODO: CRC-12

#[test]
fn crc12() {
    let mut crc = CRC::crc12();

    crc.digest(b"https://magiclen.org");

    assert_eq!("0x511", &crc.to_string());
}

#[test]
fn crc12cdma2000() {
    let mut crc = CRC::crc12cdma2000();

    crc.digest(b"https://magiclen.org");

    assert_eq!("0x71A", &crc.to_string());
}

#[test]
fn crc12gsm() {
    let mut crc = CRC::crc12gsm();

    crc.digest(b"https://magiclen.org");

    assert_eq!("0x3CB", &crc.to_string());
}

// TODO: CRC-13

#[test]
fn crc13bbc() {
    let mut crc = CRC::crc13bbc();

    crc.digest(b"https://magiclen.org");

    assert_eq!("0x0185", &crc.to_string());
}

// TODO: CRC-14

#[test]
fn crc14darc() {
    let mut crc = CRC::crc14darc();

    crc.digest(b"https://magiclen.org");

    assert_eq!("0x385D", &crc.to_string());
}

#[test]
fn crc14gsm() {
    let mut crc = CRC::crc14gsm();

    crc.digest(b"https://magiclen.org");

    assert_eq!("0x1D82", &crc.to_string());
}

// TODO: CRC-15

#[test]
fn crc15can() {
    let mut crc = CRC::crc15can();

    crc.digest(b"https://magiclen.org");

    assert_eq!("0x32C1", &crc.to_string());
}

#[test]
fn crc15mpt1327() {
    let mut crc = CRC::crc15mpt1327();

    crc.digest(b"https://magiclen.org");

    assert_eq!("0x5071", &crc.to_string());
}

// TODO: CRC-16

#[test]
fn crc16() {
    let mut crc = CRC::crc16();

    crc.digest(b"https://magiclen.org");

    assert_eq!("0x4D96", &crc.to_string());
}

#[test]
fn crc16ccitt_false() {
    let mut crc = CRC::crc16ccitt_false();

    crc.digest(b"https://magiclen.org");

    assert_eq!("0x3E46", &crc.to_string());
}

#[test]
fn crc16aug_ccitt() {
    let mut crc = CRC::crc16aug_ccitt();

    crc.digest(b"https://magiclen.org");

    assert_eq!("0x574A", &crc.to_string());
}

#[test]
fn crc16buypass() {
    let mut crc = CRC::crc16buypass();

    crc.digest(b"https://magiclen.org");

    assert_eq!("0xF25B", &crc.to_string());
}

#[test]
fn crc16cdma2000() {
    let mut crc = CRC::crc16cdma2000();

    crc.digest(b"https://magiclen.org");

    assert_eq!("0xBF6C", &crc.to_string());
}

#[test]
fn crc16dds_110() {
    let mut crc = CRC::crc16dds_110();

    crc.digest(b"https://magiclen.org");

    assert_eq!("0x288B", &crc.to_string());
}

#[test]
fn crc16dect_r() {
    let mut crc = CRC::crc16dect_r();

    crc.digest(b"https://magiclen.org");

    assert_eq!("0x673F", &crc.to_string());
}

#[test]
fn crc16dect_x() {
    let mut crc = CRC::crc16dect_x();

    crc.digest(b"https://magiclen.org");

    assert_eq!("0x673E", &crc.to_string());
}

#[test]
fn crc16dnp() {
    let mut crc = CRC::crc16dnp();

    crc.digest(b"https://magiclen.org");

    assert_eq!("0x5DFD", &crc.to_string());
}

#[test]
fn crc16en_13757() {
    let mut crc = CRC::crc16en_13757();

    crc.digest(b"https://magiclen.org");

    assert_eq!("0xC355", &crc.to_string());
}

#[test]
fn crc16genibus() {
    let mut crc = CRC::crc16genibus();

    crc.digest(b"https://magiclen.org");

    assert_eq!("0xC1B9", &crc.to_string());
}

#[test]
fn crc16maxim() {
    let mut crc = CRC::crc16maxim();

    crc.digest(b"https://magiclen.org");

    assert_eq!("0xB269", &crc.to_string());
}

#[test]
fn crc16mcrf4cc() {
    let mut crc = CRC::crc16mcrf4cc();

    crc.digest(b"https://magiclen.org");

    assert_eq!("0xF238", &crc.to_string());
}

#[test]
fn crc16riello() {
    let mut crc = CRC::crc16riello();

    crc.digest(b"https://magiclen.org");

    assert_eq!("0xB319", &crc.to_string());
}

#[test]
fn crc16t10_dif() {
    let mut crc = CRC::crc16t10_dif();

    crc.digest(b"https://magiclen.org");

    assert_eq!("0x2BE3", &crc.to_string());
}

#[test]
fn crc16teledisk() {
    let mut crc = CRC::crc16teledisk();

    crc.digest(b"https://magiclen.org");

    assert_eq!("0xAAC0", &crc.to_string());
}

#[test]
fn crc16tms13157() {
    let mut crc = CRC::crc16tms13157();

    crc.digest(b"https://magiclen.org");

    assert_eq!("0x04C7", &crc.to_string());
}

#[test]
fn crc16usb() {
    let mut crc = CRC::crc16usb();

    crc.digest(b"https://magiclen.org");

    assert_eq!("0xA94D", &crc.to_string());
}

#[test]
fn crc_a() {
    let mut crc = CRC::crc_a();

    crc.digest(b"https://magiclen.org");

    assert_eq!("0x6EEC", &crc.to_string());
}

#[test]
fn crc16kermit() {
    let mut crc = CRC::crc16kermit();

    crc.digest(b"https://magiclen.org");

    assert_eq!("0xEF57", &crc.to_string());
}

#[test]
fn crc16modbus() {
    let mut crc = CRC::crc16modbus();

    crc.digest(b"https://magiclen.org");

    assert_eq!("0x56B2", &crc.to_string());
}

#[test]
fn crc16_x25() {
    let mut crc = CRC::crc16_x25();

    crc.digest(b"https://magiclen.org");

    assert_eq!("0x0DC7", &crc.to_string());
}

#[test]
fn crc16xmodem() {
    let mut crc = CRC::crc16xmodem();

    crc.digest(b"https://magiclen.org");

    assert_eq!("0xC8FE", &crc.to_string());
}

// TODO: CRC-17

#[test]
fn crc17can() {
    let mut crc = CRC::crc17can();

    crc.digest(b"https://magiclen.org");

    assert_eq!("0x145A6", &crc.to_string());
}

// TODO: CRC-21

#[test]
fn crc21can() {
    let mut crc = CRC::crc21can();

    crc.digest(b"https://magiclen.org");

    assert_eq!("0x1750F3", &crc.to_string());
}

// TODO: CRC-24

#[test]
fn crc24() {
    let mut crc = CRC::crc24();

    crc.digest(b"https://magiclen.org");

    assert_eq!("0x3B6214", &crc.to_string());
}

#[test]
fn crc24ble() {
    let mut crc = CRC::crc24ble();

    crc.digest(b"https://magiclen.org");

    assert_eq!("0x1CEC41", &crc.to_string());
}

#[test]
fn crc24flexray_a() {
    let mut crc = CRC::crc24flexray_a();

    crc.digest(b"https://magiclen.org");

    assert_eq!("0x6BF9F6", &crc.to_string());
}

#[test]
fn crc24flexray_b() {
    let mut crc = CRC::crc24flexray_b();

    crc.digest(b"https://magiclen.org");

    assert_eq!("0x87D0F0", &crc.to_string());
}

#[test]
fn crc24lte_a() {
    let mut crc = CRC::crc24lte_a();

    crc.digest(b"https://magiclen.org");

    assert_eq!("0xCD0FF8", &crc.to_string());
}

#[test]
fn crc24lte_b() {
    let mut crc = CRC::crc24lte_b();

    crc.digest(b"https://magiclen.org");

    assert_eq!("0x09DCD6", &crc.to_string());
}

#[test]
fn crc24os9() {
    let mut crc = CRC::crc24os9();

    crc.digest(b"https://magiclen.org");

    assert_eq!("0x8F1D37", &crc.to_string());
}

// TODO: CRC-30

#[test]
fn crc30cdma() {
    let mut crc = CRC::crc30cdma();

    crc.digest(b"https://magiclen.org");

    assert_eq!("0x0964EB13", &crc.to_string());
}

// TODO: CRC-32

#[test]
fn crc32() {
    let mut crc = CRC::crc32();

    crc.digest(b"https://magiclen.org");

    assert_eq!("0x9D8C7472", &crc.to_string());
}

#[test]
fn crc32mhash() {
    let mut crc = CRC::crc32mhash();

    crc.digest(b"https://magiclen.org");

    assert_eq!("0x2B457DD6", &crc.to_string());
}

#[test]
fn crc32bzip2() {
    let mut crc = CRC::crc32bzip2();

    crc.digest(b"https://magiclen.org");

    assert_eq!("0xD67D452B", &crc.to_string());
}

#[test]
fn crc32c() {
    let mut crc = CRC::crc32c();

    crc.digest(b"https://magiclen.org");

    assert_eq!("0x3E3CD9E7", &crc.to_string());
}

#[test]
fn crc32d() {
    let mut crc = CRC::crc32d();

    crc.digest(b"https://magiclen.org");

    assert_eq!("0x6369CF37", &crc.to_string());
}

#[test]
fn crc32mpeg2() {
    let mut crc = CRC::crc32mpeg2();

    crc.digest(b"https://magiclen.org");

    assert_eq!("0x2982BAD4", &crc.to_string());
}

#[test]
fn crc32posix() {
    let mut crc = CRC::crc32posix();

    crc.digest(b"https://magiclen.org");

    assert_eq!("0x985B1124", &crc.to_string());
}

#[test]
fn crc32q() {
    let mut crc = CRC::crc32q();

    crc.digest(b"https://magiclen.org");

    assert_eq!("0x013A925F", &crc.to_string());
}

#[test]
fn crc32jamcrc() {
    let mut crc = CRC::crc32jamcrc();

    crc.digest(b"https://magiclen.org");

    assert_eq!("0x62738B8D", &crc.to_string());
}

#[test]
fn crc32xfer() {
    let mut crc = CRC::crc32xfer();

    crc.digest(b"https://magiclen.org");

    assert_eq!("0xF639C6D2", &crc.to_string());
}

// TODO: CRC-40

#[test]
fn crc40gsm() {
    let mut crc = CRC::crc40gsm();

    crc.digest(b"https://magiclen.org");

    assert_eq!("0x5A5E05C398", &crc.to_string());
}

// TODO: CRC-64

#[test]
fn crc64() {
    let mut crc = CRC::crc64();

    crc.digest(b"https://magiclen.org");

    assert_eq!("0x13FCB547092E64B6", &crc.to_string());
}

#[test]
fn crc64iso() {
    let mut crc = CRC::crc64iso();

    crc.digest(b"https://magiclen.org");

    assert_eq!("0x649DB068ECA5B9DB", &crc.to_string());
}

#[test]
fn crc64jones() {
    let mut crc = CRC::crc64jones();

    crc.digest(b"https://magiclen.org");

    assert_eq!("0x4BE96FCDBAD0D303", &crc.to_string());
}
