#[cfg(feature = "alloc")]
use alloc::vec::Vec;
#[cfg(feature = "alloc")]
use core::fmt::Debug;
use core::fmt::{self, Display, Formatter};

#[cfg(feature = "heapless")]
use heapless::Vec as HeaplessVec;

#[cfg(feature = "slicing-by-8")]
use crate::lookup_table::slice_by_8;
use crate::{
    constants::crc_u32::*,
    hardware,
    lookup_table::{LookUpTable, SLICES, Tables},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Accelerator {
    None,
    Crc32,
    Crc32c,
}

#[allow(clippy::upper_case_acronyms)]
/// This struct can help you compute a CRC-32 (or CRC-x where **x** is equal or less than `32`) value.
#[derive(Clone)]
pub struct CRCu32 {
    lookup_table:    LookUpTable<u32>,
    sum:             u32,
    pub(crate) bits: u8,
    mask:            u32,
    initial:         u32,
    final_xor:       u32,
    reflect:         bool,
    reorder:         bool,
    accelerator:     Accelerator,
}

#[cfg(feature = "alloc")]
impl Debug for CRCu32 {
    #[inline]
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        debug_helper::impl_debug_for_struct!(CRCu32, f, self, let .lookup_table = self.lookup_table[0].as_ref(), (.sum, "0x{:08X}", self.sum), .bits, (.initial, "0x{:08X}", self.initial), (.final_xor, "0x{:08X}", self.final_xor), .reflect, .reorder);
    }
}

impl Display for CRCu32 {
    #[inline]
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        f.write_fmt(format_args!("0x{:01$X}", self.get_crc(), (self.bits as usize + 3) >> 2))
    }
}

impl CRCu32 {
    /// Create a `CRCu32` instance by providing a polynomial, the length of bits, an initial value, a final XOR value and a reflection setting.
    ///
    /// The parameters work the same way as in [`CRC::create_crc`](crate::CRC::create_crc), but `bits` must be between `1` and `32`.
    pub fn create_crc(poly: u32, bits: u8, initial: u32, final_xor: u32, reflect: bool) -> CRCu32 {
        debug_assert!(bits <= 32 && bits > 0);

        let lookup_table = if reflect {
            match (bits, poly) {
                (32, 0xEDB88320) => LookUpTable::Static(&REF_32_EDB88320),
                (32, 0x82F63B78) => LookUpTable::Static(&REF_32_82F63B78),
                _ => LookUpTable::dynamic(Self::crc_reflect_table(poly)),
            }
        } else {
            LookUpTable::dynamic(Self::crc_table(poly, bits))
        };

        let mut crc = Self::create_crc_with_exists_lookup_table(
            lookup_table,
            bits,
            initial,
            final_xor,
            reflect,
        );

        if bits == 32 && reflect {
            crc.accelerator = match poly {
                0xEDB88320 => Accelerator::Crc32,
                0x82F63B78 => Accelerator::Crc32c,
                _ => Accelerator::None,
            };
        }

        crc
    }

    #[inline]
    pub(crate) fn create_crc_with_exists_lookup_table(
        lookup_table: LookUpTable<u32>,
        bits: u8,
        initial: u32,
        final_xor: u32,
        reflect: bool,
    ) -> CRCu32 {
        let mask = u32::MAX >> (u32::BITS - u32::from(bits));

        let mut crc = CRCu32 {
            lookup_table,
            sum: 0,
            bits,
            mask,
            initial,
            final_xor,
            reflect,
            reorder: false,
            accelerator: Accelerator::None,
        };

        crc.reset();

        crc
    }

    #[inline]
    fn reflect_function(bits: u8, n: u32) -> u32 {
        n.reverse_bits() >> (u32::BITS - u32::from(bits))
    }

