pub mod chunk;
pub mod compress;
pub mod decoder;
pub mod encoder;
pub mod filter;
pub mod optimizer;
pub mod quantize;
pub mod reconstruct;

#[cfg(target_arch = "wasm32")]
use decoder::load_apng_from_memory;
use decoder::ApngImage;
use encoder::save_apng_to_memory;
use optimizer::{optimize_rect, OptimizedFrame};
use reconstruct::reconstruct_frames;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

pub struct OptimizeOptions {
    pub z_method: u8,
    pub iterations: u32,
    pub disable_imagequant: u8,
}

pub fn optimize_apng_logic(
    apng: ApngImage,
    options: &OptimizeOptions,
) -> Result<Vec<u8>, String> {
    let mut full_frames = reconstruct_frames(&apng);

    let mut palette = None;
    let mut transparency = None;
    let mut bpp = 4;
    let mut has_tcolor = false;
    let mut tcolor = 0;

    if options.disable_imagequant == 0 {
        match quantize::quantize_frames(&full_frames, apng.width, apng.height) {
            Ok(quantized) => {
                full_frames = quantized.frames;
                palette = Some(quantized.palette);
                transparency = Some(quantized.transparency);
                bpp = 1;

                // Find transparent color index
                if let Some(trns) = &transparency {
                    if let Some(idx) = trns.iter().position(|&a| a == 0) {
                        has_tcolor = true;
                        tcolor = idx as u8;
                    } else if let Some(idx) = trns.iter().position(|&a| a < 255) {
                        has_tcolor = true;
                        tcolor = idx as u8;
                    }
                }
            }
            Err(e) => {
                eprintln!("Quantization failed: {}. Proceeding without quantization.", e);
            }
        }
    }

    let mut optimized_frames: Vec<OptimizedFrame> = Vec::new();
    for i in 0..full_frames.len() {
        let curr_canvas = &full_frames[i];
        let candidates = if i == 0 {
            let source_data = curr_canvas.clone();
            vec![optimizer::RectResult {
                x: 0,
                y: 0,
                w: apng.width,
                h: apng.height,
                blend_op: 0,
                dispose_op: 0,
                data: source_data,
            }]
        } else {
            let prev_canvas = &full_frames[i - 1];
            optimizer::get_rect_candidates(
                prev_canvas,
                curr_canvas,
                apng.width,
                apng.height,
                0,
                bpp,
                has_tcolor,
                tcolor,
            )
        };

        if candidates.is_empty() {
            if let Some(last) = optimized_frames.last_mut() {
                if last.delay_den == apng.frames[i].delay_den {
                    last.delay_num += apng.frames[i].delay_num;
                }
            }
        } else {
            let mut best_compressed: Option<(Vec<u8>, optimizer::RectResult)> = None;

            for cand in candidates {
                let compressed = optimize_rect(&cand, options.z_method, options.iterations, bpp);
                if best_compressed.is_none()
                    || compressed.len() < best_compressed.as_ref().unwrap().0.len()
                {
                    best_compressed = Some((compressed, cand));
                }
            }

            let (compressed, cand) = best_compressed.unwrap();

            optimized_frames.push(OptimizedFrame {
                x: cand.x,
                y: cand.y,
                w: cand.w,
                h: cand.h,
                blend_op: cand.blend_op,
                dispose_op: cand.dispose_op,
                delay_num: apng.frames[i].delay_num,
                delay_den: apng.frames[i].delay_den,
                compressed_data: compressed,
            });
        }
    }

    save_apng_to_memory(
        apng.width,
        apng.height,
        apng.loops,
        &optimized_frames,
        palette.as_deref(),
        transparency.as_deref(),
    )
    .map_err(|e| format!("Error saving APNG: {}", e))
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn optimize_apng_wasm(
    input_data: &[u8],
    z_method: u8,
    iterations: u32,
    disable_imagequant: u8,
) -> Result<Vec<u8>, JsValue> {
    let options = OptimizeOptions {
        z_method,
        iterations,
        disable_imagequant,
    };
    let apng = load_apng_from_memory(input_data).map_err(|e| JsValue::from_str(&e))?;
    optimize_apng_logic(apng, &options).map_err(|e| JsValue::from_str(&e))
}