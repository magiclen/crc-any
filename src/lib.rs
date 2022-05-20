/*!
# CRC Any

To compute CRC values by providing the length of bits, expression, reflection, an initial value and a final xor value. It has many built-in CRC functions.

## Usage

You can use `create_crc` associated function to create a CRC instance by providing the length of bits, expression, reflection, an initial value and a final xor value. For example, if you want to compute a CRC-24 value.

```rust
use crc_any::CRC;

let mut crc24 = CRC::create_crc(0x0000000000864CFB, 24, 0x0000000000B704CE, 0x0000000000000000, false);

crc24.digest(b"hello");
*/
#![cfg_attr(
    feature = "alloc",
    doc = "

assert_eq!([71, 245, 138].to_vec(), crc24.get_crc_vec_be());
assert_eq!(\"0x47F58A\", &crc24.to_string());
"
)]
/*!
```
 */
/*!
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
*/
#![cfg_attr(
    feature = "alloc",
    doc = "

assert_eq!([64, 84, 74, 48, 97, 55, 182, 236].to_vec(), crc64.get_crc_vec_be());
assert_eq!(\"0x40544A306137B6EC\", &crc64.to_string());
"
)]
/*!
```
*/
/*!
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
 */

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "alloc")]
#[macro_use]
extern crate alloc;

#[cfg(feature = "alloc")]
use alloc::fmt::{self, Display, Formatter};
#[cfg(feature = "alloc")]
use alloc::vec::Vec;

#[cfg(feature = "heapless")]
use heapless::Vec as HeaplessVec;

mod constants;
mod crc_u16;
mod crc_u32;
mod crc_u64;
mod crc_u8;
mod lookup_table;

pub use crc_u16::CRCu16;
pub use crc_u32::CRCu32;
pub use crc_u64::CRCu64;
pub use crc_u8::CRCu8;

#[allow(clippy::upper_case_acronyms, clippy::large_enum_variant)]
/// This struct can help you compute a CRC value.
#[cfg_attr(feature = "alloc", derive(Debug))]
pub enum CRC {
    CRCu8(CRCu8),
    CRCu16(CRCu16),
    CRCu32(CRCu32),
    CRCu64(CRCu64),
}

#[cfg(feature = "alloc")]
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
            CRC::CRCu8(crc) => u64::from(crc.get_crc()),
            CRC::CRCu16(crc) => u64::from(crc.get_crc()),
            CRC::CRCu32(crc) => u64::from(crc.get_crc()),
            CRC::CRCu64(crc) => crc.get_crc(),
        }
    }
}

#[cfg(feature = "alloc")]
impl CRC {
    /// Get the current CRC value (it always returns a vec instance with a length corresponding to the CRC bits). You can continue calling `digest` method even after getting a CRC value.
    #[inline]
    pub fn get_crc_vec_le(&mut self) -> Vec<u8> {
        match self {
            CRC::CRCu8(crc) => vec![crc.get_crc()],
            CRC::CRCu16(crc) => crc.get_crc_vec_le(),
            CRC::CRCu32(crc) => crc.get_crc_vec_le(),
            CRC::CRCu64(crc) => crc.get_crc_vec_le(),
        }
    }

    /// Get the current CRC value (it always returns a vec instance with a length corresponding to the CRC bits). You can continue calling `digest` method even after getting a CRC value.
    #[inline]
    pub fn get_crc_vec_be(&mut self) -> Vec<u8> {
        match self {
            CRC::CRCu8(crc) => vec![crc.get_crc()],
            CRC::CRCu16(crc) => crc.get_crc_vec_be(),
            CRC::CRCu32(crc) => crc.get_crc_vec_be(),
            CRC::CRCu64(crc) => crc.get_crc_vec_be(),
        }
    }
}

#[cfg(feature = "heapless")]
impl CRC {
    /// Get the current CRC value (it always returns a vec instance with a length corresponding to the CRC bits). You can continue calling `digest` method even after getting a CRC value.
    pub fn get_crc_heapless_vec_le(&mut self) -> HeaplessVec<u8, 8> {
        let mut vec = HeaplessVec::new();

        let bits = match self {
            CRC::CRCu8(crc) => f64::from(crc.bits),
            CRC::CRCu16(crc) => f64::from(crc.bits),
            CRC::CRCu32(crc) => f64::from(crc.bits),
            CRC::CRCu64(crc) => f64::from(crc.bits),
        };

        let e = ((bits + 7f64) / 8f64) as u64;

        let e_dec = e - 1;

        let o = e_dec * 8;

        let crc = self.get_crc();

        for i in 0..e {
            vec.push((crc << ((e_dec - i) * 8) >> o) as u8).unwrap();
        }

        vec
    }

    /// Get the current CRC value (it always returns a vec instance with a length corresponding to the CRC bits). You can continue calling `digest` method even after getting a CRC value.
    pub fn get_crc_heapless_vec_be(&mut self) -> HeaplessVec<u8, 8> {
        let mut vec = HeaplessVec::new();

        let bits = match self {
            CRC::CRCu8(crc) => f64::from(crc.bits),
            CRC::CRCu16(crc) => f64::from(crc.bits),
            CRC::CRCu32(crc) => f64::from(crc.bits),
            CRC::CRCu64(crc) => f64::from(crc.bits),
        };

        let e = ((bits + 7f64) / 8f64) as u64;

        let e_dec = e - 1;

        let o = e_dec * 8;

        let crc = self.get_crc();

        for i in 0..e {
            vec.push((crc << (i * 8) >> o) as u8).unwrap();
        }

        vec
    }
}

