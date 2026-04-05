use clap::Parser;
use std::path::PathBuf;
use seam_stitching::{resize_width, resize_height};

#[derive(Parser, Debug)]
#[command(author, version, about = "Seam carving for image resizing", long_about = None)]
struct Args {
    /// Input image path
    #[arg(short, long)]
    input: PathBuf,

    /// Output image path
    #[arg(short, long)]
    output: PathBuf,

    /// Target width
    #[arg(short, long)]
    width: Option<u32>,

    /// Target height
    #[arg(short = 'H', long)]
    height: Option<u32>,
}

fn main() {
    let args = Args::parse();

    let mut img = match image::open(&args.input) {
        Ok(img) => img,
        Err(e) => {
            eprintln!("Failed to open input image: {}", e);
            std::process::exit(1);
        }
    };

    if let Some(w) = args.width {
        if w < img.width() {
            println!("Resizing width from {} to {}...", img.width(), w);
            if let Err(e) = resize_width(&mut img, w) {
                eprintln!("Error resizing width: {}", e);
                std::process::exit(1);
            }
        } else {
            println!("Target width {} is greater than or equal to current width {}. Skipping.", w, img.width());
        }
    }

    if let Some(h) = args.height {
        if h < img.height() {
            println!("Resizing height from {} to {}...", img.height(), h);
            if let Err(e) = resize_height(&mut img, h) {
                eprintln!("Error resizing height: {}", e);
                std::process::exit(1);
            }
        } else {
            println!("Target height {} is greater than or equal to current height {}. Skipping.", h, img.height());
        }
    }

    if let Err(e) = img.save(&args.output) {
        eprintln!("Failed to save output image: {}", e);
        std::process::exit(1);
    }
    println!("Image saved to {:?}", args.output);
}