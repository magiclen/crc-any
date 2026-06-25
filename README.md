CRC Any
====================

[![CI](https://github.com/magiclen/crc-any/actions/workflows/ci.yml/badge.svg)](https://github.com/magiclen/crc-any/actions/workflows/ci.yml)

This crate computes CRC values from a bit width, a polynomial, a reflection setting, an initial value, and a final XOR value. It also provides many built-in CRC functions for common CRC variants.

## Usage

Use the `create_crc` associated function to create a CRC instance. For example, the following code computes a CRC-24 value:

```rust
use crc_any::CRC;

let mut crc24 = CRC::create_crc(0x0000000000864CFB, 24, 0x0000000000B704CE, 0x0000000000000000, false);

crc24.update(b"hello");
```

For simpler usage, this crate also provides built-in functions for many common CRC variants:

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
   * This is also called `crc32b` in `mhash`.
 * crc32mhash
   * The `mhash` library has two CRC32 variants named `crc32` and `crc32b`. In this crate, `crc32` matches `crc32b` from `mhash`, and `crc32mhash` matches `crc32` from `mhash`.
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

For example:

```rust
use crc_any::CRC;

let mut crc64 = CRC::crc64();

crc64.update(b"hello");
```

After you get a CRC value, you can still call `update` to continue computing the CRC with more input data. The `digest` method is still available as a compatibility wrapper for input types that implement `AsRef<[u8]>`.

## CRC-32C Hardware Acceleration

CRC-32C has an optional SSE4.2 fast path on `x86` and `x86_64` targets.

With the default features, the `std` feature is enabled. In this mode, the crate uses runtime CPU feature detection. The same binary can run on CPUs with or without SSE4.2: it uses the hardware-accelerated path when SSE4.2 is available, and falls back to the portable implementation otherwise.

For builds that only run on CPUs known to support SSE4.2, you can enable that CPU feature at compile time, for example with `-C target-cpu=native` or `-C target-feature=+sse4.2`. This lets the crate use the SSE4.2 implementation directly for CRC-32C, without the runtime detection branch, and may also help the compiler optimize the code further.

Do not enable these compile-time options for binaries that must run on older `x86` or `x86_64` CPUs without SSE4.2 support.

## No Std and Heapless Support

To make sure this crate does not use heap allocation, disable the default features. This also disables the `std` runtime CPU feature detection path.

```toml
[dependencies.crc-any]
version = "*"
default-features = false
```

After disabling the default features, the `get_crc_vec_be` and `get_crc_vec_le` methods are not available. If you still need this crate to return a vector-like value without dynamic allocation, enable the `heapless` feature and use the `get_crc_heapless_vec_be` and `get_crc_heapless_vec_le` methods.

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
