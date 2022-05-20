#[cfg(feature = "alloc")]
use alloc::fmt::{self, Debug, Display, Formatter};
#[cfg(feature = "alloc")]
use alloc::vec::Vec;

#[cfg(feature = "heapless")]
use heapless::Vec as HeaplessVec;

use crate::constants::crc_u32::*;
use crate::lookup_table::LookUpTable;

#[allow(clippy::upper_case_acronyms)]
/// This struct can help you compute a CRC-32 (or CRC-x where **x** is equal or less than `32`) value.
pub struct CRCu32 {
    by_table: bool,
    poly: u32,
    lookup_table: LookUpTable<u32>,
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
            debug_helper::impl_debug_for_struct!(CRCu64, f, self, let .lookup_table = self.lookup_table.as_ref(), (.sum, "0x{:08X}", self.sum), .bits, (.initial, "0x{:08X}", self.initial), (.final_xor, "0x{:08X}", self.final_xor), .reflect, .reorder);
        } else {
            debug_helper::impl_debug_for_struct!(CRCu64, f, self, (.poly, "0x{:08X}", self.poly), (.sum, "0x{:08X}", self.sum), .bits, (.initial, "0x{:08X}", self.initial), (.final_xor, "0x{:08X}", self.final_xor), .reflect, .reorder);
        }
    }
}

#[cfg(feature = "alloc")]
impl Display for CRCu32 {
    #[inline]
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        f.write_fmt(format_args!("0x{:01$X}", self.get_crc(), (self.bits as usize + 3) >> 2))
    }
}

