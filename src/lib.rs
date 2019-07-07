/*!
# CRC Any

To compute CRC values by providing the length of bits, expression, reflection, an initial value and a final xor value. It has many built-in CRC functions.

## Usage

You can use `create_crc` associated function to create a CRC instance by providing the length of bits, expression, reflection, an initial value and a final xor value. For example, if you want to compute a CRC-24 value.

```rust
extern crate crc_any;

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

For instance,

```rust
extern crate crc_any;

use crc_any::CRC;

let mut crc64 = CRC::crc64();

crc64.digest(b"hello");

assert_eq!([236, 83, 136, 71, 154, 124, 145, 63].to_vec(), crc64.get_crc_vec_be());
assert_eq!("0xEC5388479A7C913F", &crc64.to_string());
```

After getting a CRC value, you can still use the `digest` method to continue computing the next CRC values.

## No Std

Enable the **no_std** feature to compile this crate without std.

```toml
[dependencies.crc-any]
version = "*"
features = ["no_std"]
```
*/

#![cfg_attr(feature = "no_std", no_std)]

#[cfg(feature = "no_std")]
#[macro_use]
extern crate alloc;

mod crc_u8;
mod crc_u16;
mod crc_u32;
mod crc_u64;

#[cfg(not(feature = "no_std"))]
use std::fmt::{self, Formatter, Display};

#[cfg(feature = "no_std")]
use alloc::vec::Vec;

#[cfg(feature = "no_std")]
use alloc::fmt::{self, Formatter, Display};

pub use crc_u8::CRCu8;
pub use crc_u16::CRCu16;
pub use crc_u32::CRCu32;
pub use crc_u64::CRCu64;

/// This struct can help you compute a CRC value.
#[derive(Debug)]
pub enum CRC {
    CRCu8(CRCu8),
    CRCu16(CRCu16),
    CRCu32(CRCu32),
    CRCu64(CRCu64),
}

impl Display for CRC {
    #[inline]
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        match self {
            CRC::CRCu8(crc) => Display::fmt(crc, f),
            CRC::CRCu16(crc) => Display::fmt(crc, f),
            CRC::CRCu32(crc) => Display::fmt(crc, f),
            CRC::CRCu64(crc) => Display::fmt(crc, f),
        }
    }
}

impl CRC {
    /// Create a CRC instance by providing the length of bits, expression, reflection, an initial value and a final xor value.
    #[inline]
    pub fn create_crc(poly: u64, bits: u8, initial: u64, final_xor: u64, reflect: bool) -> CRC {
        if bits <= 8 {
            Self::create_crc_u8(poly as u8, bits, initial as u8, final_xor as u8, reflect)
        } else if bits <= 16 {
            Self::create_crc_u16(poly as u16, bits, initial as u16, final_xor as u16, reflect)
        } else if bits <= 32 {
            Self::create_crc_u32(poly as u32, bits, initial as u32, final_xor as u32, reflect)
        } else if bits <= 64 {
            Self::create_crc_u64(poly, bits, initial, final_xor, reflect)
        } else {
            unimplemented!()
        }
    }

    /// Create a CRC instance by providing the length of bits, expression, reflection, an initial value and a final xor value.
    #[inline]
    pub fn create_crc_u8(poly: u8, bits: u8, initial: u8, final_xor: u8, reflect: bool) -> CRC {
        let crc = CRCu8::create_crc(poly, bits, initial, final_xor, reflect);

        CRC::CRCu8(crc)
    }

    /// Create a CRC instance by providing the length of bits, expression, reflection, an initial value and a final xor value.
    #[inline]
    pub fn create_crc_u16(poly: u16, bits: u8, initial: u16, final_xor: u16, reflect: bool) -> CRC {
        let crc = CRCu16::create_crc(poly, bits, initial, final_xor, reflect);

        CRC::CRCu16(crc)
    }

    /// Create a CRC instance by providing the length of bits, expression, reflection, an initial value and a final xor value.
    #[inline]
    pub fn create_crc_u32(poly: u32, bits: u8, initial: u32, final_xor: u32, reflect: bool) -> CRC {
        let crc = CRCu32::create_crc(poly, bits, initial, final_xor, reflect);

        CRC::CRCu32(crc)
    }

    /// Create a CRC instance by providing the length of bits, expression, reflection, an initial value and a final xor value.
    #[inline]
    pub fn create_crc_u64(poly: u64, bits: u8, initial: u64, final_xor: u64, reflect: bool) -> CRC {
        let crc = CRCu64::create_crc(poly, bits, initial, final_xor, reflect);

        CRC::CRCu64(crc)
    }

