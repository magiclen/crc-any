/// Update a CRC-32C register with CPU instructions. It returns `None` if the instructions cannot be used.
#[inline]
pub(crate) fn crc32c_update(sum: u32, data: &[u8]) -> Option<u32> {
    #[cfg(all(
        any(target_arch = "x86", target_arch = "x86_64"),
        any(feature = "std", target_feature = "sse4.2")
    ))]
    if sse42_available() {
        // SAFETY: SSE4.2 is available.
        return Some(unsafe { crc32c_sse42_update(sum, data) });
    }

    #[cfg(all(target_arch = "aarch64", any(feature = "std", target_feature = "crc")))]
    if arm_crc_available() {
        // SAFETY: The CRC extension is available.
        return Some(unsafe { crc32c_arm_update(sum, data) });
    }

    let _ = (sum, data);

    None
}

/// Update a CRC-32 (the reflected `0x04C11DB7` polynomial) register with CPU instructions. It returns `None` if the instructions cannot be used.
#[inline]
pub(crate) fn crc32_update(sum: u32, data: &[u8]) -> Option<u32> {
    #[cfg(all(target_arch = "aarch64", any(feature = "std", target_feature = "crc")))]
    if arm_crc_available() {
        // SAFETY: The CRC extension is available.
        return Some(unsafe { crc32_arm_update(sum, data) });
    }

    let _ = (sum, data);

    None
}

#[cfg(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    any(feature = "std", target_feature = "sse4.2")
))]
#[inline]
fn sse42_available() -> bool {
    #[cfg(target_feature = "sse4.2")]
    {
        true
    }

    #[cfg(not(target_feature = "sse4.2"))]
    {
        std::is_x86_feature_detected!("sse4.2")
    }
}

#[cfg(all(target_arch = "aarch64", any(feature = "std", target_feature = "crc")))]
#[inline]
fn arm_crc_available() -> bool {
    #[cfg(target_feature = "crc")]
    {
        true
    }

    #[cfg(not(target_feature = "crc"))]
    {
        std::arch::is_aarch64_feature_detected!("crc")
    }
}

#[cfg(all(target_arch = "x86_64", any(feature = "std", target_feature = "sse4.2")))]
#[target_feature(enable = "sse4.2")]
unsafe fn crc32c_sse42_update(mut sum: u32, data: &[u8]) -> u32 {
    use core::arch::x86_64::{_mm_crc32_u8, _mm_crc32_u64};

    let mut chunks = data.chunks_exact(8);

    for chunk in &mut chunks {
        let block = u64::from_le_bytes(chunk.try_into().unwrap());

        sum = _mm_crc32_u64(sum as u64, block) as u32;
    }

    for n in chunks.remainder().iter().copied() {
        sum = _mm_crc32_u8(sum, n);
    }

    sum
}

#[cfg(all(target_arch = "x86", any(feature = "std", target_feature = "sse4.2")))]
#[target_feature(enable = "sse4.2")]
unsafe fn crc32c_sse42_update(mut sum: u32, data: &[u8]) -> u32 {
    use core::arch::x86::{_mm_crc32_u8, _mm_crc32_u32};

    let mut chunks = data.chunks_exact(4);

    for chunk in &mut chunks {
        let block = u32::from_le_bytes(chunk.try_into().unwrap());

        sum = _mm_crc32_u32(sum, block);
    }

    for n in chunks.remainder().iter().copied() {
        sum = _mm_crc32_u8(sum, n);
    }

    sum
}

#[cfg(all(target_arch = "aarch64", any(feature = "std", target_feature = "crc")))]
#[target_feature(enable = "crc")]
unsafe fn crc32c_arm_update(mut sum: u32, data: &[u8]) -> u32 {
    use core::arch::aarch64::{__crc32cb, __crc32cd};

    let mut chunks = data.chunks_exact(8);

    for chunk in &mut chunks {
        sum = __crc32cd(sum, u64::from_le_bytes(chunk.try_into().unwrap()));
    }

    for n in chunks.remainder().iter().copied() {
        sum = __crc32cb(sum, n);
    }

    sum
}

#[cfg(all(target_arch = "aarch64", any(feature = "std", target_feature = "crc")))]
#[target_feature(enable = "crc")]
unsafe fn crc32_arm_update(mut sum: u32, data: &[u8]) -> u32 {
    use core::arch::aarch64::{__crc32b, __crc32d};

    let mut chunks = data.chunks_exact(8);

    for chunk in &mut chunks {
        sum = __crc32d(sum, u64::from_le_bytes(chunk.try_into().unwrap()));
    }

    for n in chunks.remainder().iter().copied() {
        sum = __crc32b(sum, n);
    }

    sum
}
