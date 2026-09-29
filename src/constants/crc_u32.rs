use crate::crc_u32::CRCu32;

pub(crate) static NO_REF_17_0001685B: [u32; 256] = CRCu32::crc_table(0x0001685B, 17);
pub(crate) static NO_REF_21_00102899: [u32; 256] = CRCu32::crc_table(0x00102899, 21);
pub(crate) static NO_REF_24_005D6DCB: [u32; 256] = CRCu32::crc_table(0x005D6DCB, 24);
pub(crate) static NO_REF_24_00800063: [u32; 256] = CRCu32::crc_table(0x00800063, 24);
pub(crate) static NO_REF_24_00864CFB: [u32; 256] = CRCu32::crc_table(0x00864CFB, 24);
pub(crate) static REF_24_00DA6000: [u32; 256] = CRCu32::crc_reflect_table(0x00DA6000);
pub(crate) static NO_REF_30_2030B9C7: [u32; 256] = CRCu32::crc_table(0x2030B9C7, 30);
pub(crate) static NO_REF_32_000000AF: [u32; 256] = CRCu32::crc_table(0x000000AF, 32);
pub(crate) static NO_REF_32_04C11DB7: [u32; 256] = CRCu32::crc_table(0x04C11DB7, 32);
pub(crate) static NO_REF_32_814141AB: [u32; 256] = CRCu32::crc_table(0x814141AB, 32);
pub(crate) static REF_32_82F63B78: [u32; 256] = CRCu32::crc_reflect_table(0x82F63B78);
pub(crate) static REF_32_D419CC15: [u32; 256] = CRCu32::crc_reflect_table(0xD419CC15);
pub(crate) static REF_32_EDB88320: [u32; 256] = CRCu32::crc_reflect_table(0xEDB88320);
