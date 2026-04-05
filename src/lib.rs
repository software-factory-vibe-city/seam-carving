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

    let mut rgba_data = img.to_rgba8().into_raw();
    let mut gray_data = img.to_luma8().into_raw();
    
    let mut current_w = width as usize;
    let h = height as usize;
    
    let mut energy = calculate_energy_forward(&gray_data, current_w, h);
    
    let total_seams = width - target_width;
    let mut seams_removed = 0;

    while current_w > target_width as usize {
        let seam = find_vertical_seam_optimized(&energy, current_w, h);
        
        remove_seam_in_place(&mut rgba_data, &seam, current_w, h, 4);
        remove_seam_in_place(&mut gray_data, &seam, current_w, h, 1);
        
        current_w -= 1;
        
        // Recalculate energy map for the new image size
        energy = calculate_energy_forward(&gray_data, current_w, h);
        
        seams_removed += 1;
        if let Some(cb) = progress_cb {
            cb(seams_removed as f32 / total_seams as f32);
        }
    }
    
    let final_rgba_data = rgba_data;
    let final_img = image::RgbaImage::from_raw(target_width, height, final_rgba_data).ok_or("Failed to reconstruct RGBA image")?;
    *img = DynamicImage::ImageRgba8(final_img);
    Ok(())
}

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
            get_sobel_energy_at(gray, x, y, w, h)
        }).collect::<Vec<_>>()
    }).collect()
}

fn calculate_energy_forward(gray: &[u8], w: usize, h: usize) -> Vec<f32> {
    (0..h).into_par_iter().flat_map(|y| {
        (0..w).map(move |x| {
            // Use Sobel energy as the base for forward energy calculation
            let base_energy = get_sobel_energy_at(gray, x, y, w, h);
            
            if x == 0 || x == w - 1 {
                return base_energy * 2.0; 
            }
            
            let l = x - 1;
            let r = x + 1;
            let val_l = gray[y * w + l] as f32;
            let val_x = gray[y * w + x] as f32;
            let val_r = gray[y * w + r] as f32;
            
            let current_diff = (val_l - val_x).abs() + (val_x - val_r).abs();
            let new_diff = (val_l - val_r).abs();
            
            let forward_cost = (new_diff - current_diff).max(0.0);
            
            // Combine local gradient (Sobel) with the forward cost
            // This prevents the algorithm from just picking a straight line 
            // and forces it to respect structural edges.
            base_energy + forward_cost
        }).collect::<Vec<_>>()
    }).collect()
}

fn get_sobel_x(gray: &[u8], x: usize, y: usize, w: usize, h: usize) -> f32 {
    let get_px = |nx: isize, ny: isize| {
        let cx = nx.clamp(0, (w - 1) as isize) as usize;
        let cy = ny.clamp(0, (h - 1) as isize) as usize;
        gray[cy * w + cx] as f32
    };

    let p00 = get_px(x as isize - 1, y as isize - 1);
    let p01 = get_px(x as isize + 1, y as isize - 1);
    let p10 = get_px(x as isize - 1, y as isize);
    let p11 = get_px(x as isize + 1, y as isize);
    let p20 = get_px(x as isize - 1, y as isize + 1);
    let p21 = get_px(x as isize + 1, y as isize + 1);

    -p00 + p01 - 2.0 * p10 + 2.0 * p11 - p20 + p21
}

fn get_sobel_y(gray: &[u8], x: usize, y: usize, w: usize, h: usize) -> f32 {
    let get_px = |nx: isize, ny: isize| {
        let cx = nx.clamp(0, (w - 1) as isize) as usize;
        let cy = ny.clamp(0, (h - 1) as isize) as usize;
        gray[cy * w + cx] as f32
    };

    let p00 = get_px(x as isize - 1, y as isize - 1);
    let p01 = get_px(x as isize, y as isize - 1);
    let p02 = get_px(x as isize + 1, y as isize - 1);
    let p20 = get_px(x as isize - 1, y as isize + 1);
    let p21 = get_px(x as isize, y as isize + 1);
    let p22 = get_px(x as isize + 1, y as isize + 1);

    -p00 - 2.0 * p01 - p02 + p20 + 2.0 * p21 + p22
}

fn get_sobel_energy_at(gray: &[u8], x: usize, y: usize, w: usize, h: usize) -> f32 {
    let gx = get_sobel_x(gray, x, y, w, h);
    let gy = get_sobel_y(gray, x, y, w, h);
    (gx * gx + gy * gy).sqrt()
}