    /// Digest some data.
    #[inline]
    pub fn digest<T: ?Sized + AsRef<[u8]>>(&mut self, data: &T) {
        match self {
            CRC::CRCu8(crc) => crc.digest(data),
            CRC::CRCu16(crc) => crc.digest(data),
            CRC::CRCu32(crc) => crc.digest(data),
            CRC::CRCu64(crc) => crc.digest(data),
        }
    }

    /// Reset the sum.
    #[inline]
    pub fn reset(&mut self) {
        match self {
            CRC::CRCu8(crc) => crc.reset(),
            CRC::CRCu16(crc) => crc.reset(),
            CRC::CRCu32(crc) => crc.reset(),
            CRC::CRCu64(crc) => crc.reset(),
        }
    }

    /// Get the current CRC value (it always returns a `u64` value). You can continue calling `digest` method even after getting a CRC value.
    #[inline]
    pub fn get_crc(&mut self) -> u64 {
        match self {
            CRC::CRCu8(crc) => crc.get_crc() as u64,
            CRC::CRCu16(crc) => crc.get_crc() as u64,
            CRC::CRCu32(crc) => crc.get_crc() as u64,
            CRC::CRCu64(crc) => crc.get_crc(),
        }
    }

    /// Get the current CRC value (it always returns a vec instance with a length corresponding to the CRC bits). You can continue calling `digest` method even after getting a CRC value.
    #[inline]
    pub fn get_crc_vec_le(&mut self) -> Vec<u8> {
        match self {
            CRC::CRCu8(crc) => {
                let mut vec = Vec::with_capacity(1);

                vec.push(crc.get_crc());

                vec
            }
            CRC::CRCu16(crc) => crc.get_crc_vec_le(),
            CRC::CRCu32(crc) => crc.get_crc_vec_le(),
            CRC::CRCu64(crc) => crc.get_crc_vec_le(),
        }
    }

    /// Get the current CRC value (it always returns a vec instance with a length corresponding to the CRC bits). You can continue calling `digest` method even after getting a CRC value.
    #[inline]
    pub fn get_crc_vec_be(&mut self) -> Vec<u8> {
        match self {
            CRC::CRCu8(crc) => {
                let mut vec = Vec::with_capacity(1);

                vec.push(crc.get_crc());

                vec
            }
            CRC::CRCu16(crc) => crc.get_crc_vec_be(),
            CRC::CRCu32(crc) => crc.get_crc_vec_be(),
            CRC::CRCu64(crc) => crc.get_crc_vec_be(),
        }
    }
}

impl CRC {
    // TODO: CRC-3

    pub fn crc3gsm() -> CRC {
        CRC::CRCu8(CRCu8::crc3gsm())
    }

    // TODO: CRC-4

    pub fn crc4itu() -> CRC {
        CRC::CRCu8(CRCu8::crc4itu())
    }

    pub fn crc4interlaken() -> CRC {
        CRC::CRCu8(CRCu8::crc4interlaken())
    }

    // TODO: CRC-5

    pub fn crc5epc() -> CRC {
        CRC::CRCu8(CRCu8::crc5epc())
    }

    pub fn crc5itu() -> CRC {
        CRC::CRCu8(CRCu8::crc5itu())
    }

    pub fn crc5usb() -> CRC {
        CRC::CRCu8(CRCu8::crc5usb())
    }

    // TODO: CRC-6

    #[inline]
    pub fn crc6cdma2000_a() -> CRC {
        CRC::CRCu8(CRCu8::crc6cdma2000_a())
    }

    #[inline]
    pub fn crc6cdma2000_b() -> CRC {
        CRC::CRCu8(CRCu8::crc6cdma2000_b())
    }

    #[inline]
    pub fn crc6darc() -> CRC {
        CRC::CRCu8(CRCu8::crc6darc())
    }

    #[inline]
    pub fn crc6gsm() -> CRC {
        CRC::CRCu8(CRCu8::crc6gsm())
    }

    #[inline]
    pub fn crc6itu() -> CRC {
        CRC::CRCu8(CRCu8::crc6itu())
    }

    // TODO: CRC-7

    #[inline]
    pub fn crc7() -> CRC {
        CRC::CRCu8(CRCu8::crc7())
    }

    #[inline]
    pub fn crc7umts() -> CRC {
        CRC::CRCu8(CRCu8::crc7umts())
    }

    // TODO: CRC-8

    #[inline]
    pub fn crc8() -> CRC {
        CRC::CRCu8(CRCu8::crc8())
    }

    #[inline]
    pub fn crc8cdma2000() -> CRC {
        CRC::CRCu8(CRCu8::crc8cdma2000())
    }