    /// Update the current CRC state with bytes.
    #[inline]
    pub fn update(&mut self, data: &[u8]) {
        let hardware_sum = match self.accelerator {
            Accelerator::None => None,
            Accelerator::Crc32 => hardware::crc32_update(self.sum, data),
            Accelerator::Crc32c => hardware::crc32c_update(self.sum, data),
        };

        if let Some(sum) = hardware_sum {
            self.sum = sum;

            return;
        }

        let tables = &*self.lookup_table;

        let mut sum = self.sum;

        #[cfg(feature = "slicing-by-8")]
        let data = {
            let mut chunks = data.chunks_exact(8);

            for chunk in &mut chunks {
                // The first input byte meets the lowest byte of a reflected register, or the highest byte of a non-reflected one.
                let register = if self.reflect { sum } else { sum.swap_bytes() };

                sum = slice_by_8(
                    tables,
                    u64::from_le_bytes(chunk.try_into().unwrap()) ^ u64::from(register),
                );
            }

            chunks.remainder()
        };

        let table = &tables[0];

        // Mix the rest of the register with the next input byte before the table lookup finishes, which shortens the dependency chain of each byte.
        if let Some((&first, rest)) = data.split_first() {
            if self.reflect {
                let mut index = sum as u8 ^ first;
                let mut sum_high = sum >> 8;

                for n in rest.iter().copied() {
                    let t = table[usize::from(index)];

                    index = t as u8 ^ (sum_high as u8 ^ n);
                    sum_high = (sum_high ^ t) >> 8;
                }

                sum = sum_high ^ table[usize::from(index)];
            } else {
                let mut index = (sum >> 24) as u8 ^ first;
                let mut sum_low = sum << 8;

                for n in rest.iter().copied() {
                    let t = table[usize::from(index)];

                    index = (t >> 24) as u8 ^ ((sum_low >> 24) as u8 ^ n);
                    sum_low = (sum_low ^ t) << 8;
                }

                sum = sum_low ^ table[usize::from(index)];
            }
        }

        self.sum = sum;
    }

    /// Digest some data.
    ///
    /// This is a compatibility wrapper around [`CRCu32::update`].
    #[inline]
    pub fn digest<T: ?Sized + AsRef<[u8]>>(&mut self, data: &T) {
        self.update(data.as_ref());
    }

    /// Reset the sum.
    pub fn reset(&mut self) {
        self.sum = if self.reflect {
            Self::reflect_function(self.bits, self.initial)
        } else {
            // A non-reflected register is left-aligned, so the same table layout works for any width.
            self.initial << (u32::BITS - u32::from(self.bits))
        };
    }

    /// Get the current CRC value (it always returns a `u32` value). You can continue calling `update` or `digest` even after getting a CRC value.
    pub fn get_crc(&self) -> u32 {
        let sum =
            if self.reflect { self.sum } else { self.sum >> (u32::BITS - u32::from(self.bits)) };

        let sum = (sum ^ self.final_xor) & self.mask;

        if self.reorder {
            let mut new_sum = 0;

            let e = u32::from(self.bits).div_ceil(8);

            let e_dec = e - 1;

            for i in 0..e {
                new_sum |= ((sum >> ((e_dec - i) * 8)) & 0xFF) << (i * 8);
            }

            new_sum
        } else {
            sum
        }
    }

    /// Build the lookup tables of a reflected CRC. `poly_rev` is the reversed polynomial.
    pub(crate) const fn crc_reflect_table(poly_rev: u32) -> Tables<u32> {
        let mut tables = [[0; 256]; SLICES];

        let mut i = 0;

        while i < 256 {
            let mut v = i as u32;

            let mut j = 0;

            while j < 8 {
                v = if v & 1 == 0 { v >> 1 } else { (v >> 1) ^ poly_rev };

                j += 1;
            }

            tables[0][i] = v;

            i += 1;
        }

        // Each extra table handles one more byte that follows the looked-up byte.
        let mut k = 1;

        while k < SLICES {
            let mut i = 0;

            while i < 256 {
                let v = tables[k - 1][i];

                tables[k][i] = (v >> 8) ^ tables[0][(v & 0xFF) as usize];

                i += 1;
            }

            k += 1;
        }

        tables
    }

    /// Build the lookup tables of a non-reflected CRC. The polynomial is left-aligned to 32 bits first.
    pub(crate) const fn crc_table(poly: u32, bits: u8) -> Tables<u32> {
        let poly = poly << (u32::BITS - bits as u32);

        let mut tables = [[0; 256]; SLICES];

        let mut i = 0;

        while i < 256 {
            let mut v = (i as u32) << 24;

            let mut j = 0;

            while j < 8 {
                v = if v & (1 << 31) == 0 { v << 1 } else { (v << 1) ^ poly };

                j += 1;
            }

            tables[0][i] = v;

            i += 1;
        }

        // Each extra table handles one more byte that follows the looked-up byte.
        let mut k = 1;

        while k < SLICES {
            let mut i = 0;

            while i < 256 {
                let v = tables[k - 1][i];

                tables[k][i] = (v << 8) ^ tables[0][(v >> 24) as usize];

                i += 1;
            }

            k += 1;
        }

        tables
    }
}

