#[cfg(feature = "alloc")]
use alloc::fmt::{self, Debug, Display, Formatter};
#[cfg(feature = "alloc")]
use alloc::vec::Vec;

#[cfg(feature = "heapless")]
use heapless::consts::U2;
#[cfg(feature = "heapless")]
use heapless::Vec as HeaplessVec;

use crate::constants::crc_u16::*;

/// This struct can help you compute a CRC-16 (or CRC-x where **x** is under `16`) value.
pub struct CRCu16 {
    by_table: bool,
    poly: u16,
    lookup_table: [u16; 256],
    sum: u16,
    pub(crate) bits: u8,
    high_bit: u16,
    mask: u16,
    initial: u16,
    final_xor: u16,
    reflect: bool,
    reorder: bool,
}

#[cfg(feature = "alloc")]
impl Debug for CRCu16 {
    #[inline]
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        if self.by_table {
            impl_debug_for_struct!(CRCu64, f, self, let .lookup_table = self.lookup_table.as_ref(), (.sum, "0x{:04X}", self.sum), .bits, (.initial, "0x{:04X}", self.initial), (.final_xor, "0x{:04X}", self.final_xor), .reflect, .reorder);
        } else {
            impl_debug_for_struct!(CRCu64, f, self, (.poly, "0x{:04X}", self.poly), (.sum, "0x{:04X}", self.sum), .bits, (.initial, "0x{:04X}", self.initial), (.final_xor, "0x{:04X}", self.final_xor), .reflect, .reorder);
        }
    }
}

#[cfg(feature = "alloc")]
impl Display for CRCu16 {
    #[inline]
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        f.write_fmt(format_args!(
            "0x{:01$X}",
            self.get_crc(),
            ((f64::from(self.bits) + 3f64) / 4f64) as usize
        ))
    }
}

