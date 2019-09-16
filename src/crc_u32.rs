#[cfg(feature = "alloc")]
use alloc::fmt::{self, Debug, Display, Formatter};
#[cfg(feature = "alloc")]
use alloc::vec::Vec;

#[cfg(feature = "heapless")]
use heapless::consts::U4;
#[cfg(feature = "heapless")]
use heapless::Vec as HeaplessVec;

use crate::constants::crc_u32::*;

/// This struct can help you compute a CRC-32 (or CRC-x where **x** is under `32`) value.
pub struct CRCu32 {
    by_table: bool,
    poly: u32,
    lookup_table: [u32; 256],
    sum: u32,
    pub(crate) bits: u8,
    high_bit: u32,
    mask: u32,
    initial: u32,
    final_xor: u32,
    reflect: bool,
    reorder: bool,
}

#[cfg(feature = "alloc")]
impl Debug for CRCu32 {
    #[inline]
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        if self.by_table {
            impl_debug_for_struct!(CRCu64, f, self, let .lookup_table = self.lookup_table.as_ref(), (.sum, "0x{:08X}", self.sum), .bits, (.initial, "0x{:08X}", self.initial), (.final_xor, "0x{:08X}", self.final_xor), .reflect, .reorder);
        } else {
            impl_debug_for_struct!(CRCu64, f, self, (.poly, "0x{:08X}", self.poly), (.sum, "0x{:08X}", self.sum), .bits, (.initial, "0x{:08X}", self.initial), (.final_xor, "0x{:08X}", self.final_xor), .reflect, .reorder);
        }
    }
}

#[cfg(feature = "alloc")]
impl Display for CRCu32 {
    #[inline]
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        f.write_fmt(format_args!(
            "0x{:01$X}",
            self.get_crc(),
            ((f64::from(self.bits) + 3f64) / 4f64) as usize
        ))
    }
}

impl CRCu32 {
    /// Create a `CRCu32` instance by providing the length of bits, expression, reflection, an initial value and a final xor value.
    pub fn create_crc(poly: u32, bits: u8, initial: u32, final_xor: u32, reflect: bool) -> CRCu32 {
        debug_assert!(bits <= 32 && bits > 0);

        if bits % 8 == 0 {
            let lookup_table = if reflect {
                Self::crc_reflect_table(poly)
            } else {
                Self::crc_table(poly, bits)
            };

            Self::create_crc_with_exists_lookup_table(
                lookup_table,
                bits,
                initial,
                final_xor,
                reflect,
            )
        } else {
            Self::create(false, [0u32; 256], poly, bits, initial, final_xor, reflect)
        }
    }

    #[inline]
    pub(crate) fn create_crc_with_exists_lookup_table(
        lookup_table: [u32; 256],
        bits: u8,
        initial: u32,
        final_xor: u32,
        reflect: bool,
    ) -> CRCu32 {
        debug_assert!(bits % 8 == 0);

        Self::create(true, lookup_table, 0, bits, initial, final_xor, reflect)
    }