#[cfg(feature = "alloc")]
impl CRCu32 {
    /// Get the current CRC value (it always returns a vec instance with a length corresponding to the CRC bits). You can continue calling `update` or `digest` even after getting a CRC value.
    #[inline]
    pub fn get_crc_vec_le(&self) -> Vec<u8> {
        let crc = self.get_crc();

        let e = usize::from(self.bits).div_ceil(8);

        crc.to_le_bytes()[..e].to_vec()
    }

    /// Get the current CRC value (it always returns a vec instance with a length corresponding to the CRC bits). You can continue calling `update` or `digest` even after getting a CRC value.
    #[inline]
    pub fn get_crc_vec_be(&self) -> Vec<u8> {
        let crc = self.get_crc();

        let e = usize::from(self.bits).div_ceil(8);

        crc.to_be_bytes()[(4 - e)..].to_vec()
    }
}

#[cfg(feature = "heapless")]
impl CRCu32 {
    /// Get the current CRC value (it always returns a heapless vec instance with a length corresponding to the CRC bits). You can continue calling `update` or `digest` even after getting a CRC value.
    #[inline]
    pub fn get_crc_heapless_vec_le(&self) -> HeaplessVec<u8, 4, u8> {
        let crc = self.get_crc();

        let e = usize::from(self.bits).div_ceil(8);

        let mut vec = HeaplessVec::new();

        vec.extend_from_slice(&crc.to_le_bytes()[..e]).unwrap();

        vec
    }

    /// Get the current CRC value (it always returns a heapless vec instance with a length corresponding to the CRC bits). You can continue calling `update` or `digest` even after getting a CRC value.
    #[inline]
    pub fn get_crc_heapless_vec_be(&self) -> HeaplessVec<u8, 4, u8> {
        let crc = self.get_crc();

        let e = usize::from(self.bits).div_ceil(8);

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
    /// assert_eq!(0x04F03, crc.get_crc());
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0x04F03\", &crc.to_string());")]
    /// ```
    pub fn crc17can() -> CRCu32 {
        // Self::create_crc(0x0001685B, 17, 0x00000000, 0x00000000, false)

        let lookup_table = LookUpTable::Static(&NO_REF_17_0001685B);
        Self::create_crc_with_exists_lookup_table(lookup_table, 17, 0x00000000, 0x00000000, false)
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0x0ED841|0x102899|0x000000|false|0x000000|
    ///
    /// ```
    /// # use crc_any::CRCu32;
    /// let mut crc = CRCu32::crc21can();
    /// crc.digest(b"123456789");
    /// assert_eq!(0x0ED841, crc.get_crc());
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0x0ED841\", &crc.to_string());")]
    /// ```
    pub fn crc21can() -> CRCu32 {
        // Self::create_crc(0x00102899, 21, 0x00000000, 0x00000000, false)

        let lookup_table = LookUpTable::Static(&NO_REF_21_00102899);
        Self::create_crc_with_exists_lookup_table(lookup_table, 21, 0x00000000, 0x00000000, false)
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0x21CF02|0x864CFB|0xB704CE|false|0x000000|
    ///
    /// ```
    /// # use crc_any::CRCu32;
    /// let mut crc = CRCu32::crc24();
    /// crc.digest(b"123456789");
    /// assert_eq!(0x21CF02, crc.get_crc());
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
    /// assert_eq!(0xC25A56, crc.get_crc());
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
    /// assert_eq!(0x7979BD, crc.get_crc());
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
    /// assert_eq!(0x1F23B8, crc.get_crc());
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
    /// assert_eq!(0xCDE703, crc.get_crc());
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
    /// assert_eq!(0x23EF52, crc.get_crc());
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
    /// assert_eq!(0x200FA5, crc.get_crc());
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
    /// assert_eq!(0x04C34ABF, crc.get_crc());
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0x04C34ABF\", &crc.to_string());")]
    /// ```
    pub fn crc30cdma() -> CRCu32 {
        // Self::create_crc(0x2030B9C7, 30, 0x3FFFFFFF, 0x3FFFFFFF, false)

        let lookup_table = LookUpTable::Static(&NO_REF_30_2030B9C7);
        Self::create_crc_with_exists_lookup_table(lookup_table, 30, 0x3FFFFFFF, 0x3FFFFFFF, false)
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0xCBF43926|0x04C11DB7 (rev: 0xEDB88320)|0xFFFFFFFF|true|0xFFFFFFFF|
    ///
    /// ```
    /// # use crc_any::CRCu32;
    /// let mut crc = CRCu32::crc32();
    /// crc.digest(b"123456789");
    /// assert_eq!(0xCBF43926, crc.get_crc());
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0xCBF43926\", &crc.to_string());")]
    /// ```
    pub fn crc32() -> CRCu32 {
        // Self::create_crc(0xEDB88320, 32, 0xFFFFFFFF, 0xFFFFFFFF, true)

        let lookup_table = LookUpTable::Static(&REF_32_EDB88320);
        let mut crc = Self::create_crc_with_exists_lookup_table(
            lookup_table,
            32,
            0xFFFFFFFF,
            0xFFFFFFFF,
            true,
        );

        crc.accelerator = Accelerator::Crc32;

        crc
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
    /// assert_eq!(0x181989FC, crc.get_crc());
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
    /// assert_eq!(0xFC891918, crc.get_crc());
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
    /// assert_eq!(0xE3069283, crc.get_crc());
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0xE3069283\", &crc.to_string());")]
    /// ```
    pub fn crc32c() -> CRCu32 {
        // Self::create_crc(0x82F63B78, 32, 0xFFFFFFFF, 0xFFFFFFFF, true)

        let lookup_table = LookUpTable::Static(&REF_32_82F63B78);
        let mut crc = Self::create_crc_with_exists_lookup_table(
            lookup_table,
            32,
            0xFFFFFFFF,
            0xFFFFFFFF,
            true,
        );

        crc.accelerator = Accelerator::Crc32c;

        crc
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0x87315576|0xA833982B (rev: 0xD419CC15)|0xFFFFFFFF|true|0xFFFFFFFF|
    ///
    /// ```
    /// # use crc_any::CRCu32;
    /// let mut crc = CRCu32::crc32d();
    /// crc.digest(b"123456789");
    /// assert_eq!(0x87315576, crc.get_crc());
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
    /// assert_eq!(0x0376E6E7, crc.get_crc());
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
    /// assert_eq!(0x765E7680, crc.get_crc());
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
    /// assert_eq!(0x3010BF7F, crc.get_crc());
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
    /// assert_eq!(0x340BC6D9, crc.get_crc());
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0x340BC6D9\", &crc.to_string());")]
    /// ```
    pub fn crc32jamcrc() -> CRCu32 {
        // Self::create_crc(0xEDB88320, 32, 0xFFFFFFFF, 0x00000000, true)

        let lookup_table = LookUpTable::Static(&REF_32_EDB88320);
        let mut crc = Self::create_crc_with_exists_lookup_table(
            lookup_table,
            32,
            0xFFFFFFFF,
            0x00000000,
            true,
        );

        crc.accelerator = Accelerator::Crc32;

        crc
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0xBD0BE338|0x000000AF|0x00000000|false|0x00000000|
    ///
    /// ```
    /// # use crc_any::CRCu32;
    /// let mut crc = CRCu32::crc32xfer();
    /// crc.digest(b"123456789");
    /// assert_eq!(0xBD0BE338, crc.get_crc());
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
    use alloc::{fmt::Write, string::String};

    use super::CRCu32;

    #[test]
    fn print_lookup_table() {
        let crc = CRCu32::crc24ble();

        let mut s = String::new();

        for n in crc.lookup_table[0].iter().take(255) {
            s.write_fmt(format_args!("{}u32, ", n)).unwrap();
        }

        s.write_fmt(format_args!("{}u32", crc.lookup_table[0][255])).unwrap();

        println!("let lookup_table = [{}];", s);
    }
}

