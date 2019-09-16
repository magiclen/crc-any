#[cfg(feature = "alloc")]
use alloc::fmt::{self, Debug, Display, Formatter};
#[cfg(feature = "alloc")]
use alloc::vec::Vec;

#[cfg(feature = "heapless")]
use heapless::consts::U8;
#[cfg(feature = "heapless")]
use heapless::Vec as HeaplessVec;

use crate::constants::crc_u64::*;

/// This struct can help you compute a CRC-64 (or CRC-x where **x** is under `64`) value.
pub struct CRCu64 {
    by_table: bool,
    poly: u64,
    lookup_table: [u64; 256],
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
            impl_debug_for_struct!(CRCu64, f, self, let .lookup_table = self.lookup_table.as_ref(), (.sum, "0x{:016X}", self.sum), .bits, (.initial, "0x{:016X}", self.initial), (.final_xor, "0x{:016X}", self.final_xor), .reflect, .reorder);
        } else {
            impl_debug_for_struct!(CRCu64, f, self, (.poly, "0x{:016X}", self.poly), (.sum, "0x{:016X}", self.sum), .bits, (.initial, "0x{:016X}", self.initial), (.final_xor, "0x{:016X}", self.final_xor), .reflect, .reorder);
        }
    }
}

#[cfg(feature = "alloc")]
impl Display for CRCu64 {
    #[inline]
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        f.write_fmt(format_args!(
            "0x{:01$X}",
            self.get_crc(),
            ((f64::from(self.bits) + 3f64) / 4f64) as usize
        ))
    }
}

impl CRCu64 {
    /// Create a `CRCu64` instance by providing the length of bits, expression, reflection, an initial value and a final xor value.
    pub fn create_crc(poly: u64, bits: u8, initial: u64, final_xor: u64, reflect: bool) -> CRCu64 {
        debug_assert!(bits <= 64 && bits > 0);

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
            Self::create(false, [0u64; 256], poly, bits, initial, final_xor, reflect)
        }
    }

    /// Create a `CRCu64` instance by providing an existing lookup table, the length of bits, expression, reflection, an initial value and a final xor value.
    pub(crate) fn create_crc_with_exists_lookup_table(
        lookup_table: [u64; 256],
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
        lookup_table: [u64; 256],
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
                    let index = ((self.sum >> u64::from(self.bits - 8)) as u8 ^ n) as usize;
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

    /// Get the current CRC value (it always returns a `u64` value). You can continue calling `digest` method even after getting a CRC value.
    pub fn get_crc(&self) -> u64 {
        let sum = if self.by_table {
            (self.sum ^ self.final_xor) & self.mask
        } else if self.reflect {
            (self.reflect_method(self.sum) ^ self.final_xor) & self.mask
        } else {
            (self.sum ^ self.final_xor) & self.mask
        };

        if self.reorder {
            let mut new_sum = 0;

            let e = ((f64::from(self.bits) + 7f64) / 8f64) as u64;

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
    pub fn get_crc_vec_le(&mut self) -> Vec<u8> {
        let e = ((f64::from(self.bits) + 7f64) / 8f64) as u64;

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
        let e = ((f64::from(self.bits) + 7f64) / 8f64) as u64;

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
impl CRCu64 {
    /// Get the current CRC value (it always returns a heapless vec instance with a length corresponding to the CRC bits). You can continue calling `digest` method even after getting a CRC value.
    pub fn get_crc_heapless_vec_le(&mut self) -> HeaplessVec<u8, U8> {
        let e = ((f64::from(self.bits) + 7f64) / 8f64) as u64;

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
    pub fn get_crc_heapless_vec_be(&mut self) -> HeaplessVec<u8, U8> {
        let e = ((f64::from(self.bits) + 7f64) / 8f64) as u64;

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
impl CRCu64 {
    pub fn crc40gsm() -> CRCu64 {
        //         Self::create_crc(0x0000000004820009u64, 40, 0x0000000000000000, 0x000000FFFFFFFFFF, false)

        let lookup_table = NO_REF_40_0000000004820009;
        Self::create_crc_with_exists_lookup_table(
            lookup_table,
            40,
            0x0000000000000000,
            0x000000FFFFFFFFFF,
            false,
        )
    }

    pub fn crc64() -> CRCu64 {
        // Self::create_crc(0x42F0E1EBA9EA3693, 64, 0xFFFFFFFFFFFFFFFF, 0xFFFFFFFFFFFFFFFF, false)

        let lookup_table = NO_REF_64_42F0E1EBA9EA3693;
        Self::create_crc_with_exists_lookup_table(
            lookup_table,
            64,
            0xFFFFFFFFFFFFFFFF,
            0xFFFFFFFFFFFFFFFF,
            false,
        )
    }

    pub fn crc64iso() -> CRCu64 {
        // Self::create_crc(0xD800000000000000u64, 64, 0xFFFFFFFFFFFFFFFF, 0xFFFFFFFFFFFFFFFF, true)

        let lookup_table = REF_64_D800000000000000;
        Self::create_crc_with_exists_lookup_table(
            lookup_table,
            64,
            0xFFFFFFFFFFFFFFFF,
            0xFFFFFFFFFFFFFFFF,
            true,
        )
    }

    pub fn crc64jones() -> CRCu64 {
        //         Self::create_crc(0x95AC9329AC4BC9B5u64, 64, 0x0000000000000000, 0x0000000000000000, true)

        let lookup_table = REF_64_95AC9329AC4BC9B5;
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
