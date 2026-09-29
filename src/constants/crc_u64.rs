use crate::crc_u64::CRCu64;

pub(crate) static NO_REF_40_0000000004820009: [u64; 256] =
    CRCu64::crc_table(0x0000000004820009, 40);
pub(crate) static NO_REF_64_42F0E1EBA9EA3693: [u64; 256] =
    CRCu64::crc_table(0x42F0E1EBA9EA3693, 64);
pub(crate) static REF_64_95AC9329AC4BC9B5: [u64; 256] =
    CRCu64::crc_reflect_table(0x95AC9329AC4BC9B5);
pub(crate) static REF_64_D800000000000000: [u64; 256] =
    CRCu64::crc_reflect_table(0xD800000000000000);
