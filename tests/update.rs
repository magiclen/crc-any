use crc_any::{CRC, CRCu32};

const INPUT: &[u8] = b"123456789abcdefgh";

#[test]
fn updates_match_across_chunk_boundaries() {
    for template in [
        CRC::crc3gsm(),
        CRC::crc4itu(),
        CRC::crc8(),
        CRC::crc16riello(),
        CRC::crc16ccitt_false(),
        CRC::crc24(),
        CRC::crc24ble(),
        CRC::crc32(),
        CRC::crc32c(),
        CRC::crc64(),
        CRC::crc64iso(),
    ] {
        for len in [7, 8, 9, 15, 16, 17] {
            let data = &INPUT[..len];
            let mut whole = template.clone();
            whole.update(data);
            let expected = whole.get_crc();

            for chunk_size in [1, 7, 8, 9] {
                let mut streamed = template.clone();
                let mut bytewise = template.clone();

                for chunk in data.chunks(chunk_size) {
                    for byte in chunk {
                        bytewise.update(core::slice::from_ref(byte));
                    }

                    streamed.update(chunk);
                    assert_eq!(bytewise.get_crc(), streamed.get_crc());
                }

                assert_eq!(expected, streamed.get_crc());
            }
        }
    }
}

#[test]
fn custom_crc32_parameters() {
    for (poly, expected) in [(0xEDB88320, 0x7112429F), (0x82F63B78, 0xECCC2944)] {
        let mut crc = CRCu32::create_crc(poly, 32, 0x12345678, 0x87654321, true);
        crc.update(INPUT);
        assert_eq!(expected, crc.get_crc());

        crc.reset();
        crc.update(&INPUT[..7]);
        crc.update(&INPUT[7..]);
        assert_eq!(expected, crc.get_crc());
    }
}
