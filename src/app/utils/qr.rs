use anywho::anywho;
use std::path::PathBuf;

pub fn read_qr_from_file(path: PathBuf) -> Result<Vec<String>, anywho::Error> {
    let mut img = image::open(&path)?;
    
    // If the image has an alpha channel, flatten it onto a white background
    if img.color().has_alpha() {
        use image::{Rgba, ImageBuffer, DynamicImage};
        let mut bg: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::from_pixel(img.width(), img.height(), Rgba([255, 255, 255, 255]));
        image::imageops::overlay(&mut bg, &img.to_rgba8(), 0, 0);
        img = DynamicImage::ImageRgba8(bg);
    }

    // Resize image if it's too large or too small for reliable detection
    // Optimal range is usually between 512 and 2048 pixels
    let (width, height) = (img.width(), img.height());
    let max_dim = width.max(height);
    if max_dim > 2048 {
        let scale = 2048.0 / max_dim as f32;
        img = img.resize((width as f32 * scale) as u32, (height as f32 * scale) as u32, image::imageops::FilterType::Lanczos3);
    } else if max_dim < 512 {
        let scale = 512.0 / max_dim as f32;
        img = img.resize((width as f32 * scale) as u32, (height as f32 * scale) as u32, image::imageops::FilterType::Lanczos3);
    }

    let img = img.to_luma8();
    let mut img = rqrr::PreparedImage::prepare(img);
    let grids = img.detect_grids();

    let mut contents = Vec::new();
    for grid in grids {
        if let Ok((_meta, content)) = grid.decode() {
            contents.push(content);
        }
    }

    if contents.is_empty() {
        Err(anywho!("No QR code found in image"))
    } else {
        Ok(contents)
    }
}
