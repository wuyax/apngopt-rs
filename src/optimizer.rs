use crate::compress::compress_data;
use crate::filter::apply_filter;
#[cfg(not(target_arch = "wasm32"))]
use rayon::prelude::*;

#[derive(Clone)]
pub struct RectResult {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
    pub blend_op: u8,
    pub dispose_op: u8,
    pub data: Vec<u8>,
}

pub struct OptimizedFrame {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
    pub blend_op: u8,
    pub dispose_op: u8,
    pub delay_num: u16,
    pub delay_den: u16,
    pub compressed_data: Vec<u8>,
}

/// Calculate the minimum bounding box and the required pixel data for the current frame
/// given the previous canvas state. Returns both Source and Over candidates if possible.
pub fn get_rect_candidates(
    prev_canvas: &[u8],
    curr_canvas: &[u8],
    width: u32,
    height: u32,
    dispose_op: u8,
    bpp: usize,
    has_tcolor: bool,
    tcolor: u8,
) -> Vec<RectResult> {
    let w = width as usize;
    let h = height as usize;

    let mut x_min = w;
    let mut y_min = h;
    let mut x_max = 0;
    let mut y_max = 0;

    let mut diffnum = 0;
    let mut over_is_possible = if bpp == 1 && !has_tcolor { false } else { true };

    for y in 0..h {
        for x in 0..w {
            let idx = (y * w + x) * bpp;

            if bpp == 4 {
                let p1 = &prev_canvas[idx..idx + 4];
                let p2 = &curr_canvas[idx..idx + 4];
                let alpha1 = p1[3];
                let alpha2 = p2[3];

                if p1 != p2 && (alpha1 > 0 || alpha2 > 0) {
                    diffnum += 1;
                    if alpha2 != 255 {
                        over_is_possible = false;
                    }
                    if x < x_min {
                        x_min = x;
                    }
                    if x > x_max {
                        x_max = x;
                    }
                    if y < y_min {
                        y_min = y;
                    }
                    if y > y_max {
                        y_max = y;
                    }
                }
            } else if bpp == 1 {
                let p1 = prev_canvas[idx];
                let p2 = curr_canvas[idx];

                if p1 != p2 {
                    diffnum += 1;
                    if has_tcolor && p2 == tcolor {
                        over_is_possible = false;
                    }
                    if x < x_min {
                        x_min = x;
                    }
                    if x > x_max {
                        x_max = x;
                    }
                    if y < y_min {
                        y_min = y;
                    }
                    if y > y_max {
                        y_max = y;
                    }
                }
            }
        }
    }

    if diffnum == 0 {
        return Vec::new(); // Frames are identical
    }

    let rect_x = x_min as u32;
    let rect_y = y_min as u32;
    let rect_w = (x_max - x_min + 1) as u32;
    let rect_h = (y_max - y_min + 1) as u32;

    let mut candidates = Vec::new();

    // 1. Source (blend_op = 0)
    let mut source_data = Vec::with_capacity((rect_w * rect_h) as usize * bpp);
    for y in rect_y..(rect_y + rect_h) {
        let row_start = (y as usize * w + rect_x as usize) * bpp;
        let row_end = row_start + (rect_w as usize * bpp);
        source_data.extend_from_slice(&curr_canvas[row_start..row_end]);
    }
    candidates.push(RectResult {
        x: rect_x,
        y: rect_y,
        w: rect_w,
        h: rect_h,
        blend_op: 0,
        dispose_op,
        data: source_data,
    });

    // 2. Over (blend_op = 1)
    if over_is_possible {
        let mut over_data = Vec::with_capacity((rect_w * rect_h) as usize * bpp);
        for y in rect_y..(rect_y + rect_h) {
            for x in rect_x..(rect_x + rect_w) {
                let idx = (y as usize * w + x as usize) * bpp;

                if bpp == 4 {
                    let p1 = &prev_canvas[idx..idx + 4];
                    let p2 = &curr_canvas[idx..idx + 4];
                    let alpha1 = p1[3];
                    let alpha2 = p2[3];

                    if p1 != p2 && (alpha1 > 0 || alpha2 > 0) {
                        over_data.extend_from_slice(p2);
                    } else {
                        over_data.extend_from_slice(&[0, 0, 0, 0]);
                    }
                } else if bpp == 1 {
                    let p1 = prev_canvas[idx];
                    let p2 = curr_canvas[idx];

                    if p1 != p2 {
                        over_data.push(p2);
                    } else {
                        over_data.push(tcolor);
                    }
                }
            }
        }
        candidates.push(RectResult {
            x: rect_x,
            y: rect_y,
            w: rect_w,
            h: rect_h,
            blend_op: 1,
            dispose_op,
            data: over_data,
        });
    }

    candidates
}

pub fn optimize_rect(
    rect: &RectResult,
    compression_method: u8,
    iterations: u32,
    bpp: usize,
) -> Vec<u8> {
    // Try all 5 filters in parallel
    let filters = vec![0, 1, 2, 3, 4];

    #[cfg(not(target_arch = "wasm32"))]
    let iter = filters.into_par_iter();

    #[cfg(target_arch = "wasm32")]
    let iter = filters.into_iter();

    let best_compressed = iter
        .map(|filter_type| {
            let filtered_data = apply_filter(&rect.data, rect.w, rect.h, bpp, filter_type);
            compress_data(&filtered_data, compression_method, iterations)
        })
        .min_by_key(|data| data.len())
        .unwrap();

    best_compressed
}
