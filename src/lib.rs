#![warn(clippy::all, clippy::cargo_common_metadata)]
#![deny(
    future_incompatible,
    // FIXME: Enable this and add documentation to everything public!
    // missing_docs,
    missing_debug_implementations,
    missing_copy_implementations,
    rust_2018_idioms,
    trivial_casts,
    trivial_numeric_casts,
    unused_import_braces,
    unused_qualifications,
    clippy::wildcard_dependencies
)]
#![forbid(unsafe_code)]

use std::{
    error::Error,
    fs,
    io::{self, Read},
    path::Path,
    process::{Command, ExitStatus, Stdio},
};

use image::GenericImageView;

const WIDTH: f32 = 1024.0;
const HEIGHT: f32 = 400.0;

#[derive(Debug)]
pub struct Config<'a> {
    input_video_path: &'a str,
    output_image_path: &'a str,
}

impl<'a> Config<'a> {
    pub fn new(args: &'a [String]) -> Result<Config<'a>, &'static str> {
        if args.len() < 3 {
            return Err("Not enough arguments");
        }

        Ok(Config {
            input_video_path: &args[1],
            output_image_path: &args[2],
        })
    }
}

pub fn run(config: &Config<'_>) -> io::Result<()> {
    let duration = get_video_duration(config.input_video_path).unwrap();
    let dividend = get_fps_dividend(duration);
    extract_frames(config.input_video_path, dividend).expect("Failed to extract frames");
    println!("Done extracting frames");
    let pixels = generate_pixels("./frames").expect("Failed to generate pixels");
    save_image(&pixels, config.output_image_path)?;
    Ok(())
}

fn get_video_duration(path: &str) -> Result<u32, Box<dyn Error>> {
    let mut cmd = Command::new("ffprobe")
        .args(&[
            "-v",
            "error",
            "-show_entries",
            "format=duration",
            "-of",
            "default=noprint_wrappers=1:nokey=1",
            &path,
        ])
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let stdout = cmd.stdout.as_mut().unwrap();

    let mut buf = String::new();
    stdout.read_to_string(&mut buf)?;
    let duration = buf.parse::<f64>()?.round() as u32;

    cmd.wait().unwrap();

    Ok(duration)
}

fn get_fps_dividend(duration_in_s: u32) -> f32 {
    // Add 10 to width to ensure we have enough frames
    // for img width. Better way to do this? Feels like the math
    // is right but some input videos of short duration don't
    // result in a dividend with enough frames.
    duration_in_s as f32 / (WIDTH + 10.0)
}

fn extract_frames(path: &str, fps_dividend: f32) -> io::Result<ExitStatus> {
    let fps_arg = format!("fps=1/{}", fps_dividend);
    Command::new("ffmpeg")
        .args(&[
            "-i",
            path,
            "-vf",
            &fps_arg,
            "frames/frame%04d.jpg",
            "-hide_banner",
        ])
        .spawn()
        .unwrap()
        .wait()
}

fn generate_pixels(frames_path: &str) -> io::Result<Vec<image::Rgb<u8>>> {
    let path = Path::new(frames_path);
    let pixels = fs::read_dir(path)?
        .map(|entry| {
            let frame_path = entry.unwrap().path();
            get_avg_pixel_from_image(&frame_path)
        })
        .collect();
    Ok(pixels)
}

fn get_avg_pixel_from_image(path: &Path) -> image::Rgb<u8> {
    let img = image::open(path).unwrap();
    let (width, height) = img.dimensions();

    let averages = img.pixels().fold([0f64; 3], |mut acc, (_, _, rgba)| {
        let r = f64::from(rgba[0]);
        let g = f64::from(rgba[1]);
        let b = f64::from(rgba[2]);
        acc[0] += r * r;
        acc[1] += g * g;
        acc[2] += b * b;
        acc
    });
    // What happens if the image has no pixels?
    let num_pixels = f64::from(width * height);
    let r = (averages[0] / num_pixels).round() as u8;
    let g = (averages[1] / num_pixels).round() as u8;
    let b = (averages[2] / num_pixels).round() as u8;

    image::Rgb([r, g, b])
}

fn save_image(pixels: &[image::Rgb<u8>], output_image_path: &str) -> io::Result<()> {
    let img =
        image::ImageBuffer::from_fn(WIDTH as u32, HEIGHT as u32, |row, _| pixels[row as usize]);

    img.save(output_image_path)?;
    Ok(())
}
