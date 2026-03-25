use imagequant::{Attributes, Histogram, RGBA};

pub struct QuantizedData {
    pub frames: Vec<Vec<u8>>,
    pub palette: Vec<[u8; 3]>,
    pub transparency: Vec<u8>,
}

pub fn quantize_frames(
    frames: &[Vec<u8>],
    width: u32,
    height: u32,
) -> Result<QuantizedData, String> {
    let mut attr = Attributes::new();
    // Default apngopt quality
    attr.set_quality(0, 100)
        .map_err(|_| "Failed to set quality")?;

    let mut hist = Histogram::new(&attr);

    let mut images = Vec::new();

    for frame_data in frames {
        let mut rgba_frame = Vec::with_capacity(frame_data.len() / 4);
        for chunk in frame_data.chunks(4) {
            rgba_frame.push(RGBA {
                r: chunk[0],
                g: chunk[1],
                b: chunk[2],
                a: chunk[3],
            });
        }

        // new_image_stride is preferred over new_image_stride_copy in 4.x
        let mut image = attr
            .new_image_stride(
                rgba_frame.clone(),
                width as usize,
                height as usize,
                width as usize,
                0.0,
            )
            .map_err(|_| "Failed to create imagequant image")?;

        hist.add_image(&attr, &mut image)
            .map_err(|_| "Failed to add image to histogram")?;
        images.push(image);
    }

    let mut res = hist
        .quantize(&attr)
        .map_err(|_| "Failed to quantize histogram")?;

    // We can use res.remapped on the first image to get the global palette
    let mut quantized_frames = Vec::new();
    let mut final_palette = Vec::new();
    let mut final_transparency = Vec::new();

    for (i, mut image) in images.into_iter().enumerate() {
        let (pal, remapped_pixels) = res
            .remapped(&mut image)
            .map_err(|_| "Failed to remap image")?;

        if i == 0 {
            for entry in pal {
                final_palette.push([entry.r, entry.g, entry.b]);
                final_transparency.push(entry.a);
            }
        }
        quantized_frames.push(remapped_pixels);
    }

    Ok(QuantizedData {
        frames: quantized_frames,
        palette: final_palette,
        transparency: final_transparency,
    })
}
