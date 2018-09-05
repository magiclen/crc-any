CRC Any
====================

[![Build Status](https://travis-ci.org/magiclen/crc-any.svg?branch=master)](https://travis-ci.org/magiclen/crc-any)
[![Build status](https://ci.appveyor.com/api/projects/status/pnjmg58he731e8o1/branch/master?svg=true)](https://ci.appveyor.com/project/magiclen/crc-any/branch/master)

To compute CRC values by providing the length of bits, expression, reflection, an initial value and a final xor value. It has built-in CRC-8-ATM, CRC-8-CDMA, CRC-16-IBM, CRC16-CCITT, CRC-32-IEEE, CRC-32-C, CRC-64-ISO and CRC-64-ECMA functions.

## Usage

You can use `create_crc` associated function to create a CRC instance by providing the length of bits, expression, reflection, an initial value and a final xor value. For example, if you want to compute a CRC-24 value.

```rust
extern crate crc_any;

use crc_any::CRC;

let mut crc24 = CRC::create_crc(0x0000000000864cfb, 24, 0x0000000000b704ce, 0x0000000000000000, false);

crc24.digest(b"hello");

assert_eq!([71, 245, 138].to_vec(), crc24.get_crc_vec());
```

To simplify the usage, there are several common versions of CRC whose computing functions are already built-in.

 * crc8(crc8atm)
 * crc8cdma
 * crc16(crc16ibm)
 * crc16ccitt(crcccitt)
 * crc32(crc32ieee, also called crc32b in `mhash`)
 * crc32mhash
   * `mhash` is a common library which has two weird versions of CRC32 called `crc32` and `crc32b`. `crc32` and `crc32mhash` in this module are `crc32b` and `crc32` in mhash respectively.
 * crc32c
 * crc64(crc64ecma)
 * crc64iso

For instance,

```rust
extern crate crc_any;

use crc_any::CRC;

let mut crc64ecma = CRC::crc64ecma();

crc64ecma.digest(b"hello");

assert_eq!([236, 83, 136, 71, 154, 124, 145, 63].to_vec(), crc64ecma.get_crc_vec());
```

After getting a CRC value, you can still use the `digest` method to continue computing the next CRC values.

## Crates.io

https://crates.io/crates/crc-any

## Documentation

https://docs.rs/crc-any

## License

[MIT](LICENSE)