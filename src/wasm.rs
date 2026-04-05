use wasm_bindgen::prelude::*;
use image::{DynamicImage, RgbImage};
use crate::{resize_width, resize_height};

#[wasm_bindgen]
pub fn resize_width_wasm(data: &[u8], width: u32, height: u32, target_width: u32, progress_cb: &js_sys::Function) -> Result<Vec<u8>, String> {
    // Use RgbaImage to match browser's native format and avoid RGB conversion
    let img_rgba = image::RgbaImage::from_raw(width, height, data.to_vec())
        .ok_or("Failed to create image from bytes")?;
    
    let mut img = DynamicImage::ImageRgba8(img_rgba);
    
    let wrapper = move |p: f32| {
        let _ = progress_cb.call1(&wasm_bindgen::JsValue::undefined(), &js_sys::Number::from(p));
    };

    resize_width(&mut img, target_width, Some(&wrapper))?;
    
    let mut raw = img.into_rgba8().into_raw();
    raw.shrink_to_fit();
    Ok(raw)
}

#[wasm_bindgen]
pub fn resize_height_wasm(data: &[u8], width: u32, height: u32, target_height: u32, progress_cb: &js_sys::Function) -> Result<Vec<u8>, String> {
    let img_rgba = image::RgbaImage::from_raw(width, height, data.to_vec())
        .ok_or("Failed to create image from bytes")?;
    
    let mut img = DynamicImage::ImageRgba8(img_rgba);
    
    let wrapper = move |p: f32| {
        let _ = progress_cb.call1(&wasm_bindgen::JsValue::undefined(), &js_sys::Number::from(p));
    };

    resize_height(&mut img, target_height, Some(&wrapper))?;
    
    let mut raw = img.into_rgba8().into_raw();
    raw.shrink_to_fit();
    Ok(raw)
}