    #[inline]
    fn create(
        by_table: bool,
        lookup_table: [u32; 256],
        mut poly: u32,
        bits: u8,
        initial: u32,
        final_xor: u32,
        reflect: bool,
    ) -> CRCu32 {
        let high_bit = 1 << u32::from(bits - 1);
        let mask = ((high_bit - 1) << 1) | 1;

        let sum = if reflect {
            Self::reflect_function(high_bit, initial)
        } else {
            initial
        };

        if !by_table && reflect {
            poly = Self::reflect_function(high_bit, poly);
        }

        CRCu32 {
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
    fn reflect_function(high_bit: u32, n: u32) -> u32 {
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
    fn reflect_method(&self, n: u32) -> u32 {
        Self::reflect_function(self.high_bit, n)
    }

    /// Digest some data.
    pub fn digest<T: ?Sized + AsRef<[u8]>>(&mut self, data: &T) {
        if self.by_table {
            if self.bits == 8 {
                for &n in data.as_ref() {
                    let index = (self.sum as u8 ^ n) as usize;
                    self.sum = self.lookup_table[index];
                }
            } else if self.reflect {
                for &n in data.as_ref() {
                    let index = ((self.sum as u8) ^ n) as usize;
                    self.sum = (self.sum >> 8) ^ self.lookup_table[index];
                }
            } else {
                for &n in data.as_ref() {
                    let index = ((self.sum >> u32::from(self.bits - 8)) as u8 ^ n) as usize;
                    self.sum = (self.sum << 8) ^ self.lookup_table[index];
                }
            }
        } else if self.reflect {
            for &n in data.as_ref() {
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
            for &n in data.as_ref() {
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

    /// Get the current CRC value (it always returns a `u32` value). You can continue calling `digest` method even after getting a CRC value.
    pub fn get_crc(&self) -> u32 {
        let sum = if self.by_table {
            (self.sum ^ self.final_xor) & self.mask
        } else if self.reflect {
            (self.reflect_method(self.sum) ^ self.final_xor) & self.mask
        } else {
            (self.sum ^ self.final_xor) & self.mask
        };

        if self.reorder {
            let mut new_sum = 0;

            let e = ((f64::from(self.bits) + 7f64) / 8f64) as u32;

            let e_dec = e - 1;

            for i in 0..e {
                new_sum |= ((sum >> ((e_dec - i) * 8)) & 0xFF) << (i * 8);
            }

            new_sum
        } else {
            sum
        }
    }

    fn crc_reflect_table(poly_rev: u32) -> [u32; 256] {
        let mut lookup_table = [0u32; 256];

        for (i, e) in lookup_table.iter_mut().enumerate() {
            let mut v = i as u32;

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

    fn crc_table(poly: u32, bits: u8) -> [u32; 256] {
        let mut lookup_table = [0u32; 256];

        let mask1 = 1u32 << u32::from(bits - 1);

        let mask2 = ((mask1 - 1) << 1) | 1;

        for (i, e) in lookup_table.iter_mut().enumerate() {
            let mut v = i as u32;

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
impl CRCu32 {
    /// Get the current CRC value (it always returns a vec instance with a length corresponding to the CRC bits). You can continue calling `digest` method even after getting a CRC value.
    pub fn get_crc_vec_le(&mut self) -> Vec<u8> {
        let e = ((f64::from(self.bits) + 7f64) / 8f64) as u32;

        let e_dec = e - 1;

        let mut vec = Vec::with_capacity(e as usize);

        let crc = self.get_crc();

        let o = e_dec * 8;

        for i in 0..e {
            vec.push((crc << ((e_dec - i) * 8) >> o) as u8);
        }

        vec
    }

    /// Get the current CRC value (it always returns a vec instance with a length corresponding to the CRC bits). You can continue calling `digest` method even after getting a CRC value.
    pub fn get_crc_vec_be(&mut self) -> Vec<u8> {
        let e = ((f64::from(self.bits) + 7f64) / 8f64) as u32;

        let e_dec = e - 1;

        let mut vec = Vec::with_capacity(e as usize);

        let crc = self.get_crc();

        let o = e_dec * 8;

        for i in 0..e {
            vec.push((crc << (i * 8) >> o) as u8);
        }

        vec
    }
}

#[cfg(feature = "heapless")]
impl CRCu32 {
    /// Get the current CRC value (it always returns a heapless vec instance with a length corresponding to the CRC bits). You can continue calling `digest` method even after getting a CRC value.
    pub fn get_crc_heapless_vec_le(&mut self) -> HeaplessVec<u8, U4> {
        let e = ((f64::from(self.bits) + 7f64) / 8f64) as u32;

        let e_dec = e - 1;

        let mut vec = HeaplessVec::new();

        let crc = self.get_crc();

        let o = e_dec * 8;

        for i in 0..e {
            vec.push((crc << ((e_dec - i) * 8) >> o) as u8).unwrap();
        }

        vec
    }

    /// Get the current CRC value (it always returns a heapless vec instance with a length corresponding to the CRC bits). You can continue calling `digest` method even after getting a CRC value.
    pub fn get_crc_heapless_vec_be(&mut self) -> HeaplessVec<u8, U4> {
        let e = ((f64::from(self.bits) + 7f64) / 8f64) as u32;

        let e_dec = e - 1;

        let mut vec = HeaplessVec::new();

        let crc = self.get_crc();

        let o = e_dec * 8;

        for i in 0..e {
            vec.push((crc << (i * 8) >> o) as u8).unwrap();
        }

        vec
    }
}

#[allow(clippy::unreadable_literal)]
impl CRCu32 {
    pub fn crc17can() -> CRCu32 {
        Self::create_crc(0x0001685B, 17, 0x00000000, 0x00000000, false)
    }

    pub fn crc21can() -> CRCu32 {
        Self::create_crc(0x00102899, 21, 0x00000000, 0x00000000, false)
    }

    pub fn crc24() -> CRCu32 {
        //        Self::create_crc(0x00864CFB, 24, 0x00B704CE, 0x00000000, false)

        let lookup_table = NO_REF_24_00864CFB;
        Self::create_crc_with_exists_lookup_table(lookup_table, 24, 0x00B704CE, 0x00000000, false)
    }

    pub fn crc24ble() -> CRCu32 {
        //        Self::create_crc(0x00DA6000, 24, 0x00555555, 0x00000000, true)

        let lookup_table = REF_24_00DA6000;
        Self::create_crc_with_exists_lookup_table(lookup_table, 24, 0x00555555, 0x00000000, true)
    }

    pub fn crc24flexray_a() -> CRCu32 {
        //         Self::create_crc(0x005D6DCB, 24, 0x00FEDCBA, 0x00000000, false)

        let lookup_table = NO_REF_24_005D6DCB;
        Self::create_crc_with_exists_lookup_table(lookup_table, 24, 0x00FEDCBA, 0x00000000, false)
    }

    pub fn crc24flexray_b() -> CRCu32 {
        //         Self::create_crc(0x005D6DCB, 24, 0x00ABCDEF, 0x00000000, false)

        let lookup_table = NO_REF_24_005D6DCB;
        Self::create_crc_with_exists_lookup_table(lookup_table, 24, 0x00ABCDEF, 0x00000000, false)
    }

    pub fn crc24lte_a() -> CRCu32 {
        //         Self::create_crc(0x00864CFB, 24, 0x00000000, 0x00000000, false)

        let lookup_table = NO_REF_24_00864CFB;
        Self::create_crc_with_exists_lookup_table(lookup_table, 24, 0x00000000, 0x00000000, false)
    }

    pub fn crc24lte_b() -> CRCu32 {
        //         Self::create_crc(0x00800063, 24, 0x00000000, 0x00000000, false)

        let lookup_table = NO_REF_24_00800063;
        Self::create_crc_with_exists_lookup_table(lookup_table, 24, 0x00000000, 0x00000000, false)
    }

    pub fn crc24os9() -> CRCu32 {
        //         Self::create_crc(0x00800063, 24, 0x00FFFFFF, 0x00FFFFFF, false)

        let lookup_table = NO_REF_24_00800063;
        Self::create_crc_with_exists_lookup_table(lookup_table, 24, 0x00FFFFFF, 0x00FFFFFF, false)
    }

    pub fn crc30cdma() -> CRCu32 {
        Self::create_crc(0x2030B9C7, 30, 0x3FFFFFFF, 0x3FFFFFFF, false)
    }

    pub fn crc32() -> CRCu32 {
        //         Self::create_crc(0xEDB88320, 32, 0xFFFFFFFF, 0xFFFFFFFF, true)

        let lookup_table = REF_32_EDB88320;
        Self::create_crc_with_exists_lookup_table(lookup_table, 32, 0xFFFFFFFF, 0xFFFFFFFF, true)
    }

    pub fn crc32mhash() -> CRCu32 {
        let mut crc;

        //         crc = Self::create_crc(0x04C11DB7, 32, 0xFFFFFFFF, 0xFFFFFFFF, false);

        let lookup_table = NO_REF_32_04C11DB7;
        crc = Self::create_crc_with_exists_lookup_table(
            lookup_table,
            32,
            0xFFFFFFFF,
            0xFFFFFFFF,
            false,
        );

        crc.reorder = true;

        crc
    }

    pub fn crc32bzip2() -> CRCu32 {
        //        Self::create_crc(0x04C11DB7, 32, 0xFFFFFFFF, 0xFFFFFFFF, false)

        let lookup_table = NO_REF_32_04C11DB7;
        Self::create_crc_with_exists_lookup_table(lookup_table, 32, 0xFFFFFFFF, 0xFFFFFFFF, false)
    }

    pub fn crc32c() -> CRCu32 {
        // Self::create_crc(0x82F63B78, 32, 0xFFFFFFFF, 0xFFFFFFFF, true)

        let lookup_table = REF_32_82F63B78;
        Self::create_crc_with_exists_lookup_table(lookup_table, 32, 0xFFFFFFFF, 0xFFFFFFFF, true)
    }

    pub fn crc32d() -> CRCu32 {
        //         Self::create_crc(0xD419CC15, 32, 0xFFFFFFFF, 0xFFFFFFFF, true)

        let lookup_table = REF_32_D419CC15;
        Self::create_crc_with_exists_lookup_table(lookup_table, 32, 0xFFFFFFFF, 0xFFFFFFFF, true)
    }

    pub fn crc32mpeg2() -> CRCu32 {
        //         Self::create_crc(0x04C11DB7, 32, 0xFFFFFFFF, 0x00000000, false)

        let lookup_table = NO_REF_32_04C11DB7;
        Self::create_crc_with_exists_lookup_table(lookup_table, 32, 0xFFFFFFFF, 0x00000000, false)
    }

    pub fn crc32posix() -> CRCu32 {
        //         Self::create_crc(0x04C11DB7, 32, 0x00000000, 0xFFFFFFFF, false)

        let lookup_table = NO_REF_32_04C11DB7;
        Self::create_crc_with_exists_lookup_table(lookup_table, 32, 0x00000000, 0xFFFFFFFF, false)
    }

    pub fn crc32q() -> CRCu32 {
        //         Self::create_crc(0x814141AB, 32, 0x00000000, 0x00000000, false)

        let lookup_table = NO_REF_32_814141AB;
        Self::create_crc_with_exists_lookup_table(lookup_table, 32, 0x00000000, 0x00000000, false)
    }

    pub fn crc32jamcrc() -> CRCu32 {
        //         Self::create_crc(0xEDB88320, 32, 0xFFFFFFFF, 0x00000000, true)

        let lookup_table = REF_32_EDB88320;
        Self::create_crc_with_exists_lookup_table(lookup_table, 32, 0xFFFFFFFF, 0x00000000, true)
    }

    pub fn crc32xfer() -> CRCu32 {
        //         Self::create_crc(0x000000AF, 32, 0x00000000, 0x00000000, false)

        let lookup_table = NO_REF_32_000000AF;
        Self::create_crc_with_exists_lookup_table(lookup_table, 32, 0x00000000, 0x00000000, false)
    }
}

#[cfg(all(feature = "development", test))]
mod tests {
    use super::CRCu32;

    use alloc::fmt::Write;
    use alloc::string::String;

    #[test]
    fn print_lookup_table() {
        let crc = CRCu32::crc24ble();

        let mut s = String::new();

        for n in crc.lookup_table.iter().take(255) {
            s.write_fmt(format_args!("{}u32, ", n)).unwrap();
        }

        s.write_fmt(format_args!("{}u32", crc.lookup_table[255])).unwrap();

        println!("let lookup_table = [{}];", s);
    }
}
