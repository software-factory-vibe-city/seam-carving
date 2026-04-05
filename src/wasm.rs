use wasm_bindgen::prelude::*;
use image::{DynamicImage, RgbImage};
use crate::{resize_width, resize_height};

#[wasm_bindgen]
pub fn resize_width_wasm(data: &[u8], width: u32, height: u32, target_width: u32, progress_cb: &js_sys::Function) -> Result<Vec<u8>, String> {
    // 1. Create RgbImage from raw bytes
    let img_rgb = RgbImage::from_raw(width, height, data.to_vec())
        .ok_or("Failed to create image from bytes")?;
    
    let mut img = DynamicImage::ImageRgb8(img_rgb);
    
    // 2. Call core logic with a closure that calls the JS function
    let wrapper = move |p: f32| {
        let _ = progress_cb.call1(&wasm_bindgen::JsValue::undefined(), &js_sys::Number::from(p));
    };

    resize_width(&mut img, target_width, Some(&wrapper))?;
    
    // 3. Extract raw bytes back
    Ok(img.into_rgb8().into_raw())
}

#[wasm_bindgen]
pub fn resize_height_wasm(data: &[u8], width: u32, height: u32, target_height: u32, progress_cb: &js_sys::Function) -> Result<Vec<u8>, String> {
    let img_rgb = RgbImage::from_raw(width, height, data.to_vec())
        .ok_or("Failed to create image from bytes")?;
    
    let mut img = DynamicImage::ImageRgb8(img_rgb);
    
    let wrapper = move |p: f32| {
        let _ = progress_cb.call1(&wasm_bindgen::JsValue::undefined(), &js_sys::Number::from(p));
    };

    resize_height(&mut img, target_height, Some(&wrapper))?;
    
    // Extract raw bytes back
    Ok(img.into_rgb8().into_raw())
}