impl CRCu32 {
    /// Create a `CRCu32` instance by providing the length of bits, expression, reflection, an initial value and a final xor value.
    pub fn create_crc(poly: u32, bits: u8, initial: u32, final_xor: u32, reflect: bool) -> CRCu32 {
        debug_assert!(bits <= 32 && bits > 0);

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
                LookUpTable::Static(&[0u32; 256]),
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
        lookup_table: LookUpTable<u32>,
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
        lookup_table: LookUpTable<u32>,
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
                    let index = ((self.sum >> u32::from(self.bits - 8)) as u8 ^ n) as usize;
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

    /// Get the current CRC value (it always returns a `u32` value). You can continue calling `digest` method even after getting a CRC value.
    pub fn get_crc(&self) -> u32 {
        let sum = if self.by_table || !self.reflect {
            (self.sum ^ self.final_xor) & self.mask
        } else {
            (self.reflect_method(self.sum) ^ self.final_xor) & self.mask
        };

        if self.reorder {
            let mut new_sum = 0;

            let e = (self.bits as u32 + 7) >> 3;

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

    fn crc_table(poly: u32, bits: u8) -> [u32; 256] {
        let mut lookup_table = [0u32; 256];

        let mask1 = 1u32 << u32::from(bits - 1);

        let mask2 = ((mask1 - 1) << 1) | 1;

        for (i, e) in lookup_table.iter_mut().enumerate() {
            let mut v = i as u32;

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
impl CRCu32 {
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

        crc.to_be_bytes()[(4 - e)..].to_vec()
    }
}

#[cfg(feature = "heapless")]
impl CRCu32 {
    /// Get the current CRC value (it always returns a heapless vec instance with a length corresponding to the CRC bits). You can continue calling `digest` method even after getting a CRC value.
    #[inline]
    pub fn get_crc_heapless_vec_le(&mut self) -> HeaplessVec<u8, 4> {
        let crc = self.get_crc();

        let e = (self.bits as usize + 7) >> 3;

        let mut vec = HeaplessVec::new();

        vec.extend_from_slice(&crc.to_le_bytes()[..e]).unwrap();

        vec
    }

    /// Get the current CRC value (it always returns a heapless vec instance with a length corresponding to the CRC bits). You can continue calling `digest` method even after getting a CRC value.
    #[inline]
    pub fn get_crc_heapless_vec_be(&mut self) -> HeaplessVec<u8, 4> {
        let crc = self.get_crc();

        let e = (self.bits as usize + 7) >> 3;

        let mut vec = HeaplessVec::new();

        vec.extend_from_slice(&crc.to_be_bytes()[(4 - e)..]).unwrap();

        vec
    }
}

impl CRCu32 {
    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0x04F03|0x1685B|0x00000|false|0x00000|
    ///
    /// ```
    /// # use crc_any::CRCu32;
    /// let mut crc = CRCu32::crc17can();
    /// crc.digest(b"123456789");
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0x04F03\", &crc.to_string());")]
    /// ```
    pub fn crc17can() -> CRCu32 {
        Self::create_crc(0x0001685B, 17, 0x00000000, 0x00000000, false)
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0x0ED841|0x102899|0x000000|false|0x000000|
    ///
    /// ```
    /// # use crc_any::CRCu32;
    /// let mut crc = CRCu32::crc21can();
    /// crc.digest(b"123456789");
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0x0ED841\", &crc.to_string());")]
    /// ```
    pub fn crc21can() -> CRCu32 {
        Self::create_crc(0x00102899, 21, 0x00000000, 0x00000000, false)
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0x21CF02|0x864CFB|0xB704CE|false|0x000000|
    ///
    /// ```
    /// # use crc_any::CRCu32;
    /// let mut crc = CRCu32::crc24();
    /// crc.digest(b"123456789");
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0x21CF02\", &crc.to_string());")]
    /// ```
    pub fn crc24() -> CRCu32 {
        // Self::create_crc(0x00864CFB, 24, 0x00B704CE, 0x00000000, false)

        let lookup_table = LookUpTable::Static(&NO_REF_24_00864CFB);
        Self::create_crc_with_exists_lookup_table(lookup_table, 24, 0x00B704CE, 0x00000000, false)
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0xC25A56|0x00065B (rev: 0xDA6000)|0x555555|true|0x000000|
    ///
    /// ```
    /// # use crc_any::CRCu32;
    /// let mut crc = CRCu32::crc24ble();
    /// crc.digest(b"123456789");
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0xC25A56\", &crc.to_string());")]
    /// ```
    pub fn crc24ble() -> CRCu32 {
        // Self::create_crc(0x00DA6000, 24, 0x00555555, 0x00000000, true)

        let lookup_table = LookUpTable::Static(&REF_24_00DA6000);
        Self::create_crc_with_exists_lookup_table(lookup_table, 24, 0x00555555, 0x00000000, true)
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0x7979BD|0x5D6DCB|0xFEDCBA|false|0x000000|
    ///
    /// ```
    /// # use crc_any::CRCu32;
    /// let mut crc = CRCu32::crc24flexray_a();
    /// crc.digest(b"123456789");
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0x7979BD\", &crc.to_string());")]
    /// ```
    pub fn crc24flexray_a() -> CRCu32 {
        // Self::create_crc(0x005D6DCB, 24, 0x00FEDCBA, 0x00000000, false)

        let lookup_table = LookUpTable::Static(&NO_REF_24_005D6DCB);
        Self::create_crc_with_exists_lookup_table(lookup_table, 24, 0x00FEDCBA, 0x00000000, false)
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0x1F23B8|0x5D6DCB|0xABCDEF|false|0x000000|
    ///
    /// ```
    /// # use crc_any::CRCu32;
    /// let mut crc = CRCu32::crc24flexray_b();
    /// crc.digest(b"123456789");
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0x1F23B8\", &crc.to_string());")]
    /// ```
    pub fn crc24flexray_b() -> CRCu32 {
        // Self::create_crc(0x005D6DCB, 24, 0x00ABCDEF, 0x00000000, false)

        let lookup_table = LookUpTable::Static(&NO_REF_24_005D6DCB);
        Self::create_crc_with_exists_lookup_table(lookup_table, 24, 0x00ABCDEF, 0x00000000, false)
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0xCDE703|0x864CFB|0x000000|false|0x000000|
    ///
    /// ```
    /// # use crc_any::CRCu32;
    /// let mut crc = CRCu32::crc24lte_a();
    /// crc.digest(b"123456789");
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0xCDE703\", &crc.to_string());")]
    /// ```
    pub fn crc24lte_a() -> CRCu32 {
        // Self::create_crc(0x00864CFB, 24, 0x00000000, 0x00000000, false)

        let lookup_table = LookUpTable::Static(&NO_REF_24_00864CFB);
        Self::create_crc_with_exists_lookup_table(lookup_table, 24, 0x00000000, 0x00000000, false)
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0x23EF52|0x800063|0x000000|false|0x000000|
    ///
    /// ```
    /// # use crc_any::CRCu32;
    /// let mut crc = CRCu32::crc24lte_b();
    /// crc.digest(b"123456789");
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0x23EF52\", &crc.to_string());")]
    /// ```
    pub fn crc24lte_b() -> CRCu32 {
        // Self::create_crc(0x00800063, 24, 0x00000000, 0x00000000, false)

        let lookup_table = LookUpTable::Static(&NO_REF_24_00800063);
        Self::create_crc_with_exists_lookup_table(lookup_table, 24, 0x00000000, 0x00000000, false)
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0x200FA5|0x800063|0xFFFFFF|false|0xFFFFFF|
    ///
    /// ```
    /// # use crc_any::CRCu32;
    /// let mut crc = CRCu32::crc24os9();
    /// crc.digest(b"123456789");
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0x200FA5\", &crc.to_string());")]
    /// ```
    pub fn crc24os9() -> CRCu32 {
        // Self::create_crc(0x00800063, 24, 0x00FFFFFF, 0x00FFFFFF, false)

        let lookup_table = LookUpTable::Static(&NO_REF_24_00800063);
        Self::create_crc_with_exists_lookup_table(lookup_table, 24, 0x00FFFFFF, 0x00FFFFFF, false)
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0x04C34ABF|0x2030B9C7|0x3FFFFFFF|false|0x3FFFFFFF|
    ///
    /// ```
    /// # use crc_any::CRCu32;
    /// let mut crc = CRCu32::crc30cdma();
    /// crc.digest(b"123456789");
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0x04C34ABF\", &crc.to_string());")]
    /// ```
    pub fn crc30cdma() -> CRCu32 {
        Self::create_crc(0x2030B9C7, 30, 0x3FFFFFFF, 0x3FFFFFFF, false)
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0xCBF43926|0x04C11DB7 (rev: 0xEDB88320)|0xFFFFFFFF|true|0xFFFFFFFF|
    ///
    /// ```
    /// # use crc_any::CRCu32;
    /// let mut crc = CRCu32::crc32();
    /// crc.digest(b"123456789");
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0xCBF43926\", &crc.to_string());")]
    /// ```
    pub fn crc32() -> CRCu32 {
        // Self::create_crc(0xEDB88320, 32, 0xFFFFFFFF, 0xFFFFFFFF, true)

        let lookup_table = LookUpTable::Static(&REF_32_EDB88320);
        Self::create_crc_with_exists_lookup_table(lookup_table, 32, 0xFFFFFFFF, 0xFFFFFFFF, true)
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0x181989FC|0x04C11DB7|0xFFFFFFFF|false|0xFFFFFFFF|
    ///
    /// **Output will be reversed by bytes.**
    ///
    /// ```
    /// # use crc_any::CRCu32;
    /// let mut crc = CRCu32::crc32mhash();
    /// crc.digest(b"123456789");
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0x181989FC\", &crc.to_string());")]
    /// ```
    pub fn crc32mhash() -> CRCu32 {
        // let mut crc = Self::create_crc(0x04C11DB7, 32, 0xFFFFFFFF, 0xFFFFFFFF, false);

        let lookup_table = LookUpTable::Static(&NO_REF_32_04C11DB7);

        let mut crc = Self::create_crc_with_exists_lookup_table(
            lookup_table,
            32,
            0xFFFFFFFF,
            0xFFFFFFFF,
            false,
        );

        crc.reorder = true;

        crc
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0xFC891918|0x04C11DB7|0xFFFFFFFF|false|0xFFFFFFFF|
    ///
    /// ```
    /// # use crc_any::CRCu32;
    /// let mut crc = CRCu32::crc32bzip2();
    /// crc.digest(b"123456789");
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0xFC891918\", &crc.to_string());")]
    /// ```
    pub fn crc32bzip2() -> CRCu32 {
        // Self::create_crc(0x04C11DB7, 32, 0xFFFFFFFF, 0xFFFFFFFF, false)

        let lookup_table = LookUpTable::Static(&NO_REF_32_04C11DB7);
        Self::create_crc_with_exists_lookup_table(lookup_table, 32, 0xFFFFFFFF, 0xFFFFFFFF, false)
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0xE3069283|0x1EDC6F41 (rev: 0x82F63B78)|0xFFFFFFFF|true|0xFFFFFFFF|
    ///
    /// ```
    /// # use crc_any::CRCu32;
    /// let mut crc = CRCu32::crc32c();
    /// crc.digest(b"123456789");
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0xE3069283\", &crc.to_string());")]
    /// ```
    pub fn crc32c() -> CRCu32 {
        // Self::create_crc(0x82F63B78, 32, 0xFFFFFFFF, 0xFFFFFFFF, true)

        let lookup_table = LookUpTable::Static(&REF_32_82F63B78);
        Self::create_crc_with_exists_lookup_table(lookup_table, 32, 0xFFFFFFFF, 0xFFFFFFFF, true)
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0x87315576|0xA833982B (rev: 0xD419CC15)|0xFFFFFFFF|true|0xFFFFFFFF|
    ///
    /// ```
    /// # use crc_any::CRCu32;
    /// let mut crc = CRCu32::crc32d();
    /// crc.digest(b"123456789");
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0x87315576\", &crc.to_string());")]
    /// ```
    pub fn crc32d() -> CRCu32 {
        // Self::create_crc(0xD419CC15, 32, 0xFFFFFFFF, 0xFFFFFFFF, true)

        let lookup_table = LookUpTable::Static(&REF_32_D419CC15);
        Self::create_crc_with_exists_lookup_table(lookup_table, 32, 0xFFFFFFFF, 0xFFFFFFFF, true)
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0x0376E6E7|0x04C11DB7|0xFFFFFFFF|false|0x00000000|
    ///
    /// ```
    /// # use crc_any::CRCu32;
    /// let mut crc = CRCu32::crc32mpeg2();
    /// crc.digest(b"123456789");
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0x0376E6E7\", &crc.to_string());")]
    /// ```
    pub fn crc32mpeg2() -> CRCu32 {
        // Self::create_crc(0x04C11DB7, 32, 0xFFFFFFFF, 0x00000000, false)

        let lookup_table = LookUpTable::Static(&NO_REF_32_04C11DB7);
        Self::create_crc_with_exists_lookup_table(lookup_table, 32, 0xFFFFFFFF, 0x00000000, false)
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0x765E7680|0x04C11DB7|0x00000000|false|0xFFFFFFFF|
    ///
    /// ```
    /// # use crc_any::CRCu32;
    /// let mut crc = CRCu32::crc32posix();
    /// crc.digest(b"123456789");
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0x765E7680\", &crc.to_string());")]
    /// ```
    pub fn crc32posix() -> CRCu32 {
        // Self::create_crc(0x04C11DB7, 32, 0x00000000, 0xFFFFFFFF, false)

        let lookup_table = LookUpTable::Static(&NO_REF_32_04C11DB7);
        Self::create_crc_with_exists_lookup_table(lookup_table, 32, 0x00000000, 0xFFFFFFFF, false)
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0x3010BF7F|0x814141AB|0x00000000|false|0x00000000|
    ///
    /// ```
    /// # use crc_any::CRCu32;
    /// let mut crc = CRCu32::crc32q();
    /// crc.digest(b"123456789");
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0x3010BF7F\", &crc.to_string());")]
    /// ```
    pub fn crc32q() -> CRCu32 {
        // Self::create_crc(0x814141AB, 32, 0x00000000, 0x00000000, false)

        let lookup_table = LookUpTable::Static(&NO_REF_32_814141AB);
        Self::create_crc_with_exists_lookup_table(lookup_table, 32, 0x00000000, 0x00000000, false)
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0x340BC6D9|0x04C11DB7 (rev: 0xEDB88320)|0xFFFFFFFF|true|0x00000000|
    ///
    /// ```
    /// # use crc_any::CRCu32;
    /// let mut crc = CRCu32::crc32jamcrc();
    /// crc.digest(b"123456789");
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0x340BC6D9\", &crc.to_string());")]
    /// ```
    pub fn crc32jamcrc() -> CRCu32 {
        // Self::create_crc(0xEDB88320, 32, 0xFFFFFFFF, 0x00000000, true)

        let lookup_table = LookUpTable::Static(&REF_32_EDB88320);
        Self::create_crc_with_exists_lookup_table(lookup_table, 32, 0xFFFFFFFF, 0x00000000, true)
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0xBD0BE338|0x000000AF|0x00000000|false|0x00000000|
    ///
    /// ```
    /// # use crc_any::CRCu32;
    /// let mut crc = CRCu32::crc32xfer();
    /// crc.digest(b"123456789");
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0xBD0BE338\", &crc.to_string());")]
    /// ```
    pub fn crc32xfer() -> CRCu32 {
        // Self::create_crc(0x000000AF, 32, 0x00000000, 0x00000000, false)

        let lookup_table = LookUpTable::Static(&NO_REF_32_000000AF);
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
