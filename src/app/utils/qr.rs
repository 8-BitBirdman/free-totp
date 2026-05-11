use anywho::anywho;
use std::path::PathBuf;

pub fn read_qr_from_file(path: PathBuf) -> Result<Vec<String>, anywho::Error> {
    let img = image::open(&path)?;
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