impl CRCu16 {
    /// Create a `CRCu16` instance by providing the length of bits, expression, reflection, an initial value and a final xor value.
    pub fn create_crc(poly: u16, bits: u8, initial: u16, final_xor: u16, reflect: bool) -> CRCu16 {
        debug_assert!(bits <= 16 && bits > 0);

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
            Self::create(false, [0u16; 256], poly, bits, initial, final_xor, reflect)
        }
    }

    #[inline]
    pub(crate) fn create_crc_with_exists_lookup_table(
        lookup_table: [u16; 256],
        bits: u8,
        initial: u16,
        final_xor: u16,
        reflect: bool,
    ) -> CRCu16 {
        debug_assert!(bits % 8 == 0);

        Self::create(true, lookup_table, 0, bits, initial, final_xor, reflect)
    }

    #[inline]
    fn create(
        by_table: bool,
        lookup_table: [u16; 256],
        mut poly: u16,
        bits: u8,
        initial: u16,
        final_xor: u16,
        reflect: bool,
    ) -> CRCu16 {
        let high_bit = 1 << u16::from(bits - 1);
        let mask = ((high_bit - 1) << 1) | 1;

        let sum = if reflect {
            Self::reflect_function(high_bit, initial)
        } else {
            initial
        };

        if !by_table && reflect {
            poly = Self::reflect_function(high_bit, poly);
        }

        CRCu16 {
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
    fn reflect_function(high_bit: u16, n: u16) -> u16 {
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
    fn reflect_method(&self, n: u16) -> u16 {
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
                    let index = ((self.sum >> u16::from(self.bits - 8)) as u8 ^ n) as usize;
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

    /// Get the current CRC value (it always returns a `u16` value). You can continue calling `digest` method even after getting a CRC value.
    pub fn get_crc(&self) -> u16 {
        let sum = if self.by_table {
            (self.sum ^ self.final_xor) & self.mask
        } else if self.reflect {
            (self.reflect_method(self.sum) ^ self.final_xor) & self.mask
        } else {
            (self.sum ^ self.final_xor) & self.mask
        };

        if self.reorder {
            let mut new_sum = 0;

            let e = ((f64::from(self.bits) + 7f64) / 8f64) as u16;

            let e_dec = e - 1;

            for i in 0..e {
                new_sum |= ((sum >> ((e_dec - i) * 8)) & 0xFF) << (i * 8);
            }

            new_sum
        } else {
            sum
        }
    }

    fn crc_reflect_table(poly_rev: u16) -> [u16; 256] {
        let mut lookup_table = [0u16; 256];

        for (i, e) in lookup_table.iter_mut().enumerate() {
            let mut v = i as u16;

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

    fn crc_table(poly: u16, bits: u8) -> [u16; 256] {
        let mut lookup_table = [0u16; 256];

        let mask1 = 1u16 << u16::from(bits - 1);

        let mask2 = ((mask1 - 1) << 1) | 1;

        for (i, e) in lookup_table.iter_mut().enumerate() {
            let mut v = i as u16;

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
impl CRCu16 {
    /// Get the current CRC value (it always returns a vec instance with a length corresponding to the CRC bits). You can continue calling `digest` method even after getting a CRC value.
    pub fn get_crc_vec_le(&mut self) -> Vec<u8> {
        let e = ((f64::from(self.bits) + 7f64) / 8f64) as u16;

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
        let e = ((f64::from(self.bits) + 7f64) / 8f64) as u16;

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
impl CRCu16 {
    /// Get the current CRC value (it always returns a heapless vec instance with a length corresponding to the CRC bits). You can continue calling `digest` method even after getting a CRC value.
    pub fn get_crc_heapless_vec_le(&mut self) -> HeaplessVec<u8, U2> {
        let e = ((f64::from(self.bits) + 7f64) / 8f64) as u16;

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
    pub fn get_crc_heapless_vec_be(&mut self) -> HeaplessVec<u8, U2> {
        let e = ((f64::from(self.bits) + 7f64) / 8f64) as u16;

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
impl CRCu16 {
    pub fn crc10() -> CRCu16 {
        Self::create_crc(0x0233, 10, 0x0000, 0x0000, false)
    }

    pub fn crc10cdma2000() -> CRCu16 {
        Self::create_crc(0x03D9, 10, 0x03FF, 0x0000, false)
    }

    pub fn crc10gsm() -> CRCu16 {
        Self::create_crc(0x0175, 10, 0x0000, 0x03FF, false)
    }

    pub fn crc11() -> CRCu16 {
        Self::create_crc(0x0385, 11, 0x001a, 0x0000, false)
    }

    pub fn crc12() -> CRCu16 {
        Self::create_crc(0x080F, 12, 0x0000, 0x0000, false)
    }

    pub fn crc12cdma2000() -> CRCu16 {
        Self::create_crc(0x0F13, 12, 0x0FFF, 0x0000, false)
    }

    pub fn crc12gsm() -> CRCu16 {
        Self::create_crc(0x0D31, 12, 0x0000, 0x0FFF, false)
    }

    pub fn crc13bbc() -> CRCu16 {
        Self::create_crc(0x1CF5, 13, 0x0000, 0x0000, false)
    }

    pub fn crc14darc() -> CRCu16 {
        Self::create_crc(0x2804, 14, 0x0000, 0x0000, true)
    }

    pub fn crc14gsm() -> CRCu16 {
        Self::create_crc(0x202d, 14, 0x0000, 0x3FFF, false)
    }

    pub fn crc15can() -> CRCu16 {
        Self::create_crc(0x4599, 15, 0x0000, 0x0000, false)
    }

    pub fn crc15mpt1327() -> CRCu16 {
        Self::create_crc(0x6815, 15, 0x0000, 0x0001, false)
    }

    pub fn crc16() -> CRCu16 {
        //         Self::create_crc(0xA001, 16, 0x0000, 0x0000, true)

        let lookup_table = REF_16_A001;
        Self::create_crc_with_exists_lookup_table(lookup_table, 16, 0x0000, 0x0000, true)
    }

    pub fn crc16ccitt_false() -> CRCu16 {
        //         Self::create_crc(0x1021, 16, 0xFFFF, 0x0000, false)

        let lookup_table = NO_REF_16_1021;
        Self::create_crc_with_exists_lookup_table(lookup_table, 16, 0xFFFF, 0x0000, false)
    }

    pub fn crc16aug_ccitt() -> CRCu16 {
        //         Self::create_crc(0x1021, 16, 0x1D0F, 0x0000, false)

        let lookup_table = NO_REF_16_1021;
        Self::create_crc_with_exists_lookup_table(lookup_table, 16, 0x1D0F, 0x0000, false)
    }

    pub fn crc16buypass() -> CRCu16 {
        //         Self::create_crc(0x8005, 16, 0x0000, 0x0000, false)

        let lookup_table = NO_REF_16_8005;
        Self::create_crc_with_exists_lookup_table(lookup_table, 16, 0x0000, 0x0000, false)
    }

    pub fn crc16cdma2000() -> CRCu16 {
        //         Self::create_crc(0xC867, 16, 0xFFFF, 0x0000, false)

        let lookup_table = NO_REF_16_C867;
        Self::create_crc_with_exists_lookup_table(lookup_table, 16, 0xFFFF, 0x0000, false)
    }

    pub fn crc16dds_110() -> CRCu16 {
        //         Self::create_crc(0x8005, 16, 0x800D, 0x0000, false)

        let lookup_table = NO_REF_16_8005;
        Self::create_crc_with_exists_lookup_table(lookup_table, 16, 0x800D, 0x0000, false)
    }

    pub fn crc16dect_r() -> CRCu16 {
        //         Self::create_crc(0x0589, 16, 0x0000, 0x0001, false)

        let lookup_table = NO_REF_16_0589;
        Self::create_crc_with_exists_lookup_table(lookup_table, 16, 0x0000, 0x0001, false)
    }

    pub fn crc16dect_x() -> CRCu16 {
        //         Self::create_crc(0x0589, 16, 0x0000, 0x0000, false)

        let lookup_table = NO_REF_16_0589;
        Self::create_crc_with_exists_lookup_table(lookup_table, 16, 0x0000, 0x0000, false)
    }

    pub fn crc16dnp() -> CRCu16 {
        //         Self::create_crc(0xA6BC, 16, 0x0000, 0xFFFF, true)

        let lookup_table = REF_16_A6BC;
        Self::create_crc_with_exists_lookup_table(lookup_table, 16, 0x0000, 0xFFFF, true)
    }

    pub fn crc16en_13757() -> CRCu16 {
        //         Self::create_crc(0x3D65, 16, 0x0000, 0xFFFF, false)

        let lookup_table = NO_REF_16_3D65;
        Self::create_crc_with_exists_lookup_table(lookup_table, 16, 0x0000, 0xFFFF, false)
    }

    pub fn crc16genibus() -> CRCu16 {
        //         Self::create_crc(0x1021, 16, 0xFFFF, 0xFFFF, false)

        let lookup_table = NO_REF_16_1021;
        Self::create_crc_with_exists_lookup_table(lookup_table, 16, 0xFFFF, 0xFFFF, false)
    }

    pub fn crc16maxim() -> CRCu16 {
        //         Self::create_crc(0xA001, 16, 0x0000, 0xFFFF, true)

        let lookup_table = REF_16_A001;
        Self::create_crc_with_exists_lookup_table(lookup_table, 16, 0x0000, 0xFFFF, true)
    }

    pub fn crc16mcrf4cc() -> CRCu16 {
        //         Self::create_crc(0x8408, 16, 0xFFFF, 0x0000, true)

        let lookup_table = REF_16_8408;
        Self::create_crc_with_exists_lookup_table(lookup_table, 16, 0xFFFF, 0x0000, true)
    }

    pub fn crc16riello() -> CRCu16 {
        //        Self::create_crc(0x8408, 16, 0xB2AA, 0x0000, true)

        let lookup_table = REF_16_8408;
        Self::create_crc_with_exists_lookup_table(lookup_table, 16, 0xB2AA, 0x0000, true)
    }

    pub fn crc16t10_dif() -> CRCu16 {
        //         Self::create_crc(0x8BB7, 16, 0x0000, 0x0000, false)

        let lookup_table = NO_REF_16_8BB7;
        Self::create_crc_with_exists_lookup_table(lookup_table, 16, 0x0000, 0x0000, false)
    }

    pub fn crc16teledisk() -> CRCu16 {
        //         Self::create_crc(0xA097, 16, 0x0000, 0x0000, false)

        let lookup_table = REF_16_A097;
        Self::create_crc_with_exists_lookup_table(lookup_table, 16, 0x0000, 0x0000, false)
    }

    pub fn crc16tms13157() -> CRCu16 {
        //         Self::create_crc(0x8408, 16, 0x89EC, 0x0000, true)

        let lookup_table = REF_16_8408;
        Self::create_crc_with_exists_lookup_table(lookup_table, 16, 0x89EC, 0x0000, true)
    }

    pub fn crc16usb() -> CRCu16 {
        //         Self::create_crc(0xA001, 16, 0xFFFF, 0xFFFF, true)

        let lookup_table = REF_16_A001;
        Self::create_crc_with_exists_lookup_table(lookup_table, 16, 0xFFFF, 0xFFFF, true)
    }

    pub fn crc_a() -> CRCu16 {
        //         Self::create_crc(0x8408, 16, 0xC6C6, 0x0000, true)

        let lookup_table = REF_16_8408;
        Self::create_crc_with_exists_lookup_table(lookup_table, 16, 0xC6C6, 0x0000, true)
    }

    pub fn crc16kermit() -> CRCu16 {
        //         Self::create_crc(0x8408, 16, 0x0000, 0x0000, true)

        let lookup_table = REF_16_8408;
        Self::create_crc_with_exists_lookup_table(lookup_table, 16, 0x0000, 0x0000, true)
    }

    pub fn crc16modbus() -> CRCu16 {
        //         Self::create_crc(0xA001, 16, 0xFFFF, 0x0000, true)

        let lookup_table = REF_16_A001;
        Self::create_crc_with_exists_lookup_table(lookup_table, 16, 0xFFFF, 0x0000, true)
    }

    pub fn crc16_x25() -> CRCu16 {
        //         Self::create_crc(0x8408, 16, 0xFFFF, 0xFFFF, true)

        let lookup_table = REF_16_8408;
        Self::create_crc_with_exists_lookup_table(lookup_table, 16, 0xFFFF, 0xFFFF, true)
    }

    pub fn crc16xmodem() -> CRCu16 {
        //         Self::create_crc(0x1021, 16, 0x0000, 0x0000, false)

        let lookup_table = NO_REF_16_1021;
        Self::create_crc_with_exists_lookup_table(lookup_table, 16, 0x0000, 0x0000, false)
    }
}

#[cfg(all(feature = "development", test))]
mod tests {
    use super::CRCu16;

    use alloc::fmt::Write;
    use alloc::string::String;

    #[test]
    fn print_lookup_table() {
        let crc = CRCu16::crc16kermit();

        let mut s = String::new();

        for n in crc.lookup_table.iter().take(255) {
            s.write_fmt(format_args!("{}u16, ", n)).unwrap();
        }

        s.write_fmt(format_args!("{}u16", crc.lookup_table[255])).unwrap();

        println!("let lookup_table = [{}];", s);
    }
}