#[cfg(test)]
mod update_tests {
    use super::{Accelerator, CRCu32};
    use crate::hardware;

    #[test]
    fn hardware_matches_portable() {
        let input = b"123456789abcdefgh";

        for template in [
            CRCu32::crc32(),
            CRCu32::crc32c(),
            CRCu32::crc32jamcrc(),
            CRCu32::create_crc(0xEDB88320, 32, 0x12345678, 0x87654321, true),
            CRCu32::create_crc(0x82F63B78, 32, 0x12345678, 0x87654321, true),
        ] {
            for len in [7, 8, 9, 15, 16, 17] {
                let data = &input[..len];
                let hardware_sum = match template.accelerator {
                    Accelerator::Crc32 => hardware::crc32_update(template.sum, data),
                    Accelerator::Crc32c => hardware::crc32c_update(template.sum, data),
                    Accelerator::None => unreachable!(),
                };

                if let Some(hardware_sum) = hardware_sum {
                    let mut portable = template.clone();
                    portable.accelerator = Accelerator::None;
                    portable.update(data);

                    assert_eq!(portable.sum, hardware_sum);

                    let mut accelerated = template.clone();
                    accelerated.update(data);
                    assert_eq!(portable.get_crc(), accelerated.get_crc());
                }
            }
        }
    }
}
