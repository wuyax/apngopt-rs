#[cfg(not(target_arch = "wasm32"))]
use clap::Parser;
#[cfg(not(target_arch = "wasm32"))]
use std::process;

#[cfg(not(target_arch = "wasm32"))]
use apngopt_rs::{decoder::load_apng, OptimizeOptions, optimize_apng_logic};
#[cfg(not(target_arch = "wasm32"))]
use std::fs::File;
#[cfg(not(target_arch = "wasm32"))]
use std::io::Write;

/// APNG Optimizer (Rust Port)
/// Optimizes APNG animations.
#[cfg(not(target_arch = "wasm32"))]
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

#[cfg(target_arch = "wasm32")]
fn main() {}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    let args = Args::parse();

    let out_file = args.output.clone().unwrap_or_else(|| {
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
            println!("Optimizing {} ({}x{}, {} frames)...", args.input, apng.width, apng.height, apng.frames.len());

            let options = OptimizeOptions {
                z_method: args.z_method,
                iterations: args.iterations,
                disable_imagequant: args.disable_imagequant,
            };

            match optimize_apng_logic(apng, &options) {
                Ok(optimized_data) => {
                    match File::create(&out_file) {
                        Ok(mut file) => {
                            if let Err(e) = file.write_all(&optimized_data) {
                                eprintln!("Error writing to output file: {}", e);
                                process::exit(1);
                            }
                            println!("Saved to {}.", out_file);
                        }
                        Err(e) => {
                            eprintln!("Error creating output file: {}", e);
                            process::exit(1);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Optimization failed: {}", e);
                    process::exit(1);
                }
            }
        }
        Err(e) => {
            eprintln!("Error loading APNG: {}", e);
            process::exit(1);
        }
    }
}
