use criterion::{black_box, criterion_group, criterion_main, Criterion};
use image::{DynamicImage, RgbImage};
use seam_stitching::{resize_width, resize_height};

fn create_test_image(width: u32, height: u32) -> DynamicImage {
    let mut img = RgbImage::new(width, height);
    for (x, y, pixel) in img.enumerate_pixels_mut() {
        *pixel = image::Rgb([ (x % 255) as u8, (y % 255) as u8, 0]);
    }
    DynamicImage::ImageRgb8(img)
}

fn bench_resize_width(c: &mut Criterion) {
    let mut img = create_test_image(200, 200);
    c.bench_function("resize_width_200_to_150", |b| {
        b.iter(|| {
            let mut test_img = img.clone();
            let _ = resize_width(black_box(&mut test_img), black_box(150));
        })
    });
}

fn bench_resize_height(c: &mut Criterion) {
    let mut img = create_test_image(200, 200);
    c.bench_function("resize_height_200_to_150", |b| {
        b.iter(|| {
            let mut test_img = img.clone();
            let _ = resize_height(black_box(&mut test_img), black_box(150));
        })
    });
}

criterion_group!(benches, bench_resize_width, bench_resize_height);
criterion_main!(benches);