fn find_vertical_seam_optimized(energy: &[f32], w: usize, h: usize) -> Vec<usize> {
    let mut dp_prev = vec![0.0; w];
    let mut dp_curr = vec![0.0; w];
    let mut pointers = vec![0; w * h];

    for x in 0..w {
        dp_prev[x] = energy[x];
    }

    for y in 1..h {
        let row_offset = y * w;
        for x in 0..w {
            let left = if x > 0 { dp_prev[x - 1] } else { f32::MAX };
            let mid = dp_prev[x];
            let right = if x < w - 1 { dp_prev[x + 1] } else { f32::MAX };
            
            let min_prev = left.min(mid).min(right);
            
            // Reduce diagonal penalty to 0.1 to allow seams to curve more naturally
            // while still preventing extreme jaggedness (stair-stepping).
            let penalty = if min_prev == mid { 0.0 } else { 0.1 };
            dp_curr[x] = energy[row_offset + x] + min_prev + penalty;
            
            pointers[row_offset + x] = if min_prev == left { x - 1 }
                                      else if min_prev == mid { x }
                                      else { x + 1 };
        }
        std::mem::swap(&mut dp_prev, &mut dp_curr);
    }

    let mut seam = vec![0; h];
    
    // Fix Left-Side Bias: 
    // Instead of just taking the first min_x, we can look for the "most central" 
    // minimum energy point or slightly bias the search.
    let mut min_x = 0;
    let mut min_val = dp_prev[0];
    for x in 1..w {
        // Using <= instead of < slightly shifts bias to the right,
        // preventing the algorithm from always hugging the left edge.
        if dp_prev[x] <= min_val {
            min_val = dp_prev[x];
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
            if x == sx { continue; }
            let src = row_start + x * channels;
            for c in 0..channels {
                new_data.push(data[src + c]);
            }
        }
    }
    *data = new_data;
}

fn update_energy_after_seam(energy: &mut Vec<f32>, gray: &[u8], seam: &[usize], w: usize, h: usize) {
    // The current logic for energy shifting is wrong. 
    // The `energy` vector represents the energy of the image AFTER removing a seam.
    // The current loop `energy[row_start + x] = energy[row_start + x + 1]` 
    // is attempting to shift values, but it doesn't align with how the energy map 
    // should be recalculated for the new dimensions.
    
    // The simplest and most correct way to update the energy map after a seam removal 
    // is to recalculate the energy for the affected pixels using the updated gray data.
    
    for y in 0..h {
        let sx = seam[y];
        // The image width is now (w), the original was (w+1)
        // We need to update the energy for pixels that were adjacent to the seam.
        for dx in -1..=1 {
            let nx = (sx as isize + dx).clamp(0, (w - 1) as isize) as usize;
            energy[y * w + nx] = get_forward_energy_at(gray, nx, y, w, h);
        }
    }
    // We must ensure the energy vector is the correct size.
    // However, calculate_energy_forward was called at the start.
    // In the loop, current_w decreases.
    // The energy vector should be recalculated or truncated.
}

fn get_forward_energy_at(gray: &[u8], x: usize, y: usize, w: usize, h: usize) -> f32 {
    let base = get_sobel_energy_at(gray, x, y, w, h);
    if x == 0 || x == w - 1 {
        return base * 2.0; 
    }
    
    let l = x - 1;
    let r = x + 1;
    let val_l = gray[y * w + l] as f32;
    let val_x = gray[y * w + x] as f32;
    let val_r = gray[y * w + r] as f32;
    
    let current_diff = (val_l - val_x).abs() + (val_x - val_r).abs();
    let new_diff = (val_l - val_r).abs();
    
    let forward_cost = (new_diff - current_diff).max(0.0);
    base + forward_cost
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
        resize_width(&mut img, target_width, None).unwrap();
        assert_eq!(img.width(), target_width);
        assert_eq!(img.height(), 100);
    }

    #[test]
    fn test_resize_height() {
        let mut img = create_test_image(100, 100);
        let target_height = 50;
        resize_height(&mut img, target_height, None).unwrap();
        assert_eq!(img.width(), 100);
        assert_eq!(img.height(), 50);
    }

    #[test]
    fn test_resize_both() {
        let mut img = create_test_image(100, 100);
        resize_width(&mut img, 60, None).unwrap();
        resize_height(&mut img, 40, None).unwrap();
        assert_eq!(img.width(), 60);
        assert_eq!(img.height(), 40);
    }

    #[test]
    fn test_resize_no_change() {
        let mut img = create_test_image(100, 100);
        resize_width(&mut img, 100, None).unwrap();
        resize_height(&mut img, 100, None).unwrap();
        assert_eq!(img.width(), 100);
        assert_eq!(img.height(), 100);
    }

    #[test]
    fn test_resize_to_one_pixel() {
        let mut img = create_test_image(10, 10);
        resize_width(&mut img, 1, None).unwrap();
        assert_eq!(img.width(), 1);
        assert_eq!(img.height(), 10);
        
        resize_height(&mut img, 1, None).unwrap();
        assert_eq!(img.width(), 1);
        assert_eq!(img.height(), 1);
    }

    #[test]
    fn test_resize_empty_image() {
        let mut img = create_test_image(1, 1);
        let res = resize_width(&mut img, 1, None);
        assert!(res.is_ok());
        assert_eq!(img.width(), 1);
    }

    #[test]
    fn test_resize_too_large() {
        let mut img = create_test_image(10, 10);
        let res = resize_width(&mut img, 20, None);
        assert!(res.is_err());
    }
}
