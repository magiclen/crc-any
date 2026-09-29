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
    constants::crc_u16::*,
    lookup_table::{LookUpTable, SLICES, Tables},
};

#[allow(clippy::upper_case_acronyms)]
/// This struct can help you compute a CRC-16 (or CRC-x where **x** is equal or less than `16`) value.
#[derive(Clone)]
pub struct CRCu16 {
    lookup_table:    LookUpTable<u16>,
    sum:             u16,
    pub(crate) bits: u8,
    mask:            u16,
    initial:         u16,
    final_xor:       u16,
    reflect:         bool,
}

#[cfg(feature = "alloc")]
impl Debug for CRCu16 {
    #[inline]
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        debug_helper::impl_debug_for_struct!(CRCu16, f, self, let .lookup_table = self.lookup_table[0].as_ref(), (.sum, "0x{:04X}", self.sum), .bits, (.initial, "0x{:04X}", self.initial), (.final_xor, "0x{:04X}", self.final_xor), .reflect);
    }
}

impl Display for CRCu16 {
    #[inline]
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        f.write_fmt(format_args!("0x{:01$X}", self.get_crc(), (self.bits as usize + 3) >> 2))
    }
}

impl CRCu16 {
    /// Create a `CRCu16` instance by providing a polynomial, the length of bits, an initial value, a final XOR value and a reflection setting.
    ///
    /// The parameters work the same way as in [`CRC::create_crc`](crate::CRC::create_crc), but `bits` must be between `1` and `16`.
    pub fn create_crc(poly: u16, bits: u8, initial: u16, final_xor: u16, reflect: bool) -> CRCu16 {
        debug_assert!(bits <= 16 && bits > 0);

        let lookup_table = if reflect {
            LookUpTable::dynamic(Self::crc_reflect_table(poly))
        } else {
            LookUpTable::dynamic(Self::crc_table(poly, bits))
        };

        Self::create_crc_with_exists_lookup_table(lookup_table, bits, initial, final_xor, reflect)
    }

    #[inline]
    pub(crate) fn create_crc_with_exists_lookup_table(
        lookup_table: LookUpTable<u16>,
        bits: u8,
        initial: u16,
        final_xor: u16,
        reflect: bool,
    ) -> CRCu16 {
        let mask = u16::MAX >> (u16::BITS - u32::from(bits));

        let mut crc = CRCu16 {
            lookup_table,
            sum: 0,
            bits,
            mask,
            initial,
            final_xor,
            reflect,
        };

        crc.reset();

        crc
    }

    #[inline]
    fn reflect_function(bits: u8, n: u16) -> u16 {
        n.reverse_bits() >> (u16::BITS - u32::from(bits))
    }

    /// Update the current CRC state with bytes.
    #[inline]
    pub fn update(&mut self, data: &[u8]) {
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
                let mut index = (sum >> 8) as u8 ^ first;
                let mut sum_low = sum << 8;

                for n in rest.iter().copied() {
                    let t = table[usize::from(index)];

                    index = (t >> 8) as u8 ^ ((sum_low >> 8) as u8 ^ n);
                    sum_low = (sum_low ^ t) << 8;
                }

                sum = sum_low ^ table[usize::from(index)];
            }
        }

        self.sum = sum;
    }

    /// Digest some data.
    ///
    /// This is a compatibility wrapper around [`CRCu16::update`].
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
            self.initial << (u16::BITS - u32::from(self.bits))
        };
    }

    /// Get the current CRC value (it always returns a `u16` value). You can continue calling `update` or `digest` even after getting a CRC value.
    pub fn get_crc(&self) -> u16 {
        let sum =
            if self.reflect { self.sum } else { self.sum >> (u16::BITS - u32::from(self.bits)) };

        (sum ^ self.final_xor) & self.mask
    }

    /// Build the lookup tables of a reflected CRC. `poly_rev` is the reversed polynomial.
    pub(crate) const fn crc_reflect_table(poly_rev: u16) -> Tables<u16> {
        let mut tables = [[0; 256]; SLICES];

        let mut i = 0;

        while i < 256 {
            let mut v = i as u16;

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

    /// Build the lookup tables of a non-reflected CRC. The polynomial is left-aligned to 16 bits first.
    pub(crate) const fn crc_table(poly: u16, bits: u8) -> Tables<u16> {
        let poly = poly << (u16::BITS - bits as u32);

        let mut tables = [[0; 256]; SLICES];

        let mut i = 0;

        while i < 256 {
            let mut v = (i as u16) << 8;

            let mut j = 0;

            while j < 8 {
                v = if v & (1 << 15) == 0 { v << 1 } else { (v << 1) ^ poly };

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

                tables[k][i] = (v << 8) ^ tables[0][(v >> 8) as usize];

                i += 1;
            }

            k += 1;
        }

        tables
    }
}

#[cfg(feature = "alloc")]
impl CRCu16 {
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

        crc.to_be_bytes()[(2 - e)..].to_vec()
    }
}

