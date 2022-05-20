#[cfg(feature = "alloc")]
use alloc::fmt::{self, Debug, Display, Formatter};

use crate::constants::crc_u8::*;
use crate::lookup_table::LookUpTable;

#[allow(clippy::upper_case_acronyms)]
/// This struct can help you compute a CRC-8 (or CRC-x where **x** is equal or less than `8`) value.
pub struct CRCu8 {
    by_table: bool,
    poly: u8,
    lookup_table: LookUpTable<u8>,
    sum: u8,
    #[cfg(any(feature = "alloc", feature = "heapless"))]
    pub(crate) bits: u8,
    high_bit: u8,
    mask: u8,
    initial: u8,
    final_xor: u8,
    reflect: bool,
}

#[cfg(feature = "alloc")]
impl Debug for CRCu8 {
    #[inline]
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        if self.by_table {
            debug_helper::impl_debug_for_struct!(CRCu64, f, self, let .lookup_table = self.lookup_table.as_ref(), (.sum, "0x{:02X}", self.sum), .bits, (.initial, "0x{:02X}", self.initial), (.final_xor, "0x{:02X}", self.final_xor), .reflect);
        } else {
            debug_helper::impl_debug_for_struct!(CRCu64, f, self, (.poly, "0x{:02X}", self.poly), (.sum, "0x{:02X}", self.sum), .bits, (.initial, "0x{:02X}", self.initial), (.final_xor, "0x{:02X}", self.final_xor), .reflect);
        }
    }
}

#[cfg(feature = "alloc")]
impl Display for CRCu8 {
    #[inline]
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        f.write_fmt(format_args!("0x{:01$X}", self.get_crc(), (self.bits as usize + 3) >> 2))
    }
}

impl CRCu8 {
    /// Create a `CRCu8` instance by providing the length of bits, expression, reflection, an initial value and a final xor value.
    pub fn create_crc(poly: u8, bits: u8, initial: u8, final_xor: u8, reflect: bool) -> CRCu8 {
        debug_assert!(bits <= 8 && bits > 0);

        if bits % 8 == 0 {
            let lookup_table = if reflect {
                LookUpTable::Dynamic(Self::crc_reflect_table(poly))
            } else {
                LookUpTable::Dynamic(Self::crc_table(poly))
            };

            Self::create_crc_with_exists_lookup_table(
                lookup_table,
                bits,
                initial,
                final_xor,
                reflect,
            )
        } else {
            Self::create(
                false,
                LookUpTable::Static(&[0; 256]),
                poly,
                bits,
                initial,
                final_xor,
                reflect,
            )
        }
    }

    #[inline]
    pub(crate) fn create_crc_with_exists_lookup_table(
        lookup_table: LookUpTable<u8>,
        bits: u8,
        initial: u8,
        final_xor: u8,
        reflect: bool,
    ) -> CRCu8 {
        debug_assert!(bits % 8 == 0);

        Self::create(true, lookup_table, 0, bits, initial, final_xor, reflect)
    }

    #[inline]
    fn create(
        by_table: bool,
        lookup_table: LookUpTable<u8>,
        mut poly: u8,
        bits: u8,
        initial: u8,
        final_xor: u8,
        reflect: bool,
    ) -> CRCu8 {
        let high_bit = 1 << (bits - 1);
        let mask = ((high_bit - 1) << 1) | 1;

        let sum = if reflect {
            Self::reflect_function(high_bit, initial)
        } else {
            initial
        };

        if !by_table && reflect {
            poly = Self::reflect_function(high_bit, poly);
        }

        CRCu8 {
            by_table,
            poly,
            lookup_table,
            sum,
            #[cfg(any(feature = "alloc", feature = "heapless"))]
            bits,
            high_bit,
            mask,
            initial,
            final_xor,
            reflect,
        }
    }

    #[inline]
    pub(crate) fn reflect_function(high_bit: u8, n: u8) -> u8 {
        let mut i = high_bit;
        let mut j = 1;
        let mut out = 0;

        while i != 0 {
            if n & i != 0 {
                out |= j;
            }

            j <<= 1;
            i >>= 1;
        }

        out
    }

    #[inline]
    fn reflect_method(&self, n: u8) -> u8 {
        Self::reflect_function(self.high_bit, n)
    }

    /// Digest some data.
    pub fn digest<T: ?Sized + AsRef<[u8]>>(&mut self, data: &T) {
        if self.by_table {
            for n in data.as_ref().iter().copied() {
                let index = (self.sum ^ n) as usize;
                self.sum = self.lookup_table[index];
            }
        } else if self.reflect {
            for n in data.as_ref().iter().copied() {
                let n = Self::reflect_function(0x80, n);

                let mut i = 0x80;

                while i != 0 {
                    let mut bit = self.sum & self.high_bit;

                    self.sum <<= 1;

                    if n & i != 0 {
                        bit ^= self.high_bit;
                    }

                    if bit != 0 {
                        self.sum ^= self.poly;
                    }

                    i >>= 1;
                }
            }
        } else {
            for n in data.as_ref().iter().copied() {
                let mut i = 0x80;

                while i != 0 {
                    let mut bit = self.sum & self.high_bit;

                    self.sum <<= 1;

                    if n & i != 0 {
                        bit ^= self.high_bit;
                    }

                    if bit != 0 {
                        self.sum ^= self.poly;
                    }

                    i >>= 1;
                }
            }
        }
    }

    /// Reset the sum.
    #[inline]
    pub fn reset(&mut self) {
        self.sum = self.initial;
    }

    /// Get the current CRC value (it always returns a `u8` value). You can continue calling `digest` method even after getting a CRC value.
    #[inline]
    pub fn get_crc(&self) -> u8 {
        if self.by_table || !self.reflect {
            (self.sum ^ self.final_xor) & self.mask
        } else {
            (self.reflect_method(self.sum) ^ self.final_xor) & self.mask
        }
    }

    fn crc_reflect_table(poly_rev: u8) -> [u8; 256] {
        let mut lookup_table = [0u8; 256];

        for (i, e) in lookup_table.iter_mut().enumerate() {
            let mut v = i as u8;

            #[allow(clippy::branches_sharing_code)]
            for _ in 0..8u8 {
                if v & 1 != 0 {
                    v >>= 1;
                    v ^= poly_rev;
                } else {
                    v >>= 1;
                }
            }

            *e = v;
        }

        lookup_table
    }

    fn crc_table(poly: u8) -> [u8; 256] {
        let mut lookup_table = [0u8; 256];

        for (i, e) in lookup_table.iter_mut().enumerate() {
            let mut v = i as u8;

            #[allow(clippy::branches_sharing_code)]
            for _ in 0..8 {
                if v & 0x80 == 0 {
                    v <<= 1;
                } else {
                    v <<= 1;
                    v ^= poly;
                }
            }

            *e = v;
        }

        lookup_table
    }
}

