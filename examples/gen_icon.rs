//! One-shot generator for `assets/icon.ico` (256/64/48/32/16 px).
//!
//! Run with: `cargo run --example gen_icon`

use rust_note::ui::icon::render_icon_rgba;
use std::io::Write;
use std::path::Path;

/// Encodes one RGBA image as a BMP-in-ICO entry (BITMAPINFOHEADER + bottom-up BGRA + AND mask).
fn ico_entry(size: u32, rgba: &[u8]) -> Vec<u8> {
    let w = size;
    let h = size;
    let row_bits = w as usize;
    let and_stride = (row_bits + 31) / 32 * 4; // 1 bit per pixel, padded to 4 bytes

    let mut data = Vec::with_capacity(40 + (w * h * 4) as usize + and_stride * h as usize);
    // BITMAPINFOHEADER (height counts the AND mask rows)
    data.extend_from_slice(&40u32.to_le_bytes());
    data.extend_from_slice(&w.to_le_bytes());
    data.extend_from_slice(&(h * 2).to_le_bytes());
    data.extend_from_slice(&1u16.to_le_bytes()); // planes
    data.extend_from_slice(&32u16.to_le_bytes()); // bpp
    data.extend_from_slice(&0u32.to_le_bytes()); // compression
    data.extend_from_slice(&0u32.to_le_bytes()); // image size
    data.extend_from_slice(&0u32.to_le_bytes()); // x pels
    data.extend_from_slice(&0u32.to_le_bytes()); // y pels
    data.extend_from_slice(&0u32.to_le_bytes()); // colors used
    data.extend_from_slice(&0u32.to_le_bytes()); // colors important
    assert_eq!(data.len(), 40);

    // Pixels, bottom-up, BGRA
    for y in (0..h).rev() {
        for x in 0..w {
            let idx = (((y * w) + x) * 4) as usize;
            data.push(rgba[idx + 2]); // B
            data.push(rgba[idx + 1]); // G
            data.push(rgba[idx]); // R
            data.push(rgba[idx + 3]); // A
        }
    }
    // All-zero AND mask (alpha lives in the 32bpp pixels)
    data.resize(data.len() + and_stride * h as usize, 0);
    data
}

fn write_ico(path: &Path, sizes: &[u32]) -> std::io::Result<()> {
    let entries: Vec<(u32, Vec<u8>)> = sizes
        .iter()
        .map(|&s| (s, ico_entry(s, &render_icon_rgba(s))))
        .collect();

    let mut ico = Vec::new();
    ico.extend_from_slice(&0u16.to_le_bytes()); // reserved
    ico.extend_from_slice(&1u16.to_le_bytes()); // type: icon
    ico.extend_from_slice(&(entries.len() as u16).to_le_bytes());

    let mut offset = 6 + 16 * entries.len();
    for &(size, ref data) in &entries {
        let dim = u8::try_from(size).unwrap_or(0); // 0 encodes 256
        ico.push(dim); // width
        ico.push(dim); // height
        ico.push(0); // palette
        ico.push(0); // reserved
        ico.extend_from_slice(&1u16.to_le_bytes()); // planes
        ico.extend_from_slice(&32u16.to_le_bytes()); // bpp
        ico.extend_from_slice(&(data.len() as u32).to_le_bytes());
        ico.extend_from_slice(&(offset as u32).to_le_bytes());
        offset += data.len();
    }
    for (_, data) in &entries {
        ico.extend_from_slice(data);
    }

    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let mut f = std::fs::File::create(path)?;
    f.write_all(&ico)?;
    Ok(())
}

fn main() {
    let out = Path::new("assets/icon.ico");
    match write_ico(out, &[256, 64, 48, 32, 16]) {
        Ok(()) => println!("wrote {}", out.display()),
        Err(e) => {
            eprintln!("failed to write {}: {e}", out.display());
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ico_structure_is_wellformed() {
        let data = ico_entry(16, &render_icon_rgba(16));
        assert_eq!(data.len(), 40 + 16 * 16 * 4 + 4 * 16);
        // Header says 32bpp, width 16, height 32 (16 XOR + 16 AND)
        assert_eq!(&data[0..4], &40u32.to_le_bytes());
        assert_eq!(&data[4..8], &16u32.to_le_bytes());
        assert_eq!(&data[8..12], &32u32.to_le_bytes());
    }
}
