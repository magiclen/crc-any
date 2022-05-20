#[cfg(feature = "alloc")]
use alloc::fmt::{self, Debug, Display, Formatter};
#[cfg(feature = "alloc")]
use alloc::vec::Vec;

#[cfg(feature = "heapless")]
use heapless::Vec as HeaplessVec;

use crate::constants::crc_u64::*;
use crate::lookup_table::LookUpTable;

#[allow(clippy::upper_case_acronyms)]
/// This struct can help you compute a CRC-64 (or CRC-x where **x** is equal or less than `64`) value.
pub struct CRCu64 {
    by_table: bool,
    poly: u64,
    lookup_table: LookUpTable<u64>,
    sum: u64,
    pub(crate) bits: u8,
    high_bit: u64,
    mask: u64,
    initial: u64,
    final_xor: u64,
    reflect: bool,
    reorder: bool,
}

#[cfg(feature = "alloc")]
impl Debug for CRCu64 {
    #[inline]
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        if self.by_table {
            debug_helper::impl_debug_for_struct!(CRCu64, f, self, let .lookup_table = self.lookup_table.as_ref(), (.sum, "0x{:016X}", self.sum), .bits, (.initial, "0x{:016X}", self.initial), (.final_xor, "0x{:016X}", self.final_xor), .reflect, .reorder);
        } else {
            debug_helper::impl_debug_for_struct!(CRCu64, f, self, (.poly, "0x{:016X}", self.poly), (.sum, "0x{:016X}", self.sum), .bits, (.initial, "0x{:016X}", self.initial), (.final_xor, "0x{:016X}", self.final_xor), .reflect, .reorder);
        }
    }
}

#[cfg(feature = "alloc")]
impl Display for CRCu64 {
    #[inline]
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        f.write_fmt(format_args!("0x{:01$X}", self.get_crc(), (self.bits as usize + 3) >> 2))
    }
}

impl CRCu64 {
    /// Create a `CRCu64` instance by providing the length of bits, expression, reflection, an initial value and a final xor value.
    pub fn create_crc(poly: u64, bits: u8, initial: u64, final_xor: u64, reflect: bool) -> CRCu64 {
        debug_assert!(bits <= 64 && bits > 0);

        if bits % 8 == 0 {
            let lookup_table = if reflect {
                LookUpTable::Dynamic(Self::crc_reflect_table(poly))
            } else {
                LookUpTable::Dynamic(Self::crc_table(poly, bits))
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
                LookUpTable::Static(&[0u64; 256]),
                poly,
                bits,
                initial,
                final_xor,
                reflect,
            )
        }
    }

    /// Create a `CRCu64` instance by providing an existing lookup table, the length of bits, expression, reflection, an initial value and a final xor value.
    pub(crate) fn create_crc_with_exists_lookup_table(
        lookup_table: LookUpTable<u64>,
        bits: u8,
        initial: u64,
        final_xor: u64,
        reflect: bool,
    ) -> CRCu64 {
        debug_assert!(bits % 8 == 0);

        Self::create(true, lookup_table, 0, bits, initial, final_xor, reflect)
    }

    #[inline]
    fn create(
        by_table: bool,
        lookup_table: LookUpTable<u64>,
        mut poly: u64,
        bits: u8,
        initial: u64,
        final_xor: u64,
        reflect: bool,
    ) -> CRCu64 {
        let high_bit = 1 << u64::from(bits - 1);
        let mask = ((high_bit - 1) << 1) | 1;

        let sum = if reflect {
            Self::reflect_function(high_bit, initial)
        } else {
            initial
        };

        if !by_table && reflect {
            poly = Self::reflect_function(high_bit, poly);
        }

        CRCu64 {
            by_table,
            poly,
            lookup_table,
            sum,
            bits,
            high_bit,
            mask,
            initial,
            final_xor,
            reflect,
            reorder: false,
        }
    }

