use crc_any::CRC;

use bencher::{benchmark_group, benchmark_main, Bencher};

fn crc8_construct(bencher: &mut Bencher) {
    bencher.iter(|| CRC::create_crc(0x07, 8, 0x00, 0x00, false))
}

fn crc8_update_megabytes(bencher: &mut Bencher) {
    let mut crc = CRC::crc8();
    let mut bytes = Vec::with_capacity(1000000);

    #[allow(clippy::uninit_vec)]
    unsafe {
        bytes.set_len(1000000);
    }

    bencher.iter(|| {
        crc.digest(&bytes);

        crc.get_crc()
    })
}

fn crc12_construct(bencher: &mut Bencher) {
    bencher.iter(|| CRC::create_crc(0x080F, 12, 0x0000, 0x0000, false))
}

fn crc12_update_megabytes(bencher: &mut Bencher) {
    let mut crc = CRC::crc12();
    let mut bytes = Vec::with_capacity(1000000);

    #[allow(clippy::uninit_vec)]
    unsafe {
        bytes.set_len(1000000);
    }

    bencher.iter(|| {
        crc.digest(&bytes);

        crc.get_crc()
    })
}

fn crc16_construct(bencher: &mut Bencher) {
    bencher.iter(|| CRC::create_crc(0xA001, 16, 0x0000, 0x0000, true))
}

fn crc16_update_megabytes(bencher: &mut Bencher) {
    let mut crc = CRC::crc16();
    let mut bytes = Vec::with_capacity(1000000);

    #[allow(clippy::uninit_vec)]
    unsafe {
        bytes.set_len(1000000);
    }

    bencher.iter(|| {
        crc.digest(&bytes);

        crc.get_crc()
    })
}

fn crc16_construct_wellknown(bencher: &mut Bencher) {
    bencher.iter(crc_any::CRCu16::crc16ccitt_false)
}

fn crc16_update_megabytes_wellknown(bencher: &mut Bencher) {
    let mut crc = crc_any::CRCu16::crc16ccitt_false();
    let mut bytes = Vec::with_capacity(1000000);

    #[allow(clippy::uninit_vec)]
    unsafe {
        bytes.set_len(1000000);
    }

    bencher.iter(|| {
        crc.digest(&bytes);

        crc.get_crc()
    })
}

fn crc32_construct(bencher: &mut Bencher) {
    bencher.iter(|| CRC::create_crc(0xEDB88320, 32, 0xFFFFFFFF, 0xFFFFFFFF, true))
}

fn crc32_update_megabytes(bencher: &mut Bencher) {
    let mut crc = CRC::crc32();
    let mut bytes = Vec::with_capacity(1000000);

    #[allow(clippy::uninit_vec)]
    unsafe {
        bytes.set_len(1000000);
    }

    bencher.iter(|| {
        crc.digest(&bytes);

        crc.get_crc()
    })
}

fn crc64_construct(bencher: &mut Bencher) {
    bencher.iter(|| {
        CRC::create_crc(0x42F0E1EBA9EA3693, 64, 0xFFFFFFFFFFFFFFFF, 0xFFFFFFFFFFFFFFFF, false)
    })
}

fn crc64_update_megabytes(bencher: &mut Bencher) {
    let mut crc = CRC::crc64();
    let mut bytes = Vec::with_capacity(1000000);

    #[allow(clippy::uninit_vec)]
    unsafe {
        bytes.set_len(1000000);
    }

    bencher.iter(|| {
        crc.digest(&bytes);

        crc.get_crc()
    })
}

fn crc64_construct_wellknown(bencher: &mut Bencher) {
    bencher.iter(crc_any::CRCu64::crc64iso)
}

fn crc64_update_megabytes_wellknown(bencher: &mut Bencher) {
    let mut crc = CRC::crc64();
    let mut bytes = Vec::with_capacity(1000000);

    #[allow(clippy::uninit_vec)]
    unsafe {
        bytes.set_len(1000000);
    }

    bencher.iter(|| {
        crc.digest(&bytes);

        crc.get_crc()
    })
}

benchmark_group!(crc8, crc8_construct, crc8_update_megabytes);
benchmark_group!(crc12, crc12_construct, crc12_update_megabytes);
benchmark_group!(crc16, crc16_construct, crc16_update_megabytes);
benchmark_group!(crc16_wellknown, crc16_construct_wellknown, crc16_update_megabytes_wellknown);
benchmark_group!(crc32, crc32_construct, crc32_update_megabytes);
benchmark_group!(crc64, crc64_construct, crc64_update_megabytes);
benchmark_group!(crc64_wellknown, crc64_construct_wellknown, crc64_update_megabytes_wellknown);

benchmark_main!(crc8, crc12, crc16, crc16_wellknown, crc32, crc64, crc64_wellknown);
