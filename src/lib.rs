use image::{DynamicImage, RgbImage, GenericImageView};
use rayon::prelude::*;

pub mod wasm;

pub fn resize_width(img: &mut DynamicImage, target_width: u32, progress_cb: Option<&dyn Fn(f32)>) -> Result<(), String> {
    let (width, height) = img.dimensions();
    if target_width > width {
        return Err(format!("Target width {} is greater than current width {}", target_width, width));
    }
    if target_width == width {
        return Ok(());
    }

    let mut rgb_data = img.to_rgb8().into_raw();
    let mut gray_data = img.to_luma8().into_raw();
    
    let mut current_w = width as usize;
    let h = height as usize;
    
    let mut energy = calculate_energy_sobel_parallel(&gray_data, current_w, h);
    
    let total_seams = width - target_width;
    let mut seams_removed = 0;

    while current_w > target_width as usize {
        let seam = find_vertical_seam_optimized(&energy, current_w, h);
        
        // In-place removal in RGB and Gray buffers
        remove_seam_in_place(&mut rgb_data, &seam, current_w, h, 3);
        remove_seam_in_place(&mut gray_data, &seam, current_w, h, 1);
        
        // Update energy for the affected columns
        update_energy_after_seam(&mut energy, &gray_data, &seam, current_w - 1, h);
        
        current_w -= 1;
        seams_removed += 1;
        if let Some(cb) = progress_cb {
            cb(seams_removed as f32 / total_seams as f32);
        }
    }
    
    // Ensure the buffer is exactly the size needed for the target dimensions
    // Use a new Vec with exact size to avoid any capacity issues in Wasm
    let final_rgb_data = rgb_data.clone();
    let final_img = RgbImage::from_raw(target_width, height, final_rgb_data).ok_or("Failed to reconstruct RGB image")?;
    *img = DynamicImage::ImageRgb8(final_img);
    Ok(())
}

/// Resizes the height of an image using the seam carving algorithm.
pub fn resize_height(img: &mut DynamicImage, target_height: u32, progress_cb: Option<&dyn Fn(f32)>) -> Result<(), String> {
    if target_height > img.height() {
        return Err(format!("Target height {} is greater than current height {}", target_height, img.height()));
    }
    
    *img = img.rotate90();
    resize_width(img, target_height, progress_cb)?;
    *img = img.rotate270();
    
    Ok(())
}

fn calculate_energy_sobel_parallel(gray: &[u8], w: usize, h: usize) -> Vec<f32> {
    (0..h).into_par_iter().flat_map(|y| {
        (0..w).map(move |x| {
            let gx = get_sobel_x(gray, x, y, w, h);
            let gy = get_sobel_y(gray, x, y, w, h);
            (gx * gx + gy * gy).sqrt()
        }).collect::<Vec<_>>()
    }).collect()
}

fn get_sobel_x(gray: &[u8], x: usize, y: usize, w: usize, h: usize) -> f32 {
    let get_px = |nx: isize, ny: isize| {
        let cx = nx.clamp(0, (w - 1) as isize) as usize;
        let cy = ny.clamp(0, (h - 1) as isize) as usize;
        gray[cy * w + cx] as f32
    };

    -1.0 * get_px(x as isize - 1, y as isize - 1)
    + 0.0 * get_px(x as isize, y as isize - 1)
    + 1.0 * get_px(x as isize + 1, y as isize - 1)
    - 2.0 * get_px(x as isize - 1, y as isize)
    + 0.0 * get_px(x as isize, y as isize)
    + 2.0 * get_px(x as isize + 1, y as isize)
    - 1.0 * get_px(x as isize - 1, y as isize + 1)
    + 0.0 * get_px(x as isize, y as isize + 1)
    + 1.0 * get_px(x as isize + 1, y as isize + 1)
}

fn get_sobel_y(gray: &[u8], x: usize, y: usize, w: usize, h: usize) -> f32 {
    let get_px = |nx: isize, ny: isize| {
        let cx = nx.clamp(0, (w - 1) as isize) as usize;
        let cy = ny.clamp(0, (h - 1) as isize) as usize;
        gray[cy * w + cx] as f32
    };

    -1.0 * get_px(x as isize - 1, y as isize - 1)
    - 2.0 * get_px(x as isize, y as isize - 1)
    - 1.0 * get_px(x as isize + 1, y as isize - 1)
    + 0.0 * get_px(x as isize - 1, y as isize)
    + 0.0 * get_px(x as isize, y as isize)
    + 0.0 * get_px(x as isize + 1, y as isize)
    + 1.0 * get_px(x as isize - 1, y as isize + 1)
    + 2.0 * get_px(x as isize, y as isize + 1)
    + 1.0 * get_px(x as isize + 1, y as isize + 1)
}