    #[inline]
    pub fn crc8darc() -> CRC {
        CRC::CRCu8(CRCu8::crc8darc())
    }

    #[inline]
    pub fn crc8dvb_s2() -> CRC {
        CRC::CRCu8(CRCu8::crc8dvb_s2())
    }

    #[inline]
    pub fn crc8ebu() -> CRC {
        CRC::CRCu8(CRCu8::crc8ebu())
    }

    #[inline]
    pub fn crc8icode() -> CRC {
        CRC::CRCu8(CRCu8::crc8icode())
    }

    #[inline]
    pub fn crc8itu() -> CRC {
        CRC::CRCu8(CRCu8::crc8itu())
    }

    #[inline]
    pub fn crc8maxim() -> CRC {
        CRC::CRCu8(CRCu8::crc8maxim())
    }

    #[inline]
    pub fn crc8rohc() -> CRC {
        CRC::CRCu8(CRCu8::crc8rohc())
    }

    #[inline]
    pub fn crc8wcdma() -> CRC {
        CRC::CRCu8(CRCu8::crc8wcdma())
    }

    // TODO: CRC-10

    #[inline]
    pub fn crc10() -> CRC {
        CRC::CRCu16(CRCu16::crc10())
    }

    #[inline]
    pub fn crc10cdma2000() -> CRC {
        CRC::CRCu16(CRCu16::crc10cdma2000())
    }

    #[inline]
    pub fn crc10gsm() -> CRC {
        CRC::CRCu16(CRCu16::crc10gsm())
    }

    // TODO: CRC-11

    #[inline]
    pub fn crc11() -> CRC {
        CRC::CRCu16(CRCu16::crc11())
    }

    // TODO: CRC-12

    #[inline]
    pub fn crc12() -> CRC {
        CRC::CRCu16(CRCu16::crc12())
    }

    #[inline]
    pub fn crc12cdma2000() -> CRC {
        CRC::CRCu16(CRCu16::crc12cdma2000())
    }

    #[inline]
    pub fn crc12gsm() -> CRC {
        CRC::CRCu16(CRCu16::crc12gsm())
    }

    // TODO: CRC-13

    #[inline]
    pub fn crc13bbc() -> CRC {
        CRC::CRCu16(CRCu16::crc13bbc())
    }

    // TODO: CRC-14

    #[inline]
    pub fn crc14darc() -> CRC {
        CRC::CRCu16(CRCu16::crc14darc())
    }

    #[inline]
    pub fn crc14gsm() -> CRC {
        CRC::CRCu16(CRCu16::crc14gsm())
    }

    // TODO: CRC-15

    #[inline]
    pub fn crc15can() -> CRC {
        CRC::CRCu16(CRCu16::crc15can())
    }

    #[inline]
    pub fn crc15mpt1327() -> CRC {
        CRC::CRCu16(CRCu16::crc15mpt1327())
    }

    // TODO: CRC-16

    #[inline]
    pub fn crc16() -> CRC {
        CRC::CRCu16(CRCu16::crc16())
    }

    #[inline]
    pub fn crc16ccitt_false() -> CRC {
        CRC::CRCu16(CRCu16::crc16ccitt_false())
    }

    #[inline]
    pub fn crc16aug_ccitt() -> CRC {
        CRC::CRCu16(CRCu16::crc16aug_ccitt())
    }

    #[inline]
    pub fn crc16buypass() -> CRC {
        CRC::CRCu16(CRCu16::crc16buypass())
    }

    #[inline]
    pub fn crc16cdma2000() -> CRC {
        CRC::CRCu16(CRCu16::crc16cdma2000())
    }

    #[inline]
    pub fn crc16dds_110() -> CRC {
        CRC::CRCu16(CRCu16::crc16dds_110())
    }

    #[inline]
    pub fn crc16dect_r() -> CRC {
        CRC::CRCu16(CRCu16::crc16dect_r())
    }

    #[inline]
    pub fn crc16dect_x() -> CRC {
        CRC::CRCu16(CRCu16::crc16dect_x())
    }

    #[inline]
    pub fn crc16dnp() -> CRC {
        CRC::CRCu16(CRCu16::crc16dnp())
    }

    #[inline]
    pub fn crc16en_13757() -> CRC {
        CRC::CRCu16(CRCu16::crc16en_13757())
    }

    #[inline]
    pub fn crc16genibus() -> CRC {
        CRC::CRCu16(CRCu16::crc16genibus())
    }

    #[inline]
    pub fn crc16maxim() -> CRC {
        CRC::CRCu16(CRCu16::crc16maxim())
    }

    #[inline]
    pub fn crc16mcrf4cc() -> CRC {
        CRC::CRCu16(CRCu16::crc16mcrf4cc())
    }

