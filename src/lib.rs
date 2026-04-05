use image::{DynamicImage, RgbImage, GrayImage};

/// Resizes the width of an image using the seam carving algorithm.
///
/// This function iteratively identifies and removes the vertical seam with the lowest
/// energy (lowest visual importance) until the target width is reached.
///
/// # Arguments
/// * `img` - A mutable reference to the `DynamicImage` to be resized.
/// * `target_width` - The desired width of the output image.
pub fn resize_width(img: &mut DynamicImage, target_width: u32) -> Result<(), String> {
    if target_width > img.width() {
        return Err(format!("Target width {} is greater than current width {}", target_width, img.width()));
    }
    let mut current_img = img.to_rgb8();
    let mut gray = DynamicImage::ImageRgb8(current_img.clone()).into_luma8();
    let mut width = current_img.width();
    let height = current_img.height();
    let mut energy = calculate_energy_floats(&current_img, &gray);
    
    while current_img.width() > target_width {
        let seam = find_vertical_seam_with_energy(&energy, width, height);
        current_img = remove_vertical_seam(&current_img, &seam);
        gray = remove_vertical_seam_gray(&gray, &seam);
        update_energy_after_vertical_seam(&mut energy, &gray, &seam, &mut width);
        
        if current_img.width() % 10 == 0 || current_img.width() == target_width {
            // Progress logging removed for performance/cleanliness
        }
    }
    *img = DynamicImage::ImageRgb8(current_img);
    Ok(())
}

/// Resizes the height of an image using the seam carving algorithm.
///
/// This function iteratively identifies and removes the horizontal seam with the lowest
/// energy (lowest visual importance) until the target height is reached.
///
/// # Arguments
/// * `img` - A mutable reference to the `DynamicImage` to be resized.
/// * `target_height` - The desired height of the output image.
pub fn resize_height(img: &mut DynamicImage, target_height: u32) -> Result<(), String> {
    if target_height > img.height() {
        return Err(format!("Target height {} is greater than current height {}", target_height, img.height()));
    }
    let mut current_img = img.to_rgb8();
    let mut gray = DynamicImage::ImageRgb8(current_img.clone()).into_luma8();
    let mut height = current_img.height();
    let width = current_img.width();
    let mut energy = calculate_energy_floats_horizontal(&current_img, &gray);

    while current_img.height() > target_height {
        let seam = find_horizontal_seam_with_energy(&energy, width, height);
        current_img = remove_horizontal_seam(&current_img, &seam);
        gray = remove_horizontal_seam_gray(&gray, &seam);
        update_energy_after_horizontal_seam(&mut energy, &gray, &seam, &mut height);

        if current_img.height() % 10 == 0 || current_img.height() == target_height {
            // Progress logging removed for performance/cleanliness
        }
    }
    *img = DynamicImage::ImageRgb8(current_img);
    Ok(())
}


fn calculate_pixel_energy(gray: &GrayImage, x: u32, y: u32) -> f32 {
    let (width, height) = gray.dimensions();
    let dx = if x > 0 && x < width - 1 {
        let left = gray.get_pixel(x - 1, y).0[0];
        let right = gray.get_pixel(x + 1, y).0[0];
        (left as f32 - right as f32).abs()
    } else if x == 0 && width > 1 {
        let right = gray.get_pixel(x + 1, y).0[0];
        let center = gray.get_pixel(x, y).0[0];
        (center as f32 - right as f32).abs()
    } else if x == width - 1 && width > 1 {
        let left = gray.get_pixel(x - 1, y).0[0];
        let center = gray.get_pixel(x, y).0[0];
        (center as f32 - left as f32).abs()
    } else {
        0.0
    };
    let dy = if y > 0 && y < height - 1 {
        let top = gray.get_pixel(x, y - 1).0[0];
        let bottom = gray.get_pixel(x, y + 1).0[0];
        (top as f32 - bottom as f32).abs()
    } else if y == 0 && height > 1 {
        let bottom = gray.get_pixel(x, y + 1).0[0];
        let center = gray.get_pixel(x, y).0[0];
        (center as f32 - bottom as f32).abs()
    } else if y == height - 1 && height > 1 {
        let top = gray.get_pixel(x, y - 1).0[0];
        let center = gray.get_pixel(x, y).0[0];
        (center as f32 - top as f32).abs()
    } else {
        0.0
    };
    (dx * dx + dy * dy).sqrt()
}

