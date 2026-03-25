use crate::chunk::Chunk;
use crate::optimizer::OptimizedFrame;
use std::fs::File;
use std::io::{self, Write};

pub fn save_apng(
    output_path: &str,
    width: u32,
    height: u32,
    loops: u32,
    frames: &[OptimizedFrame],
    palette: Option<&[[u8; 3]]>,
    transparency: Option<&[u8]>,
) -> io::Result<()> {
    let mut file = File::create(output_path)?;

    // Write PNG Signature
    file.write_all(b"\x89PNG\r\n\x1a\n")?;

    // Write IHDR
    // Width (4), Height (4), Bit depth (1), Color type (1), Compression (1), Filter (1), Interlace (1)
    let mut ihdr_data = Vec::with_capacity(13);
    ihdr_data.extend_from_slice(&width.to_be_bytes());
    ihdr_data.extend_from_slice(&height.to_be_bytes());
    ihdr_data.push(8); // 8-bit
    let color_type = if palette.is_some() { 3 } else { 6 };
    ihdr_data.push(color_type); // RGBA=6, Palette=3
    ihdr_data.push(0); // Compression method
    ihdr_data.push(0); // Filter method
    ihdr_data.push(0); // Interlace method
    Chunk::new(*b"IHDR", ihdr_data).write(&mut file)?;

    // Write acTL
    let mut actl_data = Vec::with_capacity(8);
    actl_data.extend_from_slice(&(frames.len() as u32).to_be_bytes());
    actl_data.extend_from_slice(&loops.to_be_bytes());
    Chunk::new(*b"acTL", actl_data).write(&mut file)?;

    // Write PLTE and tRNS if present
    if let Some(pal) = palette {
        let mut plte_data = Vec::with_capacity(pal.len() * 3);
        for color in pal {
            plte_data.extend_from_slice(color);
        }
        Chunk::new(*b"PLTE", plte_data).write(&mut file)?;
    }

    if let Some(trns) = transparency {
        Chunk::new(*b"tRNS", trns.to_vec()).write(&mut file)?;
    }

    let mut sequence_number = 0u32;

    for (i, frame) in frames.iter().enumerate() {
        // Write fcTL
        let mut fctl_data = Vec::with_capacity(26);
        fctl_data.extend_from_slice(&sequence_number.to_be_bytes());
        fctl_data.extend_from_slice(&frame.w.to_be_bytes());
        fctl_data.extend_from_slice(&frame.h.to_be_bytes());
        fctl_data.extend_from_slice(&frame.x.to_be_bytes());
        fctl_data.extend_from_slice(&frame.y.to_be_bytes());
        fctl_data.extend_from_slice(&frame.delay_num.to_be_bytes());
        fctl_data.extend_from_slice(&frame.delay_den.to_be_bytes());
        fctl_data.push(frame.dispose_op);
        fctl_data.push(frame.blend_op);
        Chunk::new(*b"fcTL", fctl_data).write(&mut file)?;
        sequence_number += 1;

        // Write image data
        if i == 0 {
            // First frame must be IDAT
            Chunk::new(*b"IDAT", frame.compressed_data.clone()).write(&mut file)?;
        } else {
            // Subsequent frames must be fdAT (prepended with sequence_number)
            let mut fdat_data = Vec::with_capacity(4 + frame.compressed_data.len());
            fdat_data.extend_from_slice(&sequence_number.to_be_bytes());
            fdat_data.extend_from_slice(&frame.compressed_data);
            Chunk::new(*b"fdAT", fdat_data).write(&mut file)?;
            sequence_number += 1;
        }
    }

    // Write IEND
    Chunk::new(*b"IEND", Vec::new()).write(&mut file)?;

    Ok(())
}