    #[inline]
    pub fn crc16riello() -> CRC {
        CRC::CRCu16(CRCu16::crc16riello())
    }

    #[inline]
    pub fn crc16t10_dif() -> CRC {
        CRC::CRCu16(CRCu16::crc16t10_dif())
    }

    #[inline]
    pub fn crc16teledisk() -> CRC {
        CRC::CRCu16(CRCu16::crc16teledisk())
    }

    #[inline]
    pub fn crc16tms13157() -> CRC {
        CRC::CRCu16(CRCu16::crc16tms13157())
    }

    #[inline]
    pub fn crc16usb() -> CRC {
        CRC::CRCu16(CRCu16::crc16usb())
    }

    #[inline]
    pub fn crc_a() -> CRC {
        CRC::CRCu16(CRCu16::crc_a())
    }

    #[inline]
    pub fn crc16kermit() -> CRC {
        CRC::CRCu16(CRCu16::crc16kermit())
    }

    #[inline]
    pub fn crc16modbus() -> CRC {
        CRC::CRCu16(CRCu16::crc16modbus())
    }

    #[inline]
    pub fn crc16_x25() -> CRC {
        CRC::CRCu16(CRCu16::crc16_x25())
    }

    #[inline]
    pub fn crc16xmodem() -> CRC {
        CRC::CRCu16(CRCu16::crc16xmodem())
    }

    // TODO: CRC-17

    #[inline]
    pub fn crc17can() -> CRC {
        CRC::CRCu32(CRCu32::crc17can())
    }

    // TODO: CRC-21

    #[inline]
    pub fn crc21can() -> CRC {
        CRC::CRCu32(CRCu32::crc21can())
    }

    // TODO: CRC-24

    #[inline]
    pub fn crc24() -> CRC {
        CRC::CRCu32(CRCu32::crc24())
    }

    #[inline]
    pub fn crc24ble() -> CRC {
        CRC::CRCu32(CRCu32::crc24ble())
    }

    #[inline]
    pub fn crc24flexray_a() -> CRC {
        CRC::CRCu32(CRCu32::crc24flexray_a())
    }

    #[inline]
    pub fn crc24flexray_b() -> CRC {
        CRC::CRCu32(CRCu32::crc24flexray_b())
    }

    #[inline]
    pub fn crc24lte_a() -> CRC {
        CRC::CRCu32(CRCu32::crc24lte_a())
    }

    #[inline]
    pub fn crc24lte_b() -> CRC {
        CRC::CRCu32(CRCu32::crc24lte_b())
    }

    #[inline]
    pub fn crc24os9() -> CRC {
        CRC::CRCu32(CRCu32::crc24os9())
    }

    // TODO: CRC-30

    #[inline]
    pub fn crc30cdma() -> CRC {
        CRC::CRCu32(CRCu32::crc30cdma())
    }

    // TODO: CRC-32

    #[inline]
    pub fn crc32() -> CRC {
        CRC::CRCu32(CRCu32::crc32())
    }

    #[inline]
    pub fn crc32mhash() -> CRC {
        CRC::CRCu32(CRCu32::crc32mhash())
    }

    #[inline]
    pub fn crc32bzip2() -> CRC {
        CRC::CRCu32(CRCu32::crc32bzip2())
    }

    #[inline]
    pub fn crc32c() -> CRC {
        CRC::CRCu32(CRCu32::crc32c())
    }

    #[inline]
    pub fn crc32d() -> CRC {
        CRC::CRCu32(CRCu32::crc32d())
    }

    #[inline]
    pub fn crc32mpeg2() -> CRC {
        CRC::CRCu32(CRCu32::crc32mpeg2())
    }

    #[inline]
    pub fn crc32posix() -> CRC {
        CRC::CRCu32(CRCu32::crc32posix())
    }

    #[inline]
    pub fn crc32q() -> CRC {
        CRC::CRCu32(CRCu32::crc32q())
    }

    #[inline]
    pub fn crc32jamcrc() -> CRC {
        CRC::CRCu32(CRCu32::crc32jamcrc())
    }

    #[inline]
    pub fn crc32xfer() -> CRC {
        CRC::CRCu32(CRCu32::crc32xfer())
    }

    // TODO: CRC-40

    #[inline]
    pub fn crc40gsm() -> CRC {
        CRC::CRCu64(CRCu64::crc40gsm())
    }

    // TODO: CRC-64

    #[inline]
    pub fn crc64() -> CRC {
        CRC::CRCu64(CRCu64::crc64())
    }

    #[inline]
    pub fn crc64iso() -> CRC {
        CRC::CRCu64(CRCu64::crc64iso())
    }
}