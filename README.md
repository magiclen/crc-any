CRC Any
====================

[![CI](https://github.com/magiclen/crc-any/actions/workflows/ci.yml/badge.svg)](https://github.com/magiclen/crc-any/actions/workflows/ci.yml)

To compute CRC values by providing the length of bits, expression, reflection, an initial value and a final xor value. It has many built-in CRC functions.

## Usage

You can use `create_crc` associated function to create a CRC instance by providing the length of bits, expression, reflection, an initial value and a final xor value. For example, if you want to compute a CRC-24 value.

```rust
use crc_any::CRC;

let mut crc24 = CRC::create_crc(0x0000000000864CFB, 24, 0x0000000000B704CE, 0x0000000000000000, false);

crc24.digest(b"hello");

assert_eq!([71, 245, 138].to_vec(), crc24.get_crc_vec_be());
assert_eq!("0x47F58A", &crc24.to_string());
```

To simplify the usage, there are several common versions of CRC whose computing functions are already built-in.

 * crc3gsm
 * crc4itu
 * crc4interlaken
 * crc5epc
 * crc5itu
 * crc5usb
 * crc6cdma2000_a
 * crc6cdma2000_b
 * crc6darc
 * crc6gsm
 * crc6itu
 * crc7
 * crc7umts
 * crc8
 * crc8cdma2000
 * crc8darc
 * crc8dvb_s2
 * crc8ebu
 * crc8icode
 * crc8itu
 * crc8maxim
 * crc8rohc
 * crc8wcdma
 * crc10
 * crc10cdma2000
 * crc10gsm
 * crc11
 * crc12
 * crc12cdma2000
 * crc12gsm
 * crc13bbc
 * crc14darc
 * crc14gsm
 * crc15can
 * crc15mpt1327
 * crc16
 * crc16ccitt_false
 * crc16aug_ccitt
 * crc16buypass
 * crc16cdma2000
 * crc16dds_110
 * crc16dect_r
 * crc16dect_x
 * crc16dnp
 * crc16en_13757
 * crc16genibus
 * crc16maxim
 * crc16mcrf4cc
 * crc16riello
 * crc16t10_dif
 * crc16teledisk
 * crc16tms13157
 * crc16usb
 * crc_a
 * crc16kermit
 * crc16modbus
 * crc16_x25
 * crc16xmodem
 * crc17can
 * crc21can
 * crc24
 * crc24ble
 * crc24flexray_a
 * crc24flexray_b
 * crc24lte_a
 * crc24lte_b
 * crc24os9
 * crc30cdma
 * crc32
   * It also called `crc32b` in `mhash`.
 * crc32mhash
   * `mhash` is a common library which has two weird versions of CRC32 called `crc32` and `crc32b`. `crc32` and `crc32mhash` in this module are `crc32b` and `crc32` in mhash respectively.
 * crc32bzip2
 * crc32c
 * crc32d
 * crc32mpeg2
 * crc32posix
 * crc32q
 * crc32jamcrc
 * crc32xfer
 * crc40gsm
 * crc64
 * crc64iso
 * crc64we
 * crc64jones

For instance,

```rust
use crc_any::CRC;

let mut crc64 = CRC::crc64();

crc64.digest(b"hello");

assert_eq!([64, 84, 74, 48, 97, 55, 182, 236].to_vec(), crc64.get_crc_vec_be());
assert_eq!("0x40544A306137B6EC", &crc64.to_string());
```

After getting a CRC value, you can still use the `digest` method to continue computing the next CRC values.

## Heapless Support

To make sure this crate will not use heap memory allocation, you can disable the default features.

```toml
[dependencies.crc-any]
version = "*"
default-features = false
```

After doing that, the `get_crc_vec_be` and `get_crc_vec_le` methods can not be used. But if you still need this crate to return a `Vec` without dynamic allocation, you can enable the `heapless` feature to make the `get_crc_heapless_vec_be` and `get_crc_heapless_vec_le` methods available.

```toml
[dependencies.crc-any]
version = "*"
default-features = false
features = ["heapless"]
```

## Crates.io

https://crates.io/crates/crc-any

## Documentation

https://docs.rs/crc-any

## License

[MIT](LICENSE)
