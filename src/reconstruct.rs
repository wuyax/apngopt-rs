use crate::decoder::{ApngImage, Frame};

pub fn reconstruct_frames(apng: &ApngImage) -> Vec<Vec<u8>> {
    let w = apng.width as usize;
    let h = apng.height as usize;
    let bytes_per_pixel = 4; // we assumed EXPAND | STRIP_16 => RGBA8
    let row_stride = w * bytes_per_pixel;
    let full_size = h * row_stride;

    let mut full_frames = Vec::with_capacity(apng.frames.len());
    let mut canvas = vec![0u8; full_size];

    for (i, frame) in apng.frames.iter().enumerate() {
        let prev_canvas_for_dispose = canvas.clone();

        // Blend current frame onto canvas
        blend(&mut canvas, frame, w, bytes_per_pixel);

        full_frames.push(canvas.clone());

        // Dispose logic for the NEXT frame
        // APNG_DISPOSE_OP_NONE = 0
        // APNG_DISPOSE_OP_BACKGROUND = 1
        // APNG_DISPOSE_OP_PREVIOUS = 2
        match frame.dispose_op {
            0 => {
                // Do nothing, leave canvas as is
            }
            1 => {
                // Clear the region to fully transparent black
                clear_region(&mut canvas, frame, w, bytes_per_pixel);
            }
            2 => {
                // Revert to previous canvas (before this frame was blended)
                // However, for the very first frame, APNG_DISPOSE_OP_PREVIOUS is treated as OP_BACKGROUND
                if i == 0 {
                    clear_region(&mut canvas, frame, w, bytes_per_pixel);
                } else {
                    canvas = prev_canvas_for_dispose;
                }
            }
            _ => {}
        }
    }

    full_frames
}

fn blend(canvas: &mut [u8], frame: &Frame, canvas_w: usize, bpp: usize) {
    let fw = frame.width as usize;
    let fh = frame.height as usize;
    let fx = frame.x_offset as usize;
    let fy = frame.y_offset as usize;

    let mut src_idx = 0;
    for y in 0..fh {
        let canvas_y = fy + y;
        let dest_idx_start = (canvas_y * canvas_w + fx) * bpp;

        for x in 0..fw {
            let dp = dest_idx_start + x * bpp;
            let sp = src_idx;

            // APNG_BLEND_OP_SOURCE = 0
            // APNG_BLEND_OP_OVER = 1
            if frame.blend_op == 0 {
                // Overwrite
                canvas[dp..dp + 4].copy_from_slice(&frame.data[sp..sp + 4]);
            } else {
                // Alpha composite (OVER)
                let src_a = frame.data[sp + 3] as u32;
                if src_a == 255 {
                    canvas[dp..dp + 4].copy_from_slice(&frame.data[sp..sp + 4]);
                } else if src_a > 0 {
                    let dst_a = canvas[dp + 3] as u32;
                    if dst_a != 0 {
                        let u = src_a * 255;
                        let v = (255 - src_a) * dst_a;
                        let al = u + v;

                        canvas[dp] =
                            ((frame.data[sp] as u32 * u + canvas[dp] as u32 * v) / al) as u8;
                        canvas[dp + 1] = ((frame.data[sp + 1] as u32 * u
                            + canvas[dp + 1] as u32 * v)
                            / al) as u8;
                        canvas[dp + 2] = ((frame.data[sp + 2] as u32 * u
                            + canvas[dp + 2] as u32 * v)
                            / al) as u8;
                        canvas[dp + 3] = (al / 255) as u8;
                    } else {
                        canvas[dp..dp + 4].copy_from_slice(&frame.data[sp..sp + 4]);
                    }
                }
            }
            src_idx += bpp;
        }
    }
}

fn clear_region(canvas: &mut [u8], frame: &Frame, canvas_w: usize, bpp: usize) {
    let fw = frame.width as usize;
    let fh = frame.height as usize;
    let fx = frame.x_offset as usize;
    let fy = frame.y_offset as usize;

    for y in 0..fh {
        let canvas_y = fy + y;
        let dest_idx_start = (canvas_y * canvas_w + fx) * bpp;
        for x in 0..fw {
            let dp = dest_idx_start + x * bpp;
            canvas[dp..dp + 4].fill(0); // transparent black
        }
    }
}
