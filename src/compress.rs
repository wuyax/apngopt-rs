use flate2::write::ZlibEncoder;
use flate2::Compression;
use std::io::Write;
use std::num::NonZeroU64;
use zopfli::{Format, Options};

pub fn compress_zlib(data: &[u8]) -> Vec<u8> {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::best());
    encoder.write_all(data).unwrap();
    encoder.finish().unwrap()
}

pub fn compress_zopfli(data: &[u8], iterations: u32) -> Vec<u8> {
    let options = Options {
        iteration_count: NonZeroU64::new(iterations as u64).unwrap_or(NonZeroU64::new(15).unwrap()),
        ..Default::default()
    };
    let mut out = Vec::new();
    zopfli::compress(options, Format::Zlib, data, &mut out).unwrap();
    out
}

pub fn compress_data(data: &[u8], method: u8, iterations: u32) -> Vec<u8> {
    match method {
        0 => compress_zlib(data),               // zlib
        1 => compress_zlib(data), // 7zip deflate equivalent (we use max zlib for now in rust)
        2 => compress_zopfli(data, iterations), // zopfli
        _ => compress_zlib(data),
    }
}