fn find_vertical_seam_optimized(energy: &[f32], w: usize, h: usize) -> Vec<usize> {
    let mut dp = vec![0.0; w];
    let mut pointers = vec![0; w * h];

    for x in 0..w {
        dp[x] = energy[x];
    }

    for y in 1..h {
        let row_offset = y * w;
        let mut next_dp = vec![0.0; w];
        
        for x in 0..w {
            let left = if x > 0 { dp[x - 1] } else { f32::MAX };
            let mid = dp[x];
            let right = if x < w - 1 { dp[x + 1] } else { f32::MAX };
            
            let min_prev = left.min(mid).min(right);
            next_dp[x] = energy[row_offset + x] + min_prev;
            
            pointers[row_offset + x] = if min_prev == left { x - 1 }
                                  else if min_prev == mid { x }
                                  else { x + 1 };
        }
        dp = next_dp;
    }

    let mut seam = vec![0; h];
    let mut min_x = 0;
    let mut min_val = dp[0];
    for x in 1..w {
        if dp[x] < min_val {
            min_val = dp[x];
            min_x = x;
        }
    }

    seam[h - 1] = min_x;
    for y in (0..h - 1).rev() {
        seam[y] = pointers[y * w + seam[y + 1]];
    }
    seam
}

fn remove_seam_in_place(data: &mut Vec<u8>, seam: &[usize], w: usize, h: usize, channels: usize) {
    let mut new_data = Vec::with_capacity((w - 1) * h * channels);
    for y in 0..h {
        let sx = seam[y];
        let row_start = y * w * channels;
        for x in 0..w {
            if x == sx {
                continue;
            }
            let src = row_start + x * channels;
            for c in 0..channels {
                new_data.push(data[src + c]);
            }
        }
    }
    *data = new_data;
}

fn update_energy_after_seam(energy: &mut Vec<f32>, gray: &[u8], seam: &[usize], w: usize, h: usize) {
    // The input 'w' is the width AFTER the seam has been removed from the RGB/Gray buffers, 
    // but the energy vector currently has (w + 1) elements per row.
    
    // 1. Remove the seam values from the energy vector.
    // Instead of allocating a new Vec every time, we shift elements in-place.
    for y in 0..h {
        let sx = seam[y];
        let row_start = y * (w + 1);
        
        // Shift elements to the left to overwrite the seam pixel
        for x in sx..w {
            energy[row_start + x] = energy[row_start + x + 1];
        }
    }
    energy.truncate(w * h);

    // 2. Recalculate energy ONLY for the pixels that were actually affected by the seam removal.
    for y in 0..h {
        let sx = seam[y];
        let new_x = if sx < w { sx } else { sx - 1 };
        
        for dx in -1..=1 {
            let nx = (new_x as isize + dx).clamp(0, (w - 1) as isize) as usize;
            energy[y * w + nx] = get_sobel_energy_at(gray, nx, y, w, h);
        }
    }
}

fn get_sobel_energy_at(gray: &[u8], x: usize, y: usize, w: usize, h: usize) -> f32 {
    let gx = get_sobel_x(gray, x, y, w, h);
    let gy = get_sobel_y(gray, x, y, w, h);
    (gx * gx + gy * gy).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{DynamicImage, RgbImage};

    fn create_test_image(width: u32, height: u32) -> DynamicImage {
        let mut img = RgbImage::new(width, height);
        for (x, y, pixel) in img.enumerate_pixels_mut() {
            *pixel = image::Rgb([ (x % 255) as u8, (y % 255) as u8, 0]);
        }
        DynamicImage::ImageRgb8(img)
    }

    #[test]
    fn test_resize_width() {
        let mut img = create_test_image(100, 100);
        let target_width = 50;
        resize_width(&mut img, target_width).unwrap();
        assert_eq!(img.width(), target_width);
        assert_eq!(img.height(), 100);
    }

    #[test]
    fn test_resize_height() {
        let mut img = create_test_image(100, 100);
        let target_height = 50;
        resize_height(&mut img, target_height).unwrap();
        assert_eq!(img.width(), 100);
        assert_eq!(img.height(), 50);
    }

    #[test]
    fn test_resize_both() {
        let mut img = create_test_image(100, 100);
        resize_width(&mut img, 60).unwrap();
        resize_height(&mut img, 40).unwrap();
        assert_eq!(img.width(), 60);
        assert_eq!(img.height(), 40);
    }

    #[test]
    fn test_resize_no_change() {
        let mut img = create_test_image(100, 100);
        resize_width(&mut img, 100).unwrap();
        resize_height(&mut img, 100).unwrap();
        assert_eq!(img.width(), 100);
        assert_eq!(img.height(), 100);
    }

    #[test]
    fn test_resize_to_one_pixel() {
        let mut img = create_test_image(10, 10);
        resize_width(&mut img, 1).unwrap();
        assert_eq!(img.width(), 1);
        assert_eq!(img.height(), 10);
        
        resize_height(&mut img, 1).unwrap();
        assert_eq!(img.width(), 1);
        assert_eq!(img.height(), 1);
    }

    #[test]
    fn test_resize_empty_image() {
        let mut img = create_test_image(1, 1);
        let res = resize_width(&mut img, 1);
        assert!(res.is_ok());
        assert_eq!(img.width(), 1);
    }

    #[test]
    fn test_resize_too_large() {
        let mut img = create_test_image(10, 10);
        let res = resize_width(&mut img, 20);
        assert!(res.is_err());
    }
}
