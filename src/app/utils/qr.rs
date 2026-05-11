use anywho::anywho;
use std::path::PathBuf;

pub fn read_qr_from_file(path: PathBuf) -> Result<Vec<String>, anywho::Error> {
    let mut original_img = image::open(&path)?;
    
    // If the image has an alpha channel, flatten it onto a white background
    if original_img.color().has_alpha() {
        use image::{Rgba, ImageBuffer, DynamicImage};
        let mut bg: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::from_pixel(original_img.width(), original_img.height(), Rgba([255, 255, 255, 255]));
        image::imageops::overlay(&mut bg, &original_img.to_rgba8(), 0, 0);
        original_img = DynamicImage::ImageRgba8(bg);
    }

    // Define a set of preprocessing attempts to make detection as failproof as possible
    // We try: Normal, Inverted, High Contrast, and Different Scaling
    let attempts: Vec<Box<dyn Fn(image::DynamicImage) -> image::DynamicImage>> = vec![
        // 1. Normal processing with standard resizing
        Box::new(|mut img: image::DynamicImage| {
            let max_dim = img.width().max(img.height());
            if max_dim > 2048 {
                let scale = 2048.0 / max_dim as f32;
                img = img.resize((img.width() as f32 * scale) as u32, (img.height() as f32 * scale) as u32, image::imageops::FilterType::Lanczos3);
            } else if max_dim < 512 {
                let scale = 512.0 / max_dim as f32;
                img = img.resize((img.width() as f32 * scale) as u32, (img.height() as f32 * scale) as u32, image::imageops::FilterType::Lanczos3);
            }
            img
        }),
        // 2. Inverted colors (for white-on-black QR codes)
        Box::new(|mut img: image::DynamicImage| {
            img.invert();
            img
        }),
        // 3. High contrast
        Box::new(|img: image::DynamicImage| {
            img.adjust_contrast(30.0)
        }),
        // 4. Raw image (no resizing)
        Box::new(|img: image::DynamicImage| img),
    ];

    for attempt_fn in attempts {
        let work_img = attempt_fn(original_img.clone());
        let luma = work_img.to_luma8();
        let mut prepared = rqrr::PreparedImage::prepare(luma);
        let grids = prepared.detect_grids();

        let mut contents = Vec::new();
        for grid in grids {
            if let Ok((_meta, content)) = grid.decode() {
                contents.push(content);
            }
        }

        if !contents.is_empty() {
            return Ok(contents);
        }
    }

    Err(anywho!("No QR code found in image after multiple attempts"))
}
