#[cfg(not(target_arch = "wasm32"))]
use std::fs::File;
#[cfg(not(target_arch = "wasm32"))]
use std::io::BufReader;

use std::io::{Cursor, Read};

pub struct Frame {
    pub width: u32,
    pub height: u32,
    pub x_offset: u32,
    pub y_offset: u32,
    pub delay_num: u16,
    pub delay_den: u16,
    pub dispose_op: u8,
    pub blend_op: u8,
    pub data: Vec<u8>,
}

pub struct ApngImage {
    pub width: u32,
    pub height: u32,
    pub loops: u32,
    pub frames: Vec<Frame>,
}

#[cfg(not(target_arch = "wasm32"))]
pub fn load_apng(path: &str) -> Result<ApngImage, String> {
    let file = File::open(path).map_err(|e| e.to_string())?;
    let reader = BufReader::new(file);
    load_apng_from_reader(reader)
}

pub fn load_apng_from_memory(data: &[u8]) -> Result<ApngImage, String> {
    let reader = Cursor::new(data);
    load_apng_from_reader(reader)
}

pub fn load_apng_from_reader<R: Read + std::io::BufRead + std::io::Seek>(reader: R) -> Result<ApngImage, String> {
    let mut decoder = png::Decoder::new(reader);

    // We want to normalize the output to 8-bit per channel
    decoder.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);

    let mut reader = decoder.read_info().map_err(|e| e.to_string())?;
    let mut buf = vec![0; reader.output_buffer_size().unwrap_or(0)];
    let info = reader.info();

    let width = info.width;
    let height = info.height;
    let loops = info.animation_control().map(|a| a.num_plays).unwrap_or(0);
    let frame_count = info.animation_control().map(|a| a.num_frames).unwrap_or(1);

    let color_type = info.color_type;
    let mut frames = Vec::new();

    for _ in 0..frame_count {
        if let Ok(frame_info) = reader.next_frame(&mut buf) {
            let mut frame_data = buf[..frame_info.buffer_size()].to_vec();

            // Convert to RGBA if not already
            match color_type {
                png::ColorType::Rgba => {
                    // Already RGBA
                }
                png::ColorType::Rgb => {
                    let mut rgba = Vec::with_capacity((frame_data.len() / 3) * 4);
                    for chunk in frame_data.chunks(3) {
                        rgba.extend_from_slice(chunk);
                        rgba.push(255); // Alpha
                    }
                    frame_data = rgba;
                }
                png::ColorType::GrayscaleAlpha => {
                    let mut rgba = Vec::with_capacity((frame_data.len() / 2) * 4);
                    for chunk in frame_data.chunks(2) {
                        let g = chunk[0];
                        let a = chunk[1];
                        rgba.extend_from_slice(&[g, g, g, a]);
                    }
                    frame_data = rgba;
                }
                png::ColorType::Grayscale => {
                    let mut rgba = Vec::with_capacity(frame_data.len() * 4);
                    for &g in &frame_data {
                        rgba.extend_from_slice(&[g, g, g, 255]);
                    }
                    frame_data = rgba;
                }
                _ => {
                    // Indexed or others should have been EXPANDed by the reader
                }
            }

            let fc = reader
                .info()
                .frame_control
                .clone()
                .unwrap_or(png::FrameControl {
                    sequence_number: 0,
                    width: frame_info.width,
                    height: frame_info.height,
                    x_offset: 0,
                    y_offset: 0,
                    delay_num: 0,
                    delay_den: 0,
                    dispose_op: png::DisposeOp::None,
                    blend_op: png::BlendOp::Source,
                });

            frames.push(Frame {
                width: fc.width,
                height: fc.height,
                x_offset: fc.x_offset,
                y_offset: fc.y_offset,
                delay_num: fc.delay_num,
                delay_den: fc.delay_den,
                dispose_op: fc.dispose_op as u8,
                blend_op: fc.blend_op as u8,
                data: frame_data,
            });
        } else {
            break;
        }
    }

    Ok(ApngImage {
        width,
        height,
        loops,
        frames,
    })
}