fn calculate_energy_floats(img: &RgbImage, gray: &GrayImage) -> Vec<f32> {
    let (width, height) = img.dimensions();
    let mut energy = vec![0.0; (width * height) as usize];
    
    for y in 0..height {
        let y_u32 = y;
        for x in 0..width {
            energy[(y * width + x) as usize] = calculate_pixel_energy(gray, x, y_u32);
        }
    }
    energy
}

fn find_vertical_seam_with_energy(energy: &[f32], width: u32, height: u32) -> Vec<usize> {
    if width == 0 { return vec![]; }
    let w = width as usize;
    let h = height as usize;

    let mut dp = vec![0.0; w * h];
    
    // Initialize first row
    for x in 0..w {
        dp[x] = energy[x];
    }

    for y in 1..h {
        let prev_row_offset = (y - 1) * w;
        let curr_row_offset = y * w;
        for x in 0..w {
            let left = if x > 0 { dp[prev_row_offset + x - 1] } else { f32::MAX };
            let mid = dp[prev_row_offset + x];
            let right = if x < w - 1 { dp[prev_row_offset + x + 1] } else { f32::MAX };
            
            dp[curr_row_offset + x] = energy[curr_row_offset + x] + left.min(mid).min(right);
        }
    }

    let mut seam = vec![0; h];
    let mut min_x = 0;
    let last_row_offset = (h - 1) * w;
    let mut min_val = dp[last_row_offset];
    for x in 1..w {
        if dp[last_row_offset + x] < min_val {
            min_val = dp[last_row_offset + x];
            min_x = x;
        }
    }

    seam[h-1] = min_x;
    for y in (0..h-1).rev() {
        let x = seam[y+1];
        let row_offset = y * w;
        let mut best_x = x;
        let mut current_min = dp[row_offset + x];

        if x > 0 && dp[row_offset + x - 1] < current_min {
            current_min = dp[row_offset + x - 1];
            best_x = x - 1;
        }
        if x < w - 1 && dp[row_offset + x + 1] < current_min {
            best_x = x + 1;
        }
        seam[y] = best_x;
    }

    seam
}

fn update_energy_after_vertical_seam(energy: &mut Vec<f32>, gray: &GrayImage, seam: &[usize], width: &mut u32) {
    let h = gray.height() as usize;
    let w_val = *width as usize;
    
    let mut new_energy = Vec::with_capacity((w_val - 1) * h);
    for y in 0..h {
        let sx = seam[y];
        let row_start = y * w_val;
        let row_end = row_start + w_val;
        let row = &energy[row_start..row_end];
        for x in 0..w_val {
            if x != sx {
                new_energy.push(row[x]);
            }
        }
    }
    *energy = new_energy;
    *width -= 1;

    let current_w = (*width) as usize;
    for y in 0..h {
        let sx = seam[y];
        let y_u32 = y as u32;

        if sx > 0 {
            let x_idx = sx - 1;
            energy[y * current_w + x_idx] = calculate_pixel_energy(gray, x_idx as u32, y_u32);
        }
        if sx < current_w {
            let x_idx = sx;
            energy[y * current_w + x_idx] = calculate_pixel_energy(gray, x_idx as u32, y_u32);
        }
    }
}

fn calculate_energy_floats_horizontal(img: &RgbImage, gray: &GrayImage) -> Vec<f32> {
    let (width, height) = img.dimensions();
    let mut energy = vec![0.0; (width * height) as usize];
    for y in 0..height {
        for x in 0..width {
            energy[(y * width + x) as usize] = calculate_pixel_energy(gray, x, y);
        }
    }
    energy
}

