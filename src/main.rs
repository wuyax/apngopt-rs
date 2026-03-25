use clap::Parser;
use std::process;

mod chunk;
mod compress;
mod decoder;
mod encoder;
mod filter;
mod optimizer;
mod quantize;
mod reconstruct;

use decoder::load_apng;
use encoder::save_apng;
use optimizer::{optimize_rect, OptimizedFrame};
use reconstruct::reconstruct_frames;

/// APNG Optimizer (Rust Port)
/// Optimizes APNG animations.
#[derive(Parser, Debug)]
#[command(name = "apngopt-rs", version = "1.4.1", about = "APNG Optimizer in Rust", long_about = None)]
pub struct Args {
    /// Compression method: 0=zlib (default), 1=7zip, 2=zopfli
    #[arg(short = 'z', default_value_t = 0)]
    pub z_method: u8,

    /// Number of iterations
    #[arg(short = 'i', default_value_t = 15)]
    pub iterations: u32,

    /// Disable imagequant compress 0 or 1
    #[arg(short = 'd', default_value_t = 0)]
    pub disable_imagequant: u8,

    /// Input PNG file
    #[arg(value_name = "anim.png")]
    pub input: String,

    /// Output PNG file (optional)
    #[arg(value_name = "anim_opt.png")]
    pub output: Option<String>,
}

fn main() {
    let args = Args::parse();
    println!("APNG Optimizer 1.4.1 (Rust Edition)");
    println!("Input: {}", args.input);

    let out_file = args.output.unwrap_or_else(|| {
        let path = std::path::Path::new(&args.input);
        let stem = path.file_stem().unwrap().to_str().unwrap();
        let ext = path
            .extension()
            .unwrap_or_default()
            .to_str()
            .unwrap_or("png");
        format!("{}_opt.{}", stem, ext)
    });

    match load_apng(&args.input) {
        Ok(apng) => {
            println!("APNG loaded successfully!");
            println!("Size: {}x{}", apng.width, apng.height);
            println!("Loops: {}", apng.loops);
            println!("Frames: {}", apng.frames.len());

            let mut full_frames = reconstruct_frames(&apng);
            println!(
                "Successfully reconstructed {} full frames.",
                full_frames.len()
            );

            let mut palette = None;
            let mut transparency = None;
            let mut bpp = 4;
            let mut has_tcolor = false;
            let mut tcolor = 0;

            if args.disable_imagequant == 0 {
                println!("Quantizing frames with libimagequant...");
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
                        println!(
                            "Quantization complete. Palette size: {}",
                            palette.as_ref().unwrap().len()
                        );
                    }
                    Err(e) => {
                        eprintln!(
                            "Quantization failed: {}. Proceeding without quantization.",
                            e
                        );
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
                    // We simply assume APNG_DISPOSE_OP_NONE (0) for now.
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
                    println!("Frame {} is identical to previous, skipping...", i);
                    if let Some(last) = optimized_frames.last_mut() {
                        if last.delay_den == apng.frames[i].delay_den {
                            last.delay_num += apng.frames[i].delay_num;
                        }
                    }
                } else {
                    // Find the best compressed candidate among all candidates (Source, Over)
                    let mut best_compressed: Option<(Vec<u8>, optimizer::RectResult)> = None;

                    for cand in candidates {
                        let compressed = optimize_rect(&cand, args.z_method, args.iterations, bpp);
                        if best_compressed.is_none()
                            || compressed.len() < best_compressed.as_ref().unwrap().0.len()
                        {
                            best_compressed = Some((compressed, cand));
                        }
                    }

                    let (compressed, cand) = best_compressed.unwrap();
                    println!(
                        "Frame {} bounded {}x{} at ({},{}) [Blend: {}] => compressed to {} bytes",
                        i,
                        cand.w,
                        cand.h,
                        cand.x,
                        cand.y,
                        cand.blend_op,
                        compressed.len()
                    );

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
            println!(
                "Optimization completed. Final frame count: {}",
                optimized_frames.len()
            );

            println!("Saving to {}...", out_file);
            if let Err(e) = save_apng(
                &out_file,
                apng.width,
                apng.height,
                apng.loops,
                &optimized_frames,
                palette.as_deref(),
                transparency.as_deref(),
            ) {
                eprintln!("Error saving APNG: {}", e);
                process::exit(1);
            }
            println!("All done!");
        }
        Err(e) => {
            eprintln!("Error loading APNG: {}", e);
            process::exit(1);
        }
    }
}