    #[inline]
    fn reflect_function(high_bit: u64, n: u64) -> u64 {
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
    fn reflect_method(&self, n: u64) -> u64 {
        Self::reflect_function(self.high_bit, n)
    }

    /// Digest some data.
    pub fn digest<T: ?Sized + AsRef<[u8]>>(&mut self, data: &T) {
        if self.by_table {
            if self.bits == 8 {
                for n in data.as_ref().iter().copied() {
                    let index = (self.sum as u8 ^ n) as usize;
                    self.sum = self.lookup_table[index];
                }
            } else if self.reflect {
                for n in data.as_ref().iter().copied() {
                    let index = ((self.sum as u8) ^ n) as usize;
                    self.sum = (self.sum >> 8) ^ self.lookup_table[index];
                }
            } else {
                for n in data.as_ref().iter().copied() {
                    let index = ((self.sum >> u64::from(self.bits - 8)) as u8 ^ n) as usize;
                    self.sum = (self.sum << 8) ^ self.lookup_table[index];
                }
            }
        } else if self.reflect {
            for n in data.as_ref().iter().copied() {
                let n = super::crc_u8::CRCu8::reflect_function(0x80, n);

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
    pub fn reset(&mut self) {
        self.sum = self.initial;
    }

    /// Get the current CRC value (it always returns a `u64` value). You can continue calling `digest` method even after getting a CRC value.
    pub fn get_crc(&self) -> u64 {
        let sum = if self.by_table || !self.reflect {
            (self.sum ^ self.final_xor) & self.mask
        } else {
            (self.reflect_method(self.sum) ^ self.final_xor) & self.mask
        };

        if self.reorder {
            let mut new_sum = 0;

            let e = (self.bits as u64 + 7) >> 3;

            let e_dec = e - 1;

            for i in 0..e {
                new_sum |= ((sum >> ((e_dec - i) * 8)) & 0xFF) << (i * 8);
            }

            new_sum
        } else {
            sum
        }
    }

    fn crc_reflect_table(poly_rev: u64) -> [u64; 256] {
        let mut lookup_table = [0u64; 256];

        for (i, e) in lookup_table.iter_mut().enumerate() {
            let mut v = i as u64;

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

    fn crc_table(poly: u64, bits: u8) -> [u64; 256] {
        let mut lookup_table = [0u64; 256];

        let mask1 = 1u64 << u64::from(bits - 1);

        let mask2 = ((mask1 - 1) << 1) | 1;

        for (i, e) in lookup_table.iter_mut().enumerate() {
            let mut v = i as u64;

            #[allow(clippy::branches_sharing_code)]
            for _ in 0..bits {
                if v & mask1 == 0 {
                    v <<= 1;
                } else {
                    v <<= 1;
                    v ^= poly;
                }
            }

            *e = v & mask2;
        }

        lookup_table
    }
}

#[cfg(feature = "alloc")]
impl CRCu64 {
    /// Get the current CRC value (it always returns a vec instance with a length corresponding to the CRC bits). You can continue calling `digest` method even after getting a CRC value.
    #[inline]
    pub fn get_crc_vec_le(&mut self) -> Vec<u8> {
        let crc = self.get_crc();

        let e = (self.bits as usize + 7) >> 3;

        crc.to_le_bytes()[..e].to_vec()
    }

    /// Get the current CRC value (it always returns a vec instance with a length corresponding to the CRC bits). You can continue calling `digest` method even after getting a CRC value.
    #[inline]
    pub fn get_crc_vec_be(&mut self) -> Vec<u8> {
        let crc = self.get_crc();

        let e = (self.bits as usize + 7) >> 3;

        crc.to_be_bytes()[(8 - e)..].to_vec()
    }
}

#[cfg(feature = "heapless")]
impl CRCu64 {
    /// Get the current CRC value (it always returns a heapless vec instance with a length corresponding to the CRC bits). You can continue calling `digest` method even after getting a CRC value.
    #[inline]
    pub fn get_crc_heapless_vec_le(&mut self) -> HeaplessVec<u8, 8> {
        let crc = self.get_crc();

        let e = (self.bits as usize + 7) >> 3;

        let mut vec = HeaplessVec::new();

        vec.extend_from_slice(&crc.to_le_bytes()[..e]).unwrap();

        vec
    }

    /// Get the current CRC value (it always returns a heapless vec instance with a length corresponding to the CRC bits). You can continue calling `digest` method even after getting a CRC value.
    #[inline]
    pub fn get_crc_heapless_vec_be(&mut self) -> HeaplessVec<u8, 8> {
        let crc = self.get_crc();

        let e = (self.bits as usize + 7) >> 3;

        let mut vec = HeaplessVec::new();

        vec.extend_from_slice(&crc.to_be_bytes()[(8 - e)..]).unwrap();

        vec
    }
}

impl CRCu64 {
    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0xD4164FC646|0x0004820009|0x0000000000|false|0xFFFFFFFFFF|
    ///
    /// ```
    /// # use crc_any::CRCu64;
    /// let mut crc = CRCu64::crc40gsm();
    /// crc.digest(b"123456789");
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0xD4164FC646\", &crc.to_string());")]
    /// ```
    pub fn crc40gsm() -> CRCu64 {
        // Self::create_crc(0x0000000004820009u64, 40, 0x0000000000000000, 0x000000FFFFFFFFFF, false)

        let lookup_table = LookUpTable::Static(&NO_REF_40_0000000004820009);
        Self::create_crc_with_exists_lookup_table(
            lookup_table,
            40,
            0x0000000000000000,
            0x000000FFFFFFFFFF,
            false,
        )
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0x6C40DF5F0B497347|0x42F0E1EBA9EA3693|0x0000000000000000|false|0x0000000000000000|
    ///
    /// ```
    /// # use crc_any::CRCu64;
    /// let mut crc = CRCu64::crc64();
    /// crc.digest(b"123456789");
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0x6C40DF5F0B497347\", &crc.to_string());")]
    /// ```
    pub fn crc64() -> CRCu64 {
        // Self::create_crc(0x42F0E1EBA9EA3693, 64, 0x0000000000000000, 0x0000000000000000, false)

        let lookup_table = LookUpTable::Static(&NO_REF_64_42F0E1EBA9EA3693);
        Self::create_crc_with_exists_lookup_table(
            lookup_table,
            64,
            0x0000000000000000,
            0x0000000000000000,
            false,
        )
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0xB90956C775A41001|0x000000000000001B (rev: 0xD800000000000000)|0xFFFFFFFFFFFFFFFF|true|0xFFFFFFFFFFFFFFFF|
    ///
    /// ```
    /// # use crc_any::CRCu64;
    /// let mut crc = CRCu64::crc64iso();
    /// crc.digest(b"123456789");
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0xB90956C775A41001\", &crc.to_string());")]
    /// ```
    pub fn crc64iso() -> CRCu64 {
        // Self::create_crc(0xD800000000000000, 64, 0xFFFFFFFFFFFFFFFF, 0xFFFFFFFFFFFFFFFF, true)

        let lookup_table = LookUpTable::Static(&REF_64_D800000000000000);
        Self::create_crc_with_exists_lookup_table(
            lookup_table,
            64,
            0xFFFFFFFFFFFFFFFF,
            0xFFFFFFFFFFFFFFFF,
            true,
        )
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0x62EC59E3F1A4F00A|0x42F0E1EBA9EA3693|0xFFFFFFFFFFFFFFFF|false|0xFFFFFFFFFFFFFFFF|
    ///
    /// ```
    /// # use crc_any::CRCu64;
    /// let mut crc = CRCu64::crc64we();
    /// crc.digest(b"123456789");
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0x62EC59E3F1A4F00A\", &crc.to_string());")]
    /// ```
    pub fn crc64we() -> CRCu64 {
        // Self::create_crc(0x42F0E1EBA9EA3693, 64, 0xFFFFFFFFFFFFFFFF, 0xFFFFFFFFFFFFFFFF, false)

        let lookup_table = LookUpTable::Static(&NO_REF_64_42F0E1EBA9EA3693);
        Self::create_crc_with_exists_lookup_table(
            lookup_table,
            64,
            0xFFFFFFFFFFFFFFFF,
            0xFFFFFFFFFFFFFFFF,
            false,
        )
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0xE9C6D914C4B8D9CA|0xAD93D23594C935A9 (rev: 0x95AC9329AC4BC9B5)|0x0000000000000000|true|0x0000000000000000|
    ///
    /// ```
    /// # use crc_any::CRCu64;
    /// let mut crc = CRCu64::crc64jones();
    /// crc.digest(b"123456789");
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0xE9C6D914C4B8D9CA\", &crc.to_string());")]
    /// ```
    pub fn crc64jones() -> CRCu64 {
        // Self::create_crc(0x95AC9329AC4BC9B5, 64, 0x0000000000000000, 0x0000000000000000, true)

        let lookup_table = LookUpTable::Static(&REF_64_95AC9329AC4BC9B5);
        Self::create_crc_with_exists_lookup_table(
            lookup_table,
            64,
            0x0000000000000000,
            0x0000000000000000,
            true,
        )
    }
}

#[cfg(all(feature = "development", test))]
mod tests {
    use super::CRCu64;

    use alloc::fmt::Write;
    use alloc::string::String;

    #[test]
    fn print_lookup_table() {
        let crc = CRCu64::crc64jones();

        let mut s = String::new();

        for n in crc.lookup_table.iter().take(255) {
            s.write_fmt(format_args!("{}u64, ", n)).unwrap();
        }

        s.write_fmt(format_args!("{}u64", crc.lookup_table[255])).unwrap();

        println!("let lookup_table = [{}];", s);
    }
}