#[cfg(feature = "heapless")]
impl CRCu16 {
    /// Get the current CRC value (it always returns a heapless vec instance with a length corresponding to the CRC bits). You can continue calling `update` or `digest` even after getting a CRC value.
    #[inline]
    pub fn get_crc_heapless_vec_le(&self) -> HeaplessVec<u8, 2, u8> {
        let crc = self.get_crc();

        let e = usize::from(self.bits).div_ceil(8);

        let mut vec = HeaplessVec::new();

        vec.extend_from_slice(&crc.to_le_bytes()[..e]).unwrap();

        vec
    }

    /// Get the current CRC value (it always returns a heapless vec instance with a length corresponding to the CRC bits). You can continue calling `update` or `digest` even after getting a CRC value.
    #[inline]
    pub fn get_crc_heapless_vec_be(&self) -> HeaplessVec<u8, 2, u8> {
        let crc = self.get_crc();

        let e = usize::from(self.bits).div_ceil(8);

        let mut vec = HeaplessVec::new();

        vec.extend_from_slice(&crc.to_be_bytes()[(2 - e)..]).unwrap();

        vec
    }
}

impl CRCu16 {
    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0x199|0x233|0x000|false|0x000|
    ///
    /// ```
    /// # use crc_any::CRCu16;
    /// let mut crc = CRCu16::crc10();
    /// crc.digest(b"123456789");
    /// assert_eq!(0x199, crc.get_crc());
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0x199\", &crc.to_string());")]
    /// ```
    pub fn crc10() -> CRCu16 {
        // Self::create_crc(0x0233, 10, 0x0000, 0x0000, false)

        let lookup_table = LookUpTable::Static(&NO_REF_10_0233);
        Self::create_crc_with_exists_lookup_table(lookup_table, 10, 0x0000, 0x0000, false)
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0x233|0x3D9|0x3FF|false|0x000|
    ///
    /// ```
    /// # use crc_any::CRCu16;
    /// let mut crc = CRCu16::crc10cdma2000();
    /// crc.digest(b"123456789");
    /// assert_eq!(0x233, crc.get_crc());
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0x233\", &crc.to_string());")]
    /// ```
    pub fn crc10cdma2000() -> CRCu16 {
        // Self::create_crc(0x03D9, 10, 0x03FF, 0x0000, false)

        let lookup_table = LookUpTable::Static(&NO_REF_10_03D9);
        Self::create_crc_with_exists_lookup_table(lookup_table, 10, 0x03FF, 0x0000, false)
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0x12A|0x175|0x000|false|0x3FF|
    ///
    /// ```
    /// # use crc_any::CRCu16;
    /// let mut crc = CRCu16::crc10gsm();
    /// crc.digest(b"123456789");
    /// assert_eq!(0x12A, crc.get_crc());
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0x12A\", &crc.to_string());")]
    /// ```
    pub fn crc10gsm() -> CRCu16 {
        // Self::create_crc(0x0175, 10, 0x0000, 0x03FF, false)

        let lookup_table = LookUpTable::Static(&NO_REF_10_0175);
        Self::create_crc_with_exists_lookup_table(lookup_table, 10, 0x0000, 0x03FF, false)
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0x5A3|0x385|0x01a|false|0x000|
    ///
    /// ```
    /// # use crc_any::CRCu16;
    /// let mut crc = CRCu16::crc11();
    /// crc.digest(b"123456789");
    /// assert_eq!(0x5A3, crc.get_crc());
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0x5A3\", &crc.to_string());")]
    /// ```
    pub fn crc11() -> CRCu16 {
        // Self::create_crc(0x0385, 11, 0x001A, 0x0000, false)

        let lookup_table = LookUpTable::Static(&NO_REF_11_0385);
        Self::create_crc_with_exists_lookup_table(lookup_table, 11, 0x001A, 0x0000, false)
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0xF5B|0x80F|0x000|false|0x000|
    ///
    /// ```
    /// # use crc_any::CRCu16;
    /// let mut crc = CRCu16::crc12();
    /// crc.digest(b"123456789");
    /// assert_eq!(0xF5B, crc.get_crc());
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0xF5B\", &crc.to_string());")]
    /// ```
    pub fn crc12() -> CRCu16 {
        // Self::create_crc(0x080F, 12, 0x0000, 0x0000, false)

        let lookup_table = LookUpTable::Static(&NO_REF_12_080F);
        Self::create_crc_with_exists_lookup_table(lookup_table, 12, 0x0000, 0x0000, false)
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0xD4D|0xF13|0xFFF|false|0x000|
    ///
    /// ```
    /// # use crc_any::CRCu16;
    /// let mut crc = CRCu16::crc12cdma2000();
    /// crc.digest(b"123456789");
    /// assert_eq!(0xD4D, crc.get_crc());
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0xD4D\", &crc.to_string());")]
    /// ```
    pub fn crc12cdma2000() -> CRCu16 {
        // Self::create_crc(0x0F13, 12, 0x0FFF, 0x0000, false)

        let lookup_table = LookUpTable::Static(&NO_REF_12_0F13);
        Self::create_crc_with_exists_lookup_table(lookup_table, 12, 0x0FFF, 0x0000, false)
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0xB34|0xD31|0x000|false|0xFFF|
    ///
    /// ```
    /// # use crc_any::CRCu16;
    /// let mut crc = CRCu16::crc12gsm();
    /// crc.digest(b"123456789");
    /// assert_eq!(0xB34, crc.get_crc());
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0xB34\", &crc.to_string());")]
    /// ```
    pub fn crc12gsm() -> CRCu16 {
        // Self::create_crc(0x0D31, 12, 0x0000, 0x0FFF, false)

        let lookup_table = LookUpTable::Static(&NO_REF_12_0D31);
        Self::create_crc_with_exists_lookup_table(lookup_table, 12, 0x0000, 0x0FFF, false)
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0x04FA|0x1CF5|0x0000|false|0x0000|
    ///
    /// ```
    /// # use crc_any::CRCu16;
    /// let mut crc = CRCu16::crc13bbc();
    /// crc.digest(b"123456789");
    /// assert_eq!(0x04FA, crc.get_crc());
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0x04FA\", &crc.to_string());")]
    /// ```
    pub fn crc13bbc() -> CRCu16 {
        // Self::create_crc(0x1CF5, 13, 0x0000, 0x0000, false)

        let lookup_table = LookUpTable::Static(&NO_REF_13_1CF5);
        Self::create_crc_with_exists_lookup_table(lookup_table, 13, 0x0000, 0x0000, false)
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0x082D|0x0805 (rev: 0x2804)|0x0000|true|0x0000|
    ///
    /// ```
    /// # use crc_any::CRCu16;
    /// let mut crc = CRCu16::crc14darc();
    /// crc.digest(b"123456789");
    /// assert_eq!(0x082D, crc.get_crc());
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0x082D\", &crc.to_string());")]
    /// ```
    pub fn crc14darc() -> CRCu16 {
        // Self::create_crc(0x2804, 14, 0x0000, 0x0000, true)

        let lookup_table = LookUpTable::Static(&REF_14_2804);
        Self::create_crc_with_exists_lookup_table(lookup_table, 14, 0x0000, 0x0000, true)
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0x30AE|0x202D|0x0000|false|0x3FFF|
    ///
    /// ```
    /// # use crc_any::CRCu16;
    /// let mut crc = CRCu16::crc14gsm();
    /// crc.digest(b"123456789");
    /// assert_eq!(0x30AE, crc.get_crc());
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0x30AE\", &crc.to_string());")]
    /// ```
    pub fn crc14gsm() -> CRCu16 {
        // Self::create_crc(0x202D, 14, 0x0000, 0x3FFF, false)

        let lookup_table = LookUpTable::Static(&NO_REF_14_202D);
        Self::create_crc_with_exists_lookup_table(lookup_table, 14, 0x0000, 0x3FFF, false)
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0x059E|0x4599|0x0000|false|0x0000|
    ///
    /// ```
    /// # use crc_any::CRCu16;
    /// let mut crc = CRCu16::crc15can();
    /// crc.digest(b"123456789");
    /// assert_eq!(0x059E, crc.get_crc());
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0x059E\", &crc.to_string());")]
    /// ```
    pub fn crc15can() -> CRCu16 {
        // Self::create_crc(0x4599, 15, 0x0000, 0x0000, false)

        let lookup_table = LookUpTable::Static(&NO_REF_15_4599);
        Self::create_crc_with_exists_lookup_table(lookup_table, 15, 0x0000, 0x0000, false)
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0x2566|0x6815|0x0000|false|0x0001|
    ///
    /// ```
    /// # use crc_any::CRCu16;
    /// let mut crc = CRCu16::crc15mpt1327();
    /// crc.digest(b"123456789");
    /// assert_eq!(0x2566, crc.get_crc());
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0x2566\", &crc.to_string());")]
    /// ```
    pub fn crc15mpt1327() -> CRCu16 {
        // Self::create_crc(0x6815, 15, 0x0000, 0x0001, false)

        let lookup_table = LookUpTable::Static(&NO_REF_15_6815);
        Self::create_crc_with_exists_lookup_table(lookup_table, 15, 0x0000, 0x0001, false)
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0xBB3D|0x8005 (rev: 0xA001)|0x0000|true|0x0000|
    ///
    /// ```
    /// # use crc_any::CRCu16;
    /// let mut crc = CRCu16::crc16();
    /// crc.digest(b"123456789");
    /// assert_eq!(0xBB3D, crc.get_crc());
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0xBB3D\", &crc.to_string());")]
    /// ```
    pub fn crc16() -> CRCu16 {
        //         Self::create_crc(0xA001, 16, 0x0000, 0x0000, true)

        let lookup_table = LookUpTable::Static(&REF_16_A001);
        Self::create_crc_with_exists_lookup_table(lookup_table, 16, 0x0000, 0x0000, true)
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0x29B1|0x1021|0xFFFF|false|0x0000|
    ///
    /// ```
    /// # use crc_any::CRCu16;
    /// let mut crc = CRCu16::crc16ccitt_false();
    /// crc.digest(b"123456789");
    /// assert_eq!(0x29B1, crc.get_crc());
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0x29B1\", &crc.to_string());")]
    /// ```
    pub fn crc16ccitt_false() -> CRCu16 {
        //         Self::create_crc(0x1021, 16, 0xFFFF, 0x0000, false)

        let lookup_table = LookUpTable::Static(&NO_REF_16_1021);
        Self::create_crc_with_exists_lookup_table(lookup_table, 16, 0xFFFF, 0x0000, false)
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0xE5CC|0x1021|0x1D0F|false|0x0000|
    ///
    /// ```
    /// # use crc_any::CRCu16;
    /// let mut crc = CRCu16::crc16aug_ccitt();
    /// crc.digest(b"123456789");
    /// assert_eq!(0xE5CC, crc.get_crc());
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0xE5CC\", &crc.to_string());")]
    /// ```
    pub fn crc16aug_ccitt() -> CRCu16 {
        //         Self::create_crc(0x1021, 16, 0x1D0F, 0x0000, false)

        let lookup_table = LookUpTable::Static(&NO_REF_16_1021);
        Self::create_crc_with_exists_lookup_table(lookup_table, 16, 0x1D0F, 0x0000, false)
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0xFEE8|0x8005|0x0000|false|0x0000|
    ///
    /// ```
    /// # use crc_any::CRCu16;
    /// let mut crc = CRCu16::crc16buypass();
    /// crc.digest(b"123456789");
    /// assert_eq!(0xFEE8, crc.get_crc());
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0xFEE8\", &crc.to_string());")]
    /// ```
    pub fn crc16buypass() -> CRCu16 {
        //         Self::create_crc(0x8005, 16, 0x0000, 0x0000, false)

        let lookup_table = LookUpTable::Static(&NO_REF_16_8005);
        Self::create_crc_with_exists_lookup_table(lookup_table, 16, 0x0000, 0x0000, false)
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0x4C06|0xC867|0xFFFF|false|0x0000|
    ///
    /// ```
    /// # use crc_any::CRCu16;
    /// let mut crc = CRCu16::crc16cdma2000();
    /// crc.digest(b"123456789");
    /// assert_eq!(0x4C06, crc.get_crc());
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0x4C06\", &crc.to_string());")]
    /// ```
    pub fn crc16cdma2000() -> CRCu16 {
        //         Self::create_crc(0xC867, 16, 0xFFFF, 0x0000, false)

        let lookup_table = LookUpTable::Static(&NO_REF_16_C867);
        Self::create_crc_with_exists_lookup_table(lookup_table, 16, 0xFFFF, 0x0000, false)
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0x9ECF|0x8005|0x800D|false|0x0000|
    ///
    /// ```
    /// # use crc_any::CRCu16;
    /// let mut crc = CRCu16::crc16dds_110();
    /// crc.digest(b"123456789");
    /// assert_eq!(0x9ECF, crc.get_crc());
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0x9ECF\", &crc.to_string());")]
    /// ```
    pub fn crc16dds_110() -> CRCu16 {
        //         Self::create_crc(0x8005, 16, 0x800D, 0x0000, false)

        let lookup_table = LookUpTable::Static(&NO_REF_16_8005);
        Self::create_crc_with_exists_lookup_table(lookup_table, 16, 0x800D, 0x0000, false)
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0x007E|0x0589|0x0000|false|0x0001|
    ///
    /// ```
    /// # use crc_any::CRCu16;
    /// let mut crc = CRCu16::crc16dect_r();
    /// crc.digest(b"123456789");
    /// assert_eq!(0x007E, crc.get_crc());
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0x007E\", &crc.to_string());")]
    /// ```
    pub fn crc16dect_r() -> CRCu16 {
        //         Self::create_crc(0x0589, 16, 0x0000, 0x0001, false)

        let lookup_table = LookUpTable::Static(&NO_REF_16_0589);
        Self::create_crc_with_exists_lookup_table(lookup_table, 16, 0x0000, 0x0001, false)
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0x007F|0x0589|0x0000|false|0x0000|
    ///
    /// ```
    /// # use crc_any::CRCu16;
    /// let mut crc = CRCu16::crc16dect_x();
    /// crc.digest(b"123456789");
    /// assert_eq!(0x007F, crc.get_crc());
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0x007F\", &crc.to_string());")]
    /// ```
    pub fn crc16dect_x() -> CRCu16 {
        //         Self::create_crc(0x0589, 16, 0x0000, 0x0000, false)

        let lookup_table = LookUpTable::Static(&NO_REF_16_0589);
        Self::create_crc_with_exists_lookup_table(lookup_table, 16, 0x0000, 0x0000, false)
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0xEA82|0x3D65 (rev: 0xA6BC)|0x0000|true|0xFFFF|
    ///
    /// ```
    /// # use crc_any::CRCu16;
    /// let mut crc = CRCu16::crc16dnp();
    /// crc.digest(b"123456789");
    /// assert_eq!(0xEA82, crc.get_crc());
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0xEA82\", &crc.to_string());")]
    /// ```
    pub fn crc16dnp() -> CRCu16 {
        //         Self::create_crc(0xA6BC, 16, 0x0000, 0xFFFF, true)

        let lookup_table = LookUpTable::Static(&REF_16_A6BC);
        Self::create_crc_with_exists_lookup_table(lookup_table, 16, 0x0000, 0xFFFF, true)
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0xC2B7|0x3D65|0x0000|false|0xFFFF|
    ///
    /// ```
    /// # use crc_any::CRCu16;
    /// let mut crc = CRCu16::crc16en_13757();
    /// crc.digest(b"123456789");
    /// assert_eq!(0xC2B7, crc.get_crc());
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0xC2B7\", &crc.to_string());")]
    /// ```
    pub fn crc16en_13757() -> CRCu16 {
        //         Self::create_crc(0x3D65, 16, 0x0000, 0xFFFF, false)

        let lookup_table = LookUpTable::Static(&NO_REF_16_3D65);
        Self::create_crc_with_exists_lookup_table(lookup_table, 16, 0x0000, 0xFFFF, false)
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0xD64E|0x1021|0xFFFF|false|0xFFFF|
    ///
    /// ```
    /// # use crc_any::CRCu16;
    /// let mut crc = CRCu16::crc16genibus();
    /// crc.digest(b"123456789");
    /// assert_eq!(0xD64E, crc.get_crc());
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0xD64E\", &crc.to_string());")]
    /// ```
    pub fn crc16genibus() -> CRCu16 {
        //         Self::create_crc(0x1021, 16, 0xFFFF, 0xFFFF, false)

        let lookup_table = LookUpTable::Static(&NO_REF_16_1021);
        Self::create_crc_with_exists_lookup_table(lookup_table, 16, 0xFFFF, 0xFFFF, false)
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0x44C2|0x8005 (rev: 0xA001)|0x0000|true|0xFFFF|
    ///
    /// ```
    /// # use crc_any::CRCu16;
    /// let mut crc = CRCu16::crc16maxim();
    /// crc.digest(b"123456789");
    /// assert_eq!(0x44C2, crc.get_crc());
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0x44C2\", &crc.to_string());")]
    /// ```
    pub fn crc16maxim() -> CRCu16 {
        //         Self::create_crc(0xA001, 16, 0x0000, 0xFFFF, true)

        let lookup_table = LookUpTable::Static(&REF_16_A001);
        Self::create_crc_with_exists_lookup_table(lookup_table, 16, 0x0000, 0xFFFF, true)
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0x6F91|0x1021 (rev: 0x8408)|0xFFFF|true|0x0000|
    ///
    /// ```
    /// # use crc_any::CRCu16;
    /// let mut crc = CRCu16::crc16mcrf4cc();
    /// crc.digest(b"123456789");
    /// assert_eq!(0x6F91, crc.get_crc());
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0x6F91\", &crc.to_string());")]
    /// ```
    pub fn crc16mcrf4cc() -> CRCu16 {
        //         Self::create_crc(0x8408, 16, 0xFFFF, 0x0000, true)

        let lookup_table = LookUpTable::Static(&REF_16_8408);
        Self::create_crc_with_exists_lookup_table(lookup_table, 16, 0xFFFF, 0x0000, true)
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0x63D0|0x1021 (rev: 0x8408)|0xB2AA|true|0x0000|
    ///
    /// ```
    /// # use crc_any::CRCu16;
    /// let mut crc = CRCu16::crc16riello();
    /// crc.digest(b"123456789");
    /// assert_eq!(0x63D0, crc.get_crc());
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0x63D0\", &crc.to_string());")]
    /// ```
    pub fn crc16riello() -> CRCu16 {
        //        Self::create_crc(0x8408, 16, 0xB2AA, 0x0000, true)

        let lookup_table = LookUpTable::Static(&REF_16_8408);
        Self::create_crc_with_exists_lookup_table(lookup_table, 16, 0xB2AA, 0x0000, true)
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0xD0DB|0x8BB7|0x0000|false|0x0000|
    ///
    /// ```
    /// # use crc_any::CRCu16;
    /// let mut crc = CRCu16::crc16t10_dif();
    /// crc.digest(b"123456789");
    /// assert_eq!(0xD0DB, crc.get_crc());
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0xD0DB\", &crc.to_string());")]
    /// ```
    pub fn crc16t10_dif() -> CRCu16 {
        //         Self::create_crc(0x8BB7, 16, 0x0000, 0x0000, false)

        let lookup_table = LookUpTable::Static(&NO_REF_16_8BB7);
        Self::create_crc_with_exists_lookup_table(lookup_table, 16, 0x0000, 0x0000, false)
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0x0FB3|0xA097|0x0000|false|0x0000|
    ///
    /// ```
    /// # use crc_any::CRCu16;
    /// let mut crc = CRCu16::crc16teledisk();
    /// crc.digest(b"123456789");
    /// assert_eq!(0x0FB3, crc.get_crc());
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0x0FB3\", &crc.to_string());")]
    /// ```
    pub fn crc16teledisk() -> CRCu16 {
        //         Self::create_crc(0xA097, 16, 0x0000, 0x0000, false)

        let lookup_table = LookUpTable::Static(&NO_REF_16_A097);
        Self::create_crc_with_exists_lookup_table(lookup_table, 16, 0x0000, 0x0000, false)
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0x26B1|0x1021 (rev: 0x8408)|0x89EC|true|0x0000|
    ///
    /// ```
    /// # use crc_any::CRCu16;
    /// let mut crc = CRCu16::crc16tms37157();
    /// crc.digest(b"123456789");
    /// assert_eq!(0x26B1, crc.get_crc());
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0x26B1\", &crc.to_string());")]
    /// ```
    pub fn crc16tms37157() -> CRCu16 {
        //         Self::create_crc(0x8408, 16, 0x89EC, 0x0000, true)

        let lookup_table = LookUpTable::Static(&REF_16_8408);
        Self::create_crc_with_exists_lookup_table(lookup_table, 16, 0x89EC, 0x0000, true)
    }

    /// This is the same as [`CRCu16::crc16tms37157`]. The old name has a typo.
    #[deprecated(note = "use `crc16tms37157` instead")]
    #[inline]
    pub fn crc16tms13157() -> CRCu16 {
        Self::crc16tms37157()
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0xB4C8|0x8005 (rev: 0xA001)|0xFFFF|true|0xFFFF|
    ///
    /// ```
    /// # use crc_any::CRCu16;
    /// let mut crc = CRCu16::crc16usb();
    /// crc.digest(b"123456789");
    /// assert_eq!(0xB4C8, crc.get_crc());
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0xB4C8\", &crc.to_string());")]
    /// ```
    pub fn crc16usb() -> CRCu16 {
        //         Self::create_crc(0xA001, 16, 0xFFFF, 0xFFFF, true)

        let lookup_table = LookUpTable::Static(&REF_16_A001);
        Self::create_crc_with_exists_lookup_table(lookup_table, 16, 0xFFFF, 0xFFFF, true)
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0xBF05|0x1021 (rev: 0x8408)|0xC6C6|true|0x0000|
    ///
    /// ```
    /// # use crc_any::CRCu16;
    /// let mut crc = CRCu16::crc_a();
    /// crc.digest(b"123456789");
    /// assert_eq!(0xBF05, crc.get_crc());
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0xBF05\", &crc.to_string());")]
    /// ```
    pub fn crc_a() -> CRCu16 {
        //         Self::create_crc(0x8408, 16, 0xC6C6, 0x0000, true)

        let lookup_table = LookUpTable::Static(&REF_16_8408);
        Self::create_crc_with_exists_lookup_table(lookup_table, 16, 0xC6C6, 0x0000, true)
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0x2189|0x1021 (rev: 0x8408)|0x0000|true|0x0000|
    ///
    /// ```
    /// # use crc_any::CRCu16;
    /// let mut crc = CRCu16::crc16kermit();
    /// crc.digest(b"123456789");
    /// assert_eq!(0x2189, crc.get_crc());
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0x2189\", &crc.to_string());")]
    /// ```
    pub fn crc16kermit() -> CRCu16 {
        //         Self::create_crc(0x8408, 16, 0x0000, 0x0000, true)

        let lookup_table = LookUpTable::Static(&REF_16_8408);
        Self::create_crc_with_exists_lookup_table(lookup_table, 16, 0x0000, 0x0000, true)
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0x4B37|0x8005 (rev: 0xA001)|0xFFFF|true|0x0000|
    ///
    /// ```
    /// # use crc_any::CRCu16;
    /// let mut crc = CRCu16::crc16modbus();
    /// crc.digest(b"123456789");
    /// assert_eq!(0x4B37, crc.get_crc());
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0x4B37\", &crc.to_string());")]
    /// ```
    pub fn crc16modbus() -> CRCu16 {
        //         Self::create_crc(0xA001, 16, 0xFFFF, 0x0000, true)

        let lookup_table = LookUpTable::Static(&REF_16_A001);
        Self::create_crc_with_exists_lookup_table(lookup_table, 16, 0xFFFF, 0x0000, true)
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0x906E|0x1021 (rev: 0x8408)|0xFFFF|true|0xFFFF|
    ///
    /// ```
    /// # use crc_any::CRCu16;
    /// let mut crc = CRCu16::crc16_x25();
    /// crc.digest(b"123456789");
    /// assert_eq!(0x906E, crc.get_crc());
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0x906E\", &crc.to_string());")]
    /// ```
    pub fn crc16_x25() -> CRCu16 {
        //         Self::create_crc(0x8408, 16, 0xFFFF, 0xFFFF, true)

        let lookup_table = LookUpTable::Static(&REF_16_8408);
        Self::create_crc_with_exists_lookup_table(lookup_table, 16, 0xFFFF, 0xFFFF, true)
    }

    /// |Check|Poly|Init|Ref|XorOut|
    /// |---|---|---|---|---|
    /// |0x31C3|0x1021|0x0000|false|0x0000|
    ///
    /// ```
    /// # use crc_any::CRCu16;
    /// let mut crc = CRCu16::crc16xmodem();
    /// crc.digest(b"123456789");
    /// assert_eq!(0x31C3, crc.get_crc());
    #[cfg_attr(feature = "alloc", doc = "assert_eq!(\"0x31C3\", &crc.to_string());")]
    /// ```
    pub fn crc16xmodem() -> CRCu16 {
        //         Self::create_crc(0x1021, 16, 0x0000, 0x0000, false)

        let lookup_table = LookUpTable::Static(&NO_REF_16_1021);
        Self::create_crc_with_exists_lookup_table(lookup_table, 16, 0x0000, 0x0000, false)
    }
}

#[cfg(all(feature = "development", test))]
mod tests {
    use alloc::{fmt::Write, string::String};

    use super::CRCu16;

    #[test]
    fn print_lookup_table() {
        let crc = CRCu16::crc16kermit();

        let mut s = String::new();

        for n in crc.lookup_table[0].iter().take(255) {
            s.write_fmt(format_args!("{}u16, ", n)).unwrap();
        }

        s.write_fmt(format_args!("{}u16", crc.lookup_table[0][255])).unwrap();

        println!("let lookup_table = [{}];", s);
    }
}
