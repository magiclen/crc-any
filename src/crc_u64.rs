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
    constants::crc_u64::*,
    lookup_table::{LookUpTable, SLICES, Tables},
};

#[allow(clippy::upper_case_acronyms)]
/// This struct can help you compute a CRC-64 (or CRC-x where **x** is equal or less than `64`) value.
#[derive(Clone)]
pub struct CRCu64 {
    lookup_table:    LookUpTable<u64>,
    sum:             u64,
    pub(crate) bits: u8,
    mask:            u64,
    initial:         u64,
    final_xor:       u64,
    reflect:         bool,
}

#[cfg(feature = "alloc")]
impl Debug for CRCu64 {
    #[inline]
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        debug_helper::impl_debug_for_struct!(CRCu64, f, self, let .lookup_table = self.lookup_table[0].as_ref(), (.sum, "0x{:016X}", self.sum), .bits, (.initial, "0x{:016X}", self.initial), (.final_xor, "0x{:016X}", self.final_xor), .reflect);
    }
}

impl Display for CRCu64 {
    #[inline]
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        f.write_fmt(format_args!("0x{:01$X}", self.get_crc(), (self.bits as usize + 3) >> 2))
    }
}

impl CRCu64 {
    /// Create a `CRCu64` instance by providing a polynomial, the length of bits, an initial value, a final XOR value and a reflection setting.
    ///
    /// The parameters work the same way as in [`CRC::create_crc`](crate::CRC::create_crc), but `bits` must be between `1` and `64`.
    pub fn create_crc(poly: u64, bits: u8, initial: u64, final_xor: u64, reflect: bool) -> CRCu64 {
        debug_assert!(bits <= 64 && bits > 0);

        let lookup_table = if reflect {
            LookUpTable::dynamic(Self::crc_reflect_table(poly))
        } else {
            LookUpTable::dynamic(Self::crc_table(poly, bits))
        };

        Self::create_crc_with_exists_lookup_table(lookup_table, bits, initial, final_xor, reflect)
    }

    #[inline]
    pub(crate) fn create_crc_with_exists_lookup_table(
        lookup_table: LookUpTable<u64>,
        bits: u8,
        initial: u64,
        final_xor: u64,
        reflect: bool,
    ) -> CRCu64 {
        let mask = u64::MAX >> (u64::BITS - u32::from(bits));

        let mut crc = CRCu64 {
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
    fn reflect_function(bits: u8, n: u64) -> u64 {
        n.reverse_bits() >> (u64::BITS - u32::from(bits))
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

                sum = slice_by_8(tables, u64::from_le_bytes(chunk.try_into().unwrap()) ^ register);
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
                let mut index = (sum >> 56) as u8 ^ first;
                let mut sum_low = sum << 8;

                for n in rest.iter().copied() {
                    let t = table[usize::from(index)];

                    index = (t >> 56) as u8 ^ ((sum_low >> 56) as u8 ^ n);
                    sum_low = (sum_low ^ t) << 8;
                }

                sum = sum_low ^ table[usize::from(index)];
            }
        }

        self.sum = sum;
    }

    /// Digest some data.
    ///
    /// This is a compatibility wrapper around [`CRCu64::update`].
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
            self.initial << (u64::BITS - u32::from(self.bits))
        };
    }

    /// Get the current CRC value (it always returns a `u64` value). You can continue calling `update` or `digest` even after getting a CRC value.
    pub fn get_crc(&self) -> u64 {
        let sum =
            if self.reflect { self.sum } else { self.sum >> (u64::BITS - u32::from(self.bits)) };

        (sum ^ self.final_xor) & self.mask
    }

    /// Build the lookup tables of a reflected CRC. `poly_rev` is the reversed polynomial.
    pub(crate) const fn crc_reflect_table(poly_rev: u64) -> Tables<u64> {
        let mut tables = [[0; 256]; SLICES];

        let mut i = 0;

        while i < 256 {
            let mut v = i as u64;

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

    /// Build the lookup tables of a non-reflected CRC. The polynomial is left-aligned to 64 bits first.
    pub(crate) const fn crc_table(poly: u64, bits: u8) -> Tables<u64> {
        let poly = poly << (u64::BITS - bits as u32);

        let mut tables = [[0; 256]; SLICES];

        let mut i = 0;

        while i < 256 {
            let mut v = (i as u64) << 56;

            let mut j = 0;

            while j < 8 {
                v = if v & (1 << 63) == 0 { v << 1 } else { (v << 1) ^ poly };

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

                tables[k][i] = (v << 8) ^ tables[0][(v >> 56) as usize];

                i += 1;
            }

            k += 1;
        }

        tables
    }
}

#[cfg(feature = "alloc")]
impl CRCu64 {
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

        crc.to_be_bytes()[(8 - e)..].to_vec()
    }
}

#[cfg(feature = "heapless")]
impl CRCu64 {
    /// Get the current CRC value (it always returns a heapless vec instance with a length corresponding to the CRC bits). You can continue calling `update` or `digest` even after getting a CRC value.
    #[inline]
    pub fn get_crc_heapless_vec_le(&self) -> HeaplessVec<u8, 8, u8> {
        let crc = self.get_crc();

        let e = usize::from(self.bits).div_ceil(8);

        let mut vec = HeaplessVec::new();

        vec.extend_from_slice(&crc.to_le_bytes()[..e]).unwrap();

        vec
    }

    /// Get the current CRC value (it always returns a heapless vec instance with a length corresponding to the CRC bits). You can continue calling `update` or `digest` even after getting a CRC value.
    #[inline]
    pub fn get_crc_heapless_vec_be(&self) -> HeaplessVec<u8, 8, u8> {
        let crc = self.get_crc();

        let e = usize::from(self.bits).div_ceil(8);

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
    /// assert_eq!(0xD4164FC646, crc.get_crc());
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
    /// assert_eq!(0x6C40DF5F0B497347, crc.get_crc());
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
    /// assert_eq!(0xB90956C775A41001, crc.get_crc());
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
    /// assert_eq!(0x62EC59E3F1A4F00A, crc.get_crc());
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
    /// This is CRC-64/REDIS, which uses the Jones polynomial with an initial value of zero.
    ///
    /// ```
    /// # use crc_any::CRCu64;
    /// let mut crc = CRCu64::crc64jones();
    /// crc.digest(b"123456789");
    /// assert_eq!(0xE9C6D914C4B8D9CA, crc.get_crc());
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
    use alloc::{fmt::Write, string::String};

    use super::CRCu64;

    #[test]
    fn print_lookup_table() {
        let crc = CRCu64::crc64jones();

        let mut s = String::new();

        for n in crc.lookup_table[0].iter().take(255) {
            s.write_fmt(format_args!("{}u64, ", n)).unwrap();
        }

        s.write_fmt(format_args!("{}u64", crc.lookup_table[0][255])).unwrap();

        println!("let lookup_table = [{}];", s);
    }
}
