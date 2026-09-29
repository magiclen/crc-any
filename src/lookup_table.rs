/// The number of lookup tables for each CRC. With the `slicing-by-8` feature, each extra table lets one more byte be processed in the same step.
#[cfg(not(feature = "slicing-by-8"))]
pub(crate) const SLICES: usize = 1;
/// The number of lookup tables for each CRC. With the `slicing-by-8` feature, each extra table lets one more byte be processed in the same step.
#[cfg(feature = "slicing-by-8")]
pub(crate) const SLICES: usize = 8;

/// The lookup tables of a CRC. The first table is the classic byte-by-byte table.
pub(crate) type Tables<T> = [[T; 256]; SLICES];

/// This enum hold lookup table for static know or dynamic created table
#[derive(Clone)]
pub(crate) enum LookUpTable<T: 'static> {
    Static(&'static Tables<T>),
    #[cfg(not(all(feature = "slicing-by-8", feature = "alloc")))]
    Dynamic(Tables<T>),
    /// Slicing-by-8 tables are large, so they are put on the heap to keep instances small when heap allocation is allowed.
    #[cfg(all(feature = "slicing-by-8", feature = "alloc"))]
    Dynamic(alloc::boxed::Box<Tables<T>>),
}

impl<T> LookUpTable<T> {
    #[inline]
    pub(crate) fn dynamic(tables: Tables<T>) -> Self {
        #[cfg(not(all(feature = "slicing-by-8", feature = "alloc")))]
        {
            LookUpTable::Dynamic(tables)
        }

        #[cfg(all(feature = "slicing-by-8", feature = "alloc"))]
        {
            LookUpTable::Dynamic(alloc::boxed::Box::new(tables))
        }
    }
}

impl<T> core::ops::Deref for LookUpTable<T> {
    type Target = Tables<T>;

    fn deref(&self) -> &Tables<T> {
        match *self {
            LookUpTable::Static(s) => s,
            LookUpTable::Dynamic(ref d) => d,
        }
    }
}

/// Compute the register after 8 bytes. `x` is the 8 input bytes (in little-endian order) already XORed with the register bytes that meet them.
#[cfg(feature = "slicing-by-8")]
#[inline]
pub(crate) fn slice_by_8<T: Copy + core::ops::BitXor<Output = T>>(tables: &Tables<T>, x: u64) -> T {
    let a = tables[7][(x & 0xFF) as usize] ^ tables[6][((x >> 8) & 0xFF) as usize];
    let b = tables[5][((x >> 16) & 0xFF) as usize] ^ tables[4][((x >> 24) & 0xFF) as usize];
    let c = tables[3][((x >> 32) & 0xFF) as usize] ^ tables[2][((x >> 40) & 0xFF) as usize];
    let d = tables[1][((x >> 48) & 0xFF) as usize] ^ tables[0][(x >> 56) as usize];

    (a ^ b) ^ (c ^ d)
}