fn find_horizontal_seam_with_energy(energy: &[f32], width: u32, height: u32) -> Vec<usize> {
    if height == 0 { return vec![]; }
    let w = width as usize;
    let h = height as usize;

    let mut dp = vec![0.0; w * h];
    
    for y in 0..h {
        dp[y] = energy[y * w];
    }

    for x in 1..w {
        let prev_col_offset = (x - 1) * h;
        let curr_col_offset = x * h;
        for y in 0..h {
            let top = if y > 0 { dp[prev_col_offset + y - 1] } else { f32::MAX };
            let mid = dp[prev_col_offset + y];
            let bot = if y < h - 1 { dp[prev_col_offset + y + 1] } else { f32::MAX };
            
            dp[curr_col_offset + y] = energy[y * w + x] + top.min(mid).min(bot);
        }
    }

    let mut seam = vec![0; w];
    let mut min_y = 0;
    let last_col_offset = (w - 1) * h;
    let mut min_val = dp[last_col_offset];
    for y in 1..h {
        if dp[last_col_offset + y] < min_val {
            min_val = dp[last_col_offset + y];
            min_y = y;
        }
    }

    seam[w-1] = min_y;
    for x in (0..w-1).rev() {
        let y = seam[x+1];
        let col_offset = x * h;
        let mut best_y = y;
        let mut current_min = dp[col_offset + y];

        if y > 0 && dp[col_offset + y - 1] < current_min {
            current_min = dp[col_offset + y - 1];
            best_y = y - 1;
        }
        if y < h - 1 && dp[col_offset + y + 1] < current_min {
            best_y = y + 1;
        }
        seam[x] = best_y;
    }

    seam
}

fn update_energy_after_horizontal_seam(energy: &mut Vec<f32>, gray: &GrayImage, seam: &[usize], height: &mut u32) {
    let w = gray.width() as usize;
    let h_val = *height as usize;
    
    let mut new_energy = Vec::with_capacity(w * (h_val - 1));
    for y in 0..h_val {
        for x in 0..w {
            if y != seam[x] {
                new_energy.push(energy[y * w + x]);
            }
        }
    }
    *energy = new_energy;
    *height -= 1;

    let current_h = (*height) as usize;
    for x in 0..w {
        let sy = seam[x];
        let x_u32 = x as u32;

        if sy > 0 {
            let y_idx = if sy - 1 < current_h { sy - 1 } else { current_h - 1 };
            energy[y_idx * w + x] = calculate_pixel_energy(gray, x_u32, y_idx as u32);
        }
        if sy < h_val {
            let y_idx = if sy < current_h { sy } else { current_h - 1 };
            energy[y_idx * w + x] = calculate_pixel_energy(gray, x_u32, y_idx as u32);
        }
    }
}

fn remove_horizontal_seam_gray(img: &GrayImage, seam: &[usize]) -> GrayImage {
    let (width, height) = img.dimensions();
    let mut new_img = GrayImage::new(width, height - 1);

    for x in 0..width {
        let seam_y = seam[x as usize];
        let mut current_y = 0;
        for y in 0..height {
            if y as usize == seam_y {
                continue;
            }
            new_img.put_pixel(x, current_y, *img.get_pixel(x, y));
            current_y += 1;
        }
    }
    new_img
}

fn remove_horizontal_seam(img: &RgbImage, seam: &[usize]) -> RgbImage {
    let (width, height) = img.dimensions();
    let mut new_img = RgbImage::new(width, height - 1);

    for x in 0..width {
        let seam_y = seam[x as usize];
        let mut current_y = 0;
        for y in 0..height {
            if y as usize == seam_y {
                continue;
            }
            new_img.put_pixel(x, current_y, *img.get_pixel(x, y));
            current_y += 1;
        }
    }
    new_img
}

fn remove_vertical_seam_gray(img: &GrayImage, seam: &[usize]) -> GrayImage {
    let (width, height) = img.dimensions();
    let mut new_img = GrayImage::new(width - 1, height);

    for y in 0..height {
        let seam_x = seam[y as usize];
        let mut current_x = 0;
        for x in 0..width {
            if x as usize == seam_x {
                continue;
            }
            new_img.put_pixel(current_x, y, *img.get_pixel(x, y));
            current_x += 1;
        }
    }
    new_img
}

fn remove_vertical_seam(img: &RgbImage, seam: &[usize]) -> RgbImage {
    let (width, height) = img.dimensions();
    let mut new_img = RgbImage::new(width - 1, height);

    for y in 0..height {
        let seam_x = seam[y as usize];
        let mut current_x = 0;
        for x in 0..width {
            if x as usize == seam_x {
                continue;
            }
            new_img.put_pixel(current_x, y, *img.get_pixel(x, y));
            current_x += 1;
        }
    }
    new_img
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
        assert_eq!(img.height(), target_height);
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
        // image crate usually doesn't allow 0 dimensions, but we test 1x1
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