impl CRC {
    // TODO: CRC-3

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0x4|0x3|0x0|false|0x7|
    ///
    /// ```
    /// # use crc_any::CRC;
    /// let mut crc = CRC::crc3gsm();
    /// crc.digest(b"123456789");
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0x4\", &crc.to_string());")]
    /// ```
    pub fn crc3gsm() -> CRC {
        CRC::CRCu8(CRCu8::crc3gsm())
    }

    // TODO: CRC-4

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0x7|0x3 (rev: 0xC)|0x0|true|0x0|
    ///
    /// ```
    /// # use crc_any::CRC;
    /// let mut crc = CRC::crc4itu();
    /// crc.digest(b"123456789");
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0x7\", &crc.to_string());")]
    /// ```
    pub fn crc4itu() -> CRC {
        CRC::CRCu8(CRCu8::crc4itu())
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0xB|0x3|0xF|false|0xF|
    ///
    /// ```
    /// # use crc_any::CRC;
    /// let mut crc = CRC::crc4interlaken();
    /// crc.digest(b"123456789");
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0xB\", &crc.to_string());")]
    /// ```
    pub fn crc4interlaken() -> CRC {
        CRC::CRCu8(CRCu8::crc4interlaken())
    }

    // TODO: CRC-5

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0x00|0x09|0x09|false|0x00|
    ///
    /// ```
    /// # use crc_any::CRC;
    /// let mut crc = CRC::crc5epc();
    /// crc.digest(b"123456789");
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0x00\", &crc.to_string());")]
    /// ```
    pub fn crc5epc() -> CRC {
        CRC::CRCu8(CRCu8::crc5epc())
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0x07|0x15 (rev: 0x15)|0x00|true|0x00|
    ///
    /// ```
    /// # use crc_any::CRC;
    /// let mut crc = CRC::crc5itu();
    /// crc.digest(b"123456789");
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0x07\", &crc.to_string());")]
    /// ```
    pub fn crc5itu() -> CRC {
        CRC::CRCu8(CRCu8::crc5itu())
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0x19|0x05 (rev: 0x14)|0x1F|true|0x1F|
    ///
    /// ```
    /// # use crc_any::CRC;
    /// let mut crc = CRC::crc5usb();
    /// crc.digest(b"123456789");
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0x19\", &crc.to_string());")]
    /// ```
    pub fn crc5usb() -> CRC {
        CRC::CRCu8(CRCu8::crc5usb())
    }

    // TODO: CRC-6

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0x0D|0x27|0x3F|false|0x00|
    ///
    /// ```
    /// # use crc_any::CRC;
    /// let mut crc = CRC::crc6cdma2000_a();
    /// crc.digest(b"123456789");
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0x0D\", &crc.to_string());")]
    /// ```
    #[inline]
    pub fn crc6cdma2000_a() -> CRC {
        CRC::CRCu8(CRCu8::crc6cdma2000_a())
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0x3B|0x07|0x3F|false|0x00|
    ///
    /// ```
    /// # use crc_any::CRC;
    /// let mut crc = CRC::crc6cdma2000_b();
    /// crc.digest(b"123456789");
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0x3B\", &crc.to_string());")]
    /// ```
    #[inline]
    pub fn crc6cdma2000_b() -> CRC {
        CRC::CRCu8(CRCu8::crc6cdma2000_b())
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0x26|0x19 (rev: 0x26)|0x00|true|0x00|
    ///
    /// ```
    /// # use crc_any::CRC;
    /// let mut crc = CRC::crc6darc();
    /// crc.digest(b"123456789");
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0x26\", &crc.to_string());")]
    /// ```
    #[inline]
    pub fn crc6darc() -> CRC {
        CRC::CRCu8(CRCu8::crc6darc())
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0x13|0x2F|0x00|false|0x3F|
    ///
    /// ```
    /// # use crc_any::CRC;
    /// let mut crc = CRC::crc6gsm();
    /// crc.digest(b"123456789");
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0x13\", &crc.to_string());")]
    /// ```
    #[inline]
    pub fn crc6gsm() -> CRC {
        CRC::CRCu8(CRCu8::crc6gsm())
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0x06|0x03 (rev: 0x30)|0x00|true|0x00|
    ///
    /// ```
    /// # use crc_any::CRC;
    /// let mut crc = CRC::crc6itu();
    /// crc.digest(b"123456789");
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0x06\", &crc.to_string());")]
    /// ```
    #[inline]
    pub fn crc6itu() -> CRC {
        CRC::CRCu8(CRCu8::crc6itu())
    }

    // TODO: CRC-7

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0x75|0x09|0x00|false|0x00|
    ///
    /// ```
    /// # use crc_any::CRC;
    /// let mut crc = CRC::crc7();
    /// crc.digest(b"123456789");
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0x75\", &crc.to_string());")]
    /// ```
    #[inline]
    pub fn crc7() -> CRC {
        CRC::CRCu8(CRCu8::crc7())
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0x61|0x45|0x00|false|0x00|
    ///
    /// ```
    /// # use crc_any::CRC;
    /// let mut crc = CRC::crc7umts();
    /// crc.digest(b"123456789");
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0x61\", &crc.to_string());")]
    /// ```
    #[inline]
    pub fn crc7umts() -> CRC {
        CRC::CRCu8(CRCu8::crc7umts())
    }

    // TODO: CRC-8

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0xF4|0x07|0x00|false|0x00|
    ///
    /// ```
    /// # use crc_any::CRC;
    /// let mut crc = CRC::crc8();
    /// crc.digest(b"123456789");
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0xF4\", &crc.to_string());")]
    /// ```
    #[inline]
    pub fn crc8() -> CRC {
        CRC::CRCu8(CRCu8::crc8())
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0xDA|0x9B|0xFF|false|0x00|
    ///
    /// ```
    /// # use crc_any::CRC;
    /// let mut crc = CRC::crc8cdma2000();
    /// crc.digest(b"123456789");
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0xDA\", &crc.to_string());")]
    /// ```
    #[inline]
    pub fn crc8cdma2000() -> CRC {
        CRC::CRCu8(CRCu8::crc8cdma2000())
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0xDA|0x39 (rev: 0x9C)|0x00|true|0x00|
    ///
    /// ```
    /// # use crc_any::CRC;
    /// let mut crc = CRC::crc8darc();
    /// crc.digest(b"123456789");
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0x15\", &crc.to_string());")]
    /// ```
    #[inline]
    pub fn crc8darc() -> CRC {
        CRC::CRCu8(CRCu8::crc8darc())
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0xBC|0xD5|0x00|false|0x00|
    ///
    /// ```
    /// # use crc_any::CRC;
    /// let mut crc = CRC::crc8dvb_s2();
    /// crc.digest(b"123456789");
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0xBC\", &crc.to_string());")]
    /// ```
    #[inline]
    pub fn crc8dvb_s2() -> CRC {
        CRC::CRCu8(CRCu8::crc8dvb_s2())
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0x97|0x1D (rev: 0xB8)|0xFF|true|0x00|
    ///
    /// ```
    /// # use crc_any::CRC;
    /// let mut crc = CRC::crc8ebu();
    /// crc.digest(b"123456789");
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0x97\", &crc.to_string());")]
    /// ```
    #[inline]
    pub fn crc8ebu() -> CRC {
        CRC::CRCu8(CRCu8::crc8ebu())
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0x7E|0x1D|0xFD|false|0x00|
    ///
    /// ```
    /// # use crc_any::CRC;
    /// let mut crc = CRC::crc8icode();
    /// crc.digest(b"123456789");
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0x7E\", &crc.to_string());")]
    /// ```
    #[inline]
    pub fn crc8icode() -> CRC {
        CRC::CRCu8(CRCu8::crc8icode())
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0xA1|0x07|0x00|false|0x55|
    ///
    /// ```
    /// # use crc_any::CRC;
    /// let mut crc = CRC::crc8itu();
    /// crc.digest(b"123456789");
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0xA1\", &crc.to_string());")]
    /// ```
    #[inline]
    pub fn crc8itu() -> CRC {
        CRC::CRCu8(CRCu8::crc8itu())
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0xA1|0x31 (rev: 0x8C)|0x00|true|0x00|
    ///
    /// ```
    /// # use crc_any::CRC;
    /// let mut crc = CRC::crc8maxim();
    /// crc.digest(b"123456789");
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0xA1\", &crc.to_string());")]
    /// ```
    #[inline]
    pub fn crc8maxim() -> CRC {
        CRC::CRCu8(CRCu8::crc8maxim())
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0xD0|0x07 (rev: 0xE0)|0xFF|true|0x00|
    ///
    /// ```
    /// # use crc_any::CRC;
    /// let mut crc = CRC::crc8rohc();
    /// crc.digest(b"123456789");
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0xD0\", &crc.to_string());")]
    /// ```
    #[inline]
    pub fn crc8rohc() -> CRC {
        CRC::CRCu8(CRCu8::crc8rohc())
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0x25|0x9B (rev: 0xD9)|0x00|true|0x00|
    ///
    /// ```
    /// # use crc_any::CRC;
    /// let mut crc = CRC::crc8wcdma();
    /// crc.digest(b"123456789");
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0x25\", &crc.to_string());")]
    /// ```
    #[inline]
    pub fn crc8wcdma() -> CRC {
        CRC::CRCu8(CRCu8::crc8wcdma())
    }

    // TODO: CRC-10

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0x199|0x233|0x000|false|0x000|
    ///
    /// ```
    /// # use crc_any::CRC;
    /// let mut crc = CRC::crc10();
    /// crc.digest(b"123456789");
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0x199\", &crc.to_string());")]
    /// ```
    #[inline]
    pub fn crc10() -> CRC {
        CRC::CRCu16(CRCu16::crc10())
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0x233|0x3D9|0x3FF|false|0x000|
    ///
    /// ```
    /// # use crc_any::CRC;
    /// let mut crc = CRC::crc10cdma2000();
    /// crc.digest(b"123456789");
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0x233\", &crc.to_string());")]
    /// ```
    #[inline]
    pub fn crc10cdma2000() -> CRC {
        CRC::CRCu16(CRCu16::crc10cdma2000())
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0x12A|0x175|0x000|false|0x3FF|
    ///
    /// ```
    /// # use crc_any::CRC;
    /// let mut crc = CRC::crc10gsm();
    /// crc.digest(b"123456789");
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0x12A\", &crc.to_string());")]
    /// ```
    #[inline]
    pub fn crc10gsm() -> CRC {
        CRC::CRCu16(CRCu16::crc10gsm())
    }

    // TODO: CRC-11

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0x5A3|0x385|0x01a|false|0x000|
    ///
    /// ```
    /// # use crc_any::CRC;
    /// let mut crc = CRC::crc11();
    /// crc.digest(b"123456789");
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0x5A3\", &crc.to_string());")]
    /// ```
    #[inline]
    pub fn crc11() -> CRC {
        CRC::CRCu16(CRCu16::crc11())
    }

    // TODO: CRC-12

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0xF5B|0x080F|0x0000|false|0x0000|
    ///
    /// ```
    /// # use crc_any::CRC;
    /// let mut crc = CRC::crc12();
    /// crc.digest(b"123456789");
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0xF5B\", &crc.to_string());")]
    /// ```
    #[inline]
    pub fn crc12() -> CRC {
        CRC::CRCu16(CRCu16::crc12())
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0xD4D|0x0F13|0x0FFF|false|0x0000|
    ///
    /// ```
    /// # use crc_any::CRC;
    /// let mut crc = CRC::crc12cdma2000();
    /// crc.digest(b"123456789");
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0xD4D\", &crc.to_string());")]
    /// ```
    #[inline]
    pub fn crc12cdma2000() -> CRC {
        CRC::CRCu16(CRCu16::crc12cdma2000())
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0xB34|0x0D31|0x0000|false|0x0FFF|
    ///
    /// ```
    /// # use crc_any::CRC;
    /// let mut crc = CRC::crc12gsm();
    /// crc.digest(b"123456789");
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0xB34\", &crc.to_string());")]
    /// ```
    #[inline]
    pub fn crc12gsm() -> CRC {
        CRC::CRCu16(CRCu16::crc12gsm())
    }

    // TODO: CRC-13

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0x04FA|0x1CF5|0x0000|false|0x0000|
    ///
    /// ```
    /// # use crc_any::CRC;
    /// let mut crc = CRC::crc13bbc();
    /// crc.digest(b"123456789");
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0x04FA\", &crc.to_string());")]
    /// ```
    #[inline]
    pub fn crc13bbc() -> CRC {
        CRC::CRCu16(CRCu16::crc13bbc())
    }

    // TODO: CRC-14

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0x082D|0x0805 (rev: 0x2804)|0x0000|true|0x0000|
    ///
    /// ```
    /// # use crc_any::CRC;
    /// let mut crc = CRC::crc14darc();
    /// crc.digest(b"123456789");
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0x082D\", &crc.to_string());")]
    /// ```
    #[inline]
    pub fn crc14darc() -> CRC {
        CRC::CRCu16(CRCu16::crc14darc())
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0x30AE|0x202D|0x0000|false|0x3FFF|
    ///
    /// ```
    /// # use crc_any::CRC;
    /// let mut crc = CRC::crc14gsm();
    /// crc.digest(b"123456789");
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0x30AE\", &crc.to_string());")]
    /// ```
    #[inline]
    pub fn crc14gsm() -> CRC {
        CRC::CRCu16(CRCu16::crc14gsm())
    }

    // TODO: CRC-15

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0x059E|0x4599|0x0000|false|0x0000|
    ///
    /// ```
    /// # use crc_any::CRC;
    /// let mut crc = CRC::crc15can();
    /// crc.digest(b"123456789");
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0x059E\", &crc.to_string());")]
    /// ```
    #[inline]
    pub fn crc15can() -> CRC {
        CRC::CRCu16(CRCu16::crc15can())
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0x2566|0x6815|0x0000|false|0x0001|
    ///
    /// ```
    /// # use crc_any::CRC;
    /// let mut crc = CRC::crc15mpt1327();
    /// crc.digest(b"123456789");
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0x2566\", &crc.to_string());")]
    /// ```
    #[inline]
    pub fn crc15mpt1327() -> CRC {
        CRC::CRCu16(CRCu16::crc15mpt1327())
    }

    // TODO: CRC-16

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0xBB3D|0x8005 (rev: 0xA001)|0x0000|true|0x0000|
    ///
    /// ```
    /// # use crc_any::CRC;
    /// let mut crc = CRC::crc16();
    /// crc.digest(b"123456789");
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0xBB3D\", &crc.to_string());")]
    /// ```
    #[inline]
    pub fn crc16() -> CRC {
        CRC::CRCu16(CRCu16::crc16())
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0x29B1|0x1021|0xFFFF|false|0x0000|
    ///
    /// ```
    /// # use crc_any::CRC;
    /// let mut crc = CRC::crc16ccitt_false();
    /// crc.digest(b"123456789");
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0x29B1\", &crc.to_string());")]
    /// ```
    #[inline]
    pub fn crc16ccitt_false() -> CRC {
        CRC::CRCu16(CRCu16::crc16ccitt_false())
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0xE5CC|0x1021|0x1D0F|false|0x0000|
    ///
    /// ```
    /// # use crc_any::CRC;
    /// let mut crc = CRC::crc16aug_ccitt();
    /// crc.digest(b"123456789");
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0xE5CC\", &crc.to_string());")]
    /// ```
    #[inline]
    pub fn crc16aug_ccitt() -> CRC {
        CRC::CRCu16(CRCu16::crc16aug_ccitt())
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0xFEE8|0x8005|0x0000|false|0x0000|
    ///
    /// ```
    /// # use crc_any::CRC;
    /// let mut crc = CRC::crc16buypass();
    /// crc.digest(b"123456789");
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0xFEE8\", &crc.to_string());")]
    /// ```
    #[inline]
    pub fn crc16buypass() -> CRC {
        CRC::CRCu16(CRCu16::crc16buypass())
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0x4C06|0xC867|0xFFFF|false|0x0000|
    ///
    /// ```
    /// # use crc_any::CRC;
    /// let mut crc = CRC::crc16cdma2000();
    /// crc.digest(b"123456789");
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0x4C06\", &crc.to_string());")]
    /// ```
    #[inline]
    pub fn crc16cdma2000() -> CRC {
        CRC::CRCu16(CRCu16::crc16cdma2000())
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0x9ECF|0x8005|0x800D|false|0x0000|
    ///
    /// ```
    /// # use crc_any::CRC;
    /// let mut crc = CRC::crc16dds_110();
    /// crc.digest(b"123456789");
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0x9ECF\", &crc.to_string());")]
    /// ```
    #[inline]
    pub fn crc16dds_110() -> CRC {
        CRC::CRCu16(CRCu16::crc16dds_110())
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0x007E|0x0589|0x0000|false|0x0001|
    ///
    /// ```
    /// # use crc_any::CRC;
    /// let mut crc = CRC::crc16dect_r();
    /// crc.digest(b"123456789");
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0x007E\", &crc.to_string());")]
    /// ```
    #[inline]
    pub fn crc16dect_r() -> CRC {
        CRC::CRCu16(CRCu16::crc16dect_r())
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0x007F|0x0589|0x0000|false|0x0000|
    ///
    /// ```
    /// # use crc_any::CRC;
    /// let mut crc = CRC::crc16dect_r();
    /// crc.digest(b"123456789");
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0x007E\", &crc.to_string());")]
    /// ```
    #[inline]
    pub fn crc16dect_x() -> CRC {
        CRC::CRCu16(CRCu16::crc16dect_x())
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0xEA82|0x3D65 (rev: 0xA6BC)|0x0000|true|0xFFFF|
    ///
    /// ```
    /// # use crc_any::CRC;
    /// let mut crc = CRC::crc16dnp();
    /// crc.digest(b"123456789");
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0xEA82\", &crc.to_string());")]
    /// ```
    #[inline]
    pub fn crc16dnp() -> CRC {
        CRC::CRCu16(CRCu16::crc16dnp())
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0xC2B7|0x3D65|0x0000|false|0xFFFF|
    ///
    /// ```
    /// # use crc_any::CRC;
    /// let mut crc = CRC::crc16en_13757();
    /// crc.digest(b"123456789");
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0xC2B7\", &crc.to_string());")]
    /// ```
    #[inline]
    pub fn crc16en_13757() -> CRC {
        CRC::CRCu16(CRCu16::crc16en_13757())
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0xD64E|0x1021|0xFFFF|false|0xFFFF|
    ///
    /// ```
    /// # use crc_any::CRC;
    /// let mut crc = CRC::crc16genibus();
    /// crc.digest(b"123456789");
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0xD64E\", &crc.to_string());")]
    /// ```
    #[inline]
    pub fn crc16genibus() -> CRC {
        CRC::CRCu16(CRCu16::crc16genibus())
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0x44C2|0x8005 (rev: 0xA001)|0xFFFF|true|0xFFFF|
    ///
    /// ```
    /// # use crc_any::CRC;
    /// let mut crc = CRC::crc16maxim();
    /// crc.digest(b"123456789");
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0x44C2\", &crc.to_string());")]
    /// ```
    #[inline]
    pub fn crc16maxim() -> CRC {
        CRC::CRCu16(CRCu16::crc16maxim())
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0x6F91|0x1021 (rev: 0x8408)|0xFFFF|true|0x0000|
    ///
    /// ```
    /// # use crc_any::CRC;
    /// let mut crc = CRC::crc16mcrf4cc();
    /// crc.digest(b"123456789");
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0x6F91\", &crc.to_string());")]
    /// ```
    #[inline]
    pub fn crc16mcrf4cc() -> CRC {
        CRC::CRCu16(CRCu16::crc16mcrf4cc())
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0x63D0|0x1021 (rev: 0x8408)|0xB2AA|true|0x0000|
    ///
    /// ```
    /// # use crc_any::CRC;
    /// let mut crc = CRC::crc16riello();
    /// crc.digest(b"123456789");
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0x63D0\", &crc.to_string());")]
    /// ```
    #[inline]
    pub fn crc16riello() -> CRC {
        CRC::CRCu16(CRCu16::crc16riello())
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0xD0DB|0x8BB7|0x0000|false|0x0000|
    ///
    /// ```
    /// # use crc_any::CRC;
    /// let mut crc = CRC::crc16t10_dif();
    /// crc.digest(b"123456789");
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0xD0DB\", &crc.to_string());")]
    /// ```
    #[inline]
    pub fn crc16t10_dif() -> CRC {
        CRC::CRCu16(CRCu16::crc16t10_dif())
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0x0FB3|0xA097|0x0000|false|0x0000|
    ///
    /// ```
    /// # use crc_any::CRC;
    /// let mut crc = CRC::crc16teledisk();
    /// crc.digest(b"123456789");
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0x0FB3\", &crc.to_string());")]
    /// ```
    #[inline]
    pub fn crc16teledisk() -> CRC {
        CRC::CRCu16(CRCu16::crc16teledisk())
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0x26B1|0x1021 (rev: 0x8408)|0x89EC|true|0x0000|
    ///
    /// ```
    /// # use crc_any::CRC;
    /// let mut crc = CRC::crc16tms13157();
    /// crc.digest(b"123456789");
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0x26B1\", &crc.to_string());")]
    /// ```
    #[inline]
    pub fn crc16tms13157() -> CRC {
        CRC::CRCu16(CRCu16::crc16tms13157())
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0xB4C8|0x8005 (rev: 0xA001)|0xFFFF|true|0xFFFF|
    ///
    /// ```
    /// # use crc_any::CRC;
    /// let mut crc = CRC::crc16usb();
    /// crc.digest(b"123456789");
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0xB4C8\", &crc.to_string());")]
    /// ```
    #[inline]
    pub fn crc16usb() -> CRC {
        CRC::CRCu16(CRCu16::crc16usb())
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0xBF05|0x1021 (rev: 0x8408)|0xC6C6|true|0x0000|
    ///
    /// ```
    /// # use crc_any::CRC;
    /// let mut crc = CRC::crc_a();
    /// crc.digest(b"123456789");
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0xBF05\", &crc.to_string());")]
    /// ```
    #[inline]
    pub fn crc_a() -> CRC {
        CRC::CRCu16(CRCu16::crc_a())
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0x2189|0x1021 (rev: 0x8408)|0x0000|true|0x0000|
    ///
    /// ```
    /// # use crc_any::CRC;
    /// let mut crc = CRC::crc16kermit();
    /// crc.digest(b"123456789");
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0x2189\", &crc.to_string());")]
    /// ```
    #[inline]
    pub fn crc16kermit() -> CRC {
        CRC::CRCu16(CRCu16::crc16kermit())
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0x4B37|0x8005 (rev: 0xA001)|0xFFFF|true|0x0000|
    ///
    /// ```
    /// # use crc_any::CRC;
    /// let mut crc = CRC::crc16modbus();
    /// crc.digest(b"123456789");
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0x4B37\", &crc.to_string());")]
    /// ```
    #[inline]
    pub fn crc16modbus() -> CRC {
        CRC::CRCu16(CRCu16::crc16modbus())
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0x906E|0x8005 (rev: 0xA001)|0xFFFF|true|0xFFFF|
    ///
    /// ```
    /// # use crc_any::CRC;
    /// let mut crc = CRC::crc16_x25();
    /// crc.digest(b"123456789");
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0x906E\", &crc.to_string());")]
    /// ```
    #[inline]
    pub fn crc16_x25() -> CRC {
        CRC::CRCu16(CRCu16::crc16_x25())
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0x31C3|0x1021|0x0000|false|0x0000|
    ///
    /// ```
    /// # use crc_any::CRC;
    /// let mut crc = CRC::crc16xmodem();
    /// crc.digest(b"123456789");
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0x31C3\", &crc.to_string());")]
    /// ```
    #[inline]
    pub fn crc16xmodem() -> CRC {
        CRC::CRCu16(CRCu16::crc16xmodem())
    }

    // TODO: CRC-17

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0x04F03|0x1685B|0x00000|false|0x00000|
    ///
    /// ```
    /// # use crc_any::CRC;
    /// let mut crc = CRC::crc17can();
    /// crc.digest(b"123456789");
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0x04F03\", &crc.to_string());")]
    /// ```
    #[inline]
    pub fn crc17can() -> CRC {
        CRC::CRCu32(CRCu32::crc17can())
    }

    // TODO: CRC-21

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0x0ED841|0x102899|0x000000|false|0x000000|
    ///
    /// ```
    /// # use crc_any::CRC;
    /// let mut crc = CRC::crc21can();
    /// crc.digest(b"123456789");
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0x0ED841\", &crc.to_string());")]
    /// ```
    #[inline]
    pub fn crc21can() -> CRC {
        CRC::CRCu32(CRCu32::crc21can())
    }

    // TODO: CRC-24

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0x21CF02|0x864CFB|0xB704CE|false|0x000000|
    ///
    /// ```
    /// # use crc_any::CRC;
    /// let mut crc = CRC::crc24();
    /// crc.digest(b"123456789");
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0x21CF02\", &crc.to_string());")]
    /// ```
    #[inline]
    pub fn crc24() -> CRC {
        CRC::CRCu32(CRCu32::crc24())
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0xC25A56|0x00065B (rev: 0xDA6000)|0x555555|true|0x000000|
    ///
    /// ```
    /// # use crc_any::CRC;
    /// let mut crc = CRC::crc24ble();
    /// crc.digest(b"123456789");
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0xC25A56\", &crc.to_string());")]
    /// ```
    #[inline]
    pub fn crc24ble() -> CRC {
        CRC::CRCu32(CRCu32::crc24ble())
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0x7979BD|0x5D6DCB|0xFEDCBA|false|0x000000|
    ///
    /// ```
    /// # use crc_any::CRC;
    /// let mut crc = CRC::crc24flexray_a();
    /// crc.digest(b"123456789");
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0x7979BD\", &crc.to_string());")]
    /// ```
    #[inline]
    pub fn crc24flexray_a() -> CRC {
        CRC::CRCu32(CRCu32::crc24flexray_a())
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0x1F23B8|0x5D6DCB|0xABCDEF|false|0x000000|
    ///
    /// ```
    /// # use crc_any::CRC;
    /// let mut crc = CRC::crc24flexray_b();
    /// crc.digest(b"123456789");
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0x1F23B8\", &crc.to_string());")]
    /// ```
    #[inline]
    pub fn crc24flexray_b() -> CRC {
        CRC::CRCu32(CRCu32::crc24flexray_b())
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0xCDE703|0x864CFB|0x000000|false|0x000000|
    ///
    /// ```
    /// # use crc_any::CRC;
    /// let mut crc = CRC::crc24lte_a();
    /// crc.digest(b"123456789");
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0xCDE703\", &crc.to_string());")]
    /// ```
    #[inline]
    pub fn crc24lte_a() -> CRC {
        CRC::CRCu32(CRCu32::crc24lte_a())
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0x23EF52|0x800063|0x000000|false|0x000000|
    ///
    /// ```
    /// # use crc_any::CRC;
    /// let mut crc = CRC::crc24lte_b();
    /// crc.digest(b"123456789");
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0x23EF52\", &crc.to_string());")]
    /// ```
    #[inline]
    pub fn crc24lte_b() -> CRC {
        CRC::CRCu32(CRCu32::crc24lte_b())
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0x200FA5|0x800063|0xFFFFFF|false|0xFFFFFF|
    ///
    /// ```
    /// # use crc_any::CRC;
    /// let mut crc = CRC::crc24os9();
    /// crc.digest(b"123456789");
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0x200FA5\", &crc.to_string());")]
    /// ```
    #[inline]
    pub fn crc24os9() -> CRC {
        CRC::CRCu32(CRCu32::crc24os9())
    }

    // TODO: CRC-30

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0x04C34ABF|0x2030B9C7|0x3FFFFFFF|false|0x3FFFFFFF|
    ///
    /// ```
    /// # use crc_any::CRC;
    /// let mut crc = CRC::crc30cdma();
    /// crc.digest(b"123456789");
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0x04C34ABF\", &crc.to_string());")]
    /// ```
    #[inline]
    pub fn crc30cdma() -> CRC {
        CRC::CRCu32(CRCu32::crc30cdma())
    }

    // TODO: CRC-32

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0xCBF43926|0x04C11DB7 (rev: 0xEDB88320)|0xFFFFFFFF|true|0xFFFFFFFF|
    ///
    /// ```
    /// # use crc_any::CRC;
    /// let mut crc = CRC::crc32();
    /// crc.digest(b"123456789");
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0xCBF43926\", &crc.to_string());")]
    /// ```
    #[inline]
    pub fn crc32() -> CRC {
        CRC::CRCu32(CRCu32::crc32())
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0x181989FC|0x04C11DB7|0xFFFFFFFF|false|0xFFFFFFFF|
    ///
    /// **Output will be reversed by bytes.**
    ///
    /// ```
    /// # use crc_any::CRC;
    /// let mut crc = CRC::crc32mhash();
    /// crc.digest(b"123456789");
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0x181989FC\", &crc.to_string());")]
    /// ```
    #[inline]
    pub fn crc32mhash() -> CRC {
        CRC::CRCu32(CRCu32::crc32mhash())
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0xFC891918|0x04C11DB7|0xFFFFFFFF|false|0xFFFFFFFF|
    ///
    /// ```
    /// # use crc_any::CRC;
    /// let mut crc = CRC::crc32bzip2();
    /// crc.digest(b"123456789");
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0xFC891918\", &crc.to_string());")]
    /// ```
    #[inline]
    pub fn crc32bzip2() -> CRC {
        CRC::CRCu32(CRCu32::crc32bzip2())
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0xE3069283|0x1EDC6F41 (rev: 0x82F63B78)|0xFFFFFFFF|true|0xFFFFFFFF|
    ///
    /// ```
    /// # use crc_any::CRC;
    /// let mut crc = CRC::crc32c();
    /// crc.digest(b"123456789");
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0xE3069283\", &crc.to_string());")]
    /// ```
    #[inline]
    pub fn crc32c() -> CRC {
        CRC::CRCu32(CRCu32::crc32c())
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0x87315576|0xA833982B (rev: 0xD419CC15)|0xFFFFFFFF|true|0xFFFFFFFF|
    ///
    /// ```
    /// # use crc_any::CRC;
    /// let mut crc = CRC::crc32d();
    /// crc.digest(b"123456789");
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0x87315576\", &crc.to_string());")]
    /// ```
    #[inline]
    pub fn crc32d() -> CRC {
        CRC::CRCu32(CRCu32::crc32d())
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0x0376E6E7|0x04C11DB7|0xFFFFFFFF|false|0x00000000|
    ///
    /// ```
    /// # use crc_any::CRC;
    /// let mut crc = CRC::crc32mpeg2();
    /// crc.digest(b"123456789");
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0x0376E6E7\", &crc.to_string());")]
    /// ```
    #[inline]
    pub fn crc32mpeg2() -> CRC {
        CRC::CRCu32(CRCu32::crc32mpeg2())
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0x765E7680|0x04C11DB7|0x00000000|false|0xFFFFFFFF|
    ///
    /// ```
    /// # use crc_any::CRC;
    /// let mut crc = CRC::crc32posix();
    /// crc.digest(b"123456789");
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0x765E7680\", &crc.to_string());")]
    /// ```
    #[inline]
    pub fn crc32posix() -> CRC {
        CRC::CRCu32(CRCu32::crc32posix())
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0x3010BF7F|0x814141AB|0x00000000|false|0x00000000|
    ///
    /// ```
    /// # use crc_any::CRC;
    /// let mut crc = CRC::crc32q();
    /// crc.digest(b"123456789");
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0x3010BF7F\", &crc.to_string());")]
    /// ```
    #[inline]
    pub fn crc32q() -> CRC {
        CRC::CRCu32(CRCu32::crc32q())
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0x340BC6D9|0x04C11DB7 (rev: 0xEDB88320)|0x00000000|true|0x00000000|
    ///
    /// ```
    /// # use crc_any::CRC;
    /// let mut crc = CRC::crc32jamcrc();
    /// crc.digest(b"123456789");
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0x340BC6D9\", &crc.to_string());")]
    /// ```
    #[inline]
    pub fn crc32jamcrc() -> CRC {
        CRC::CRCu32(CRCu32::crc32jamcrc())
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0xBD0BE338|0x000000AF|0x00000000|false|0x00000000|
    ///
    /// ```
    /// # use crc_any::CRC;
    /// let mut crc = CRC::crc32xfer();
    /// crc.digest(b"123456789");
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0xBD0BE338\", &crc.to_string());")]
    /// ```
    #[inline]
    pub fn crc32xfer() -> CRC {
        CRC::CRCu32(CRCu32::crc32xfer())
    }

    // TODO: CRC-40

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0xD4164FC646|0x0004820009|0x0000000000|false|0xFFFFFFFFFF|
    ///
    /// ```
    /// # use crc_any::CRC;
    /// let mut crc = CRC::crc40gsm();
    /// crc.digest(b"123456789");
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0xD4164FC646\", &crc.to_string());")]
    /// ```
    #[inline]
    pub fn crc40gsm() -> CRC {
        CRC::CRCu64(CRCu64::crc40gsm())
    }

    // TODO: CRC-64

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0x6C40DF5F0B497347|0x42F0E1EBA9EA3693|0x0000000000000000|false|0x0000000000000000|
    ///
    /// ```
    /// # use crc_any::CRC;
    /// let mut crc = CRC::crc64();
    /// crc.digest(b"123456789");
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0x6C40DF5F0B497347\", &crc.to_string());")]
    /// ```
    #[inline]
    pub fn crc64() -> CRC {
        CRC::CRCu64(CRCu64::crc64())
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0xB90956C775A41001|0x000000000000001B (rev: 0xD800000000000000)|0xFFFFFFFFFFFFFFFF|true|0xFFFFFFFFFFFFFFFF|
    ///
    /// ```
    /// # use crc_any::CRC;
    /// let mut crc = CRC::crc64iso();
    /// crc.digest(b"123456789");
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0xB90956C775A41001\", &crc.to_string());")]
    /// ```
    #[inline]
    pub fn crc64iso() -> CRC {
        CRC::CRCu64(CRCu64::crc64iso())
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0x62EC59E3F1A4F00A|0x42F0E1EBA9EA3693|0xFFFFFFFFFFFFFFFF|false|0xFFFFFFFFFFFFFFFF|
    ///
    /// ```
    /// # use crc_any::CRC;
    /// let mut crc = CRC::crc64we();
    /// crc.digest(b"123456789");
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0x62EC59E3F1A4F00A\", &crc.to_string());")]
    /// ```
    #[inline]
    pub fn crc64we() -> CRC {
        CRC::CRCu64(CRCu64::crc64we())
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0xE9C6D914C4B8D9CA|0xAD93D23594C935A9 (rev: 0x95AC9329AC4BC9B5)|0x0000000000000000|true|0x0000000000000000|
    ///
    /// ```
    /// # use crc_any::CRC;
    /// let mut crc = CRC::crc64jones();
    /// crc.digest(b"123456789");
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0xE9C6D914C4B8D9CA\", &crc.to_string());")]
    /// ```
    #[inline]
    pub fn crc64jones() -> CRC {
        CRC::CRCu64(CRCu64::crc64jones())
    }
}