impl CRCu8 {
    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0x4|0x3|0x0|false|0x7|
    ///
    /// ```
    /// # use crc_any::CRCu8;
    /// let mut crc = CRCu8::crc3gsm();
    /// crc.digest(b"123456789");
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0x4\", &crc.to_string());")]
    /// ```
    #[inline]
    pub fn crc3gsm() -> CRCu8 {
        Self::create_crc(0x03, 3, 0x00, 0x07, false)
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0x7|0x3 (rev: 0xC)|0x0|true|0x0|
    ///
    /// ```
    /// # use crc_any::CRCu8;
    /// let mut crc = CRCu8::crc4itu();
    /// crc.digest(b"123456789");
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0x7\", &crc.to_string());")]
    /// ```
    #[inline]
    pub fn crc4itu() -> CRCu8 {
        Self::create_crc(0x0C, 4, 0x00, 0x00, true)
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0xB|0x3|0xF|false|0xF|
    ///
    /// ```
    /// # use crc_any::CRCu8;
    /// let mut crc = CRCu8::crc4interlaken();
    /// crc.digest(b"123456789");
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0xB\", &crc.to_string());")]
    /// ```
    #[inline]
    pub fn crc4interlaken() -> CRCu8 {
        Self::create_crc(0x03, 4, 0x0F, 0x0F, false)
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0x00|0x09|0x09|false|0x00|
    ///
    /// ```
    /// # use crc_any::CRCu8;
    /// let mut crc = CRCu8::crc5epc();
    /// crc.digest(b"123456789");
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0x00\", &crc.to_string());")]
    /// ```
    #[inline]
    pub fn crc5epc() -> CRCu8 {
        Self::create_crc(0x09, 5, 0x09, 0x00, false)
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0x07|0x15 (rev: 0x15)|0x00|true|0x00|
    ///
    /// ```
    /// # use crc_any::CRCu8;
    /// let mut crc = CRCu8::crc5itu();
    /// crc.digest(b"123456789");
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0x07\", &crc.to_string());")]
    /// ```
    #[inline]
    pub fn crc5itu() -> CRCu8 {
        Self::create_crc(0x15, 5, 0x00, 0x00, true)
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0x19|0x05 (rev: 0x14)|0x1F|true|0x1F|
    ///
    /// ```
    /// # use crc_any::CRCu8;
    /// let mut crc = CRCu8::crc5usb();
    /// crc.digest(b"123456789");
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0x19\", &crc.to_string());")]
    /// ```
    #[inline]
    pub fn crc5usb() -> CRCu8 {
        Self::create_crc(0x14, 5, 0x1F, 0x1F, true)
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0x0D|0x27|0x3F|false|0x00|
    ///
    /// ```
    /// # use crc_any::CRCu8;
    /// let mut crc = CRCu8::crc6cdma2000_a();
    /// crc.digest(b"123456789");
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0x0D\", &crc.to_string());")]
    /// ```
    #[inline]
    pub fn crc6cdma2000_a() -> CRCu8 {
        Self::create_crc(0x27, 6, 0x3f, 0x00, false)
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0x3B|0x07|0x3F|false|0x00|
    ///
    /// ```
    /// # use crc_any::CRCu8;
    /// let mut crc = CRCu8::crc6cdma2000_b();
    /// crc.digest(b"123456789");
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0x3B\", &crc.to_string());")]
    /// ```
    #[inline]
    pub fn crc6cdma2000_b() -> CRCu8 {
        Self::create_crc(0x07, 6, 0x3f, 0x00, false)
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0x26|0x19 (rev: 0x26)|0x00|true|0x00|
    ///
    /// ```
    /// # use crc_any::CRCu8;
    /// let mut crc = CRCu8::crc6darc();
    /// crc.digest(b"123456789");
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0x26\", &crc.to_string());")]
    /// ```
    #[inline]
    pub fn crc6darc() -> CRCu8 {
        Self::create_crc(0x26, 6, 0x00, 0x00, true)
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0x13|0x2F|0x00|false|0x3F|
    ///
    /// ```
    /// # use crc_any::CRCu8;
    /// let mut crc = CRCu8::crc6gsm();
    /// crc.digest(b"123456789");
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0x13\", &crc.to_string());")]
    /// ```
    #[inline]
    pub fn crc6gsm() -> CRCu8 {
        Self::create_crc(0x2F, 6, 0x00, 0x3F, false)
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0x06|0x03 (rev: 0x30)|0x00|true|0x00|
    ///
    /// ```
    /// # use crc_any::CRCu8;
    /// let mut crc = CRCu8::crc6itu();
    /// crc.digest(b"123456789");
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0x06\", &crc.to_string());")]
    /// ```
    #[inline]
    pub fn crc6itu() -> CRCu8 {
        Self::create_crc(0x30, 6, 0x00, 0x00, true)
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0x75|0x09|0x00|false|0x00|
    ///
    /// ```
    /// # use crc_any::CRCu8;
    /// let mut crc = CRCu8::crc7();
    /// crc.digest(b"123456789");
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0x75\", &crc.to_string());")]
    /// ```
    #[inline]
    pub fn crc7() -> CRCu8 {
        Self::create_crc(0x09, 7, 0x00, 0x00, false)
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0x61|0x45|0x00|false|0x00|
    ///
    /// ```
    /// # use crc_any::CRCu8;
    /// let mut crc = CRCu8::crc7umts();
    /// crc.digest(b"123456789");
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0x61\", &crc.to_string());")]
    /// ```
    #[inline]
    pub fn crc7umts() -> CRCu8 {
        Self::create_crc(0x45, 7, 0x00, 0x00, false)
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0xF4|0x07|0x00|false|0x00|
    ///
    /// ```
    /// # use crc_any::CRCu8;
    /// let mut crc = CRCu8::crc8();
    /// crc.digest(b"123456789");
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0xF4\", &crc.to_string());")]
    /// ```
    #[inline]
    pub fn crc8() -> CRCu8 {
        // Self::create_crc(0x07, 8, 0x00, 0x00, false)

        let lookup_table = LookUpTable::Static(&NO_REF_8_07);
        Self::create_crc_with_exists_lookup_table(lookup_table, 8, 0x00, 0x00, false)
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0xDA|0x9B|0xFF|false|0x00|
    ///
    /// ```
    /// # use crc_any::CRCu8;
    /// let mut crc = CRCu8::crc8cdma2000();
    /// crc.digest(b"123456789");
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0xDA\", &crc.to_string());")]
    /// ```
    #[inline]
    pub fn crc8cdma2000() -> CRCu8 {
        // Self::create_crc(0x9B, 8, 0xFF, 0x00, false)

        let lookup_table = LookUpTable::Static(&NO_REF_8_9B);
        Self::create_crc_with_exists_lookup_table(lookup_table, 8, 0xFF, 0x00, false)
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0x15|0x39 (rev: 0x9C)|0x00|true|0x00|
    ///
    /// ```
    /// # use crc_any::CRCu8;
    /// let mut crc = CRCu8::crc8darc();
    /// crc.digest(b"123456789");
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0x15\", &crc.to_string());")]
    /// ```
    #[inline]
    pub fn crc8darc() -> CRCu8 {
        // Self::create_crc(0x9C, 8, 0x00, 0x00, true)

        let lookup_table = LookUpTable::Static(&REF_8_9C);
        Self::create_crc_with_exists_lookup_table(lookup_table, 8, 0x00, 0x00, true)
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0xBC|0xD5|0x00|false|0x00|
    ///
    /// ```
    /// # use crc_any::CRCu8;
    /// let mut crc = CRCu8::crc8dvb_s2();
    /// crc.digest(b"123456789");
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0xBC\", &crc.to_string());")]
    /// ```
    #[inline]
    pub fn crc8dvb_s2() -> CRCu8 {
        // Self::create_crc(0xD5, 8, 0x00, 0x00, false)

        let lookup_table = LookUpTable::Static(&NO_REF_8_D5);
        Self::create_crc_with_exists_lookup_table(lookup_table, 8, 0x00, 0x00, false)
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0x97|0x1D (rev: 0xB8)|0xFF|true|0x00|
    ///
    /// ```
    /// # use crc_any::CRCu8;
    /// let mut crc = CRCu8::crc8ebu();
    /// crc.digest(b"123456789");
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0x97\", &crc.to_string());")]
    /// ```
    #[inline]
    pub fn crc8ebu() -> CRCu8 {
        // Self::create_crc(0xB8, 8, 0xFF, 0x00, true)

        let lookup_table = LookUpTable::Static(&REF_8_B8);
        Self::create_crc_with_exists_lookup_table(lookup_table, 8, 0xFF, 0x00, true)
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0x7E|0x1D|0xFD|false|0x00|
    ///
    /// ```
    /// # use crc_any::CRCu8;
    /// let mut crc = CRCu8::crc8icode();
    /// crc.digest(b"123456789");
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0x7E\", &crc.to_string());")]
    /// ```
    #[inline]
    pub fn crc8icode() -> CRCu8 {
        // Self::create_crc(0x1D, 8, 0xFD, 0x00, false)

        let lookup_table = LookUpTable::Static(&NO_REF_8_1D);
        Self::create_crc_with_exists_lookup_table(lookup_table, 8, 0xFD, 0x00, false)
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0xA1|0x07|0x00|false|0x55|
    ///
    /// ```
    /// # use crc_any::CRCu8;
    /// let mut crc = CRCu8::crc8itu();
    /// crc.digest(b"123456789");
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0xA1\", &crc.to_string());")]
    /// ```
    #[inline]
    pub fn crc8itu() -> CRCu8 {
        // Self::create_crc(0x07, 8, 0x00, 0x55, false)

        let lookup_table = LookUpTable::Static(&NO_REF_8_07);
        Self::create_crc_with_exists_lookup_table(lookup_table, 8, 0x00, 0x55, false)
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0xA1|0x31 (rev: 0x8C)|0x00|true|0x00|
    ///
    /// ```
    /// # use crc_any::CRCu8;
    /// let mut crc = CRCu8::crc8maxim();
    /// crc.digest(b"123456789");
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0xA1\", &crc.to_string());")]
    /// ```
    #[inline]
    pub fn crc8maxim() -> CRCu8 {
        // Self::create_crc(0x8C, 8, 0x00, 0x00, true)

        let lookup_table = LookUpTable::Static(&REF_8_8C);
        Self::create_crc_with_exists_lookup_table(lookup_table, 8, 0x00, 0x00, true)
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0xD0|0x07 (rev: 0xE0)|0xFF|true|0x00|
    ///
    /// ```
    /// # use crc_any::CRCu8;
    /// let mut crc = CRCu8::crc8rohc();
    /// crc.digest(b"123456789");
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0xD0\", &crc.to_string());")]
    /// ```
    #[inline]
    pub fn crc8rohc() -> CRCu8 {
        // Self::create_crc(0xE0, 8, 0xFF, 0x00, true)

        let lookup_table = LookUpTable::Static(&REF_8_E0);
        Self::create_crc_with_exists_lookup_table(lookup_table, 8, 0xFF, 0x00, true)
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0x25|0x9B (rev: 0xD9)|0x00|true|0x00|
    ///
    /// ```
    /// # use crc_any::CRCu8;
    /// let mut crc = CRCu8::crc8wcdma();
    /// crc.digest(b"123456789");
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0x25\", &crc.to_string());")]
    /// ```
    #[inline]
    pub fn crc8wcdma() -> CRCu8 {
        // Self::create_crc(0xD9, 8, 0x00, 0x00, true)

        let lookup_table = LookUpTable::Static(&REF_8_D9);
        Self::create_crc_with_exists_lookup_table(lookup_table, 8, 0x00, 0x00, true)
    }
}

#[cfg(all(feature = "development", test))]
mod tests {
    use super::CRCu8;

    use alloc::fmt::Write;
    use alloc::string::String;

    #[test]
    fn print_lookup_table() {
        let crc = CRCu8::crc4interlaken();

        let mut s = String::new();

        for n in crc.lookup_table.iter().take(255) {
            s.write_fmt(format_args!("{}u8, ", n)).unwrap();
        }

        s.write_fmt(format_args!("{}u8", crc.lookup_table[255])).unwrap();

        println!("let lookup_table = [{}];", s);
    }
}
