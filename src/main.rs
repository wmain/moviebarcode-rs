extern crate image;

use image::GenericImageView;
use std::fs;
use std::io;
use std::path::Path;

fn main() -> io::Result<()> {
    let path = Path::new("/Users/wmain/Desktop/escape/frames");
    let mut pending_barcode = PendingBarcode::new(path);
    pending_barcode.generate_pixels()?;
    pending_barcode.save_image()?;

    Ok(())
}

enum PendingBarcodeStatus {
    Initialized,
    PixelsGenerated,
    Saved,
}

struct PendingBarcode<'a> {
    status: PendingBarcodeStatus,
    frames_path: &'a Path,
    pixels: Vec<image::Rgb<u8>>,
}

impl<'a> PendingBarcode<'a> {
    pub fn new(frames_path: &Path) -> PendingBarcode {
        PendingBarcode {
            status: PendingBarcodeStatus::Initialized,
            frames_path: frames_path,
            pixels: vec![],
        }
    }

    pub fn generate_pixels(&mut self) -> io::Result<()> {
        for entry in fs::read_dir(self.frames_path)? {
            let frame_path = entry.unwrap().path();
            let pixel = PendingBarcode::get_avg_pixel_from_image(&frame_path);
            self.pixels.push(pixel);
        }

        self.status = PendingBarcodeStatus::PixelsGenerated;

        Ok(())
    }

    pub fn save_image(&mut self) -> io::Result<()> {
        let img = image::ImageBuffer::from_fn(754, 400, |row, _| self.pixels[row as usize]);

        img.save("output.png")?;
        self.status = PendingBarcodeStatus::Saved;

        Ok(())
    }

    fn get_avg_pixel_from_image(path: &Path) -> image::Rgb<u8> {
        let img = image::open(path).unwrap();
        let (width, height) = img.dimensions();

        let averages = img.pixels().fold([0u32; 3], |mut acc, pix| {
            let rgba = pix.2;
            let r = rgba[0] as u32;
            let g = rgba[1] as u32;
            let b = rgba[2] as u32;
            acc[0] += r;
            acc[1] += g;
            acc[2] += b;
            acc
        });
        let num_pixels = width * height;
        let r = f64::round(averages[0] as f64 / num_pixels as f64) as u8;
        let g = f64::round(averages[1] as f64 / num_pixels as f64) as u8;
        let b = f64::round(averages[2] as f64 / num_pixels as f64) as u8;

        image::Rgb([r, g, b])
    }
}
