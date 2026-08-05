//! Thread safe upload manager to store and retrieve user's uploaded files
//! Also handles conversions such as resizing images.
//!
//! At least in the initial iteration, everything will be stored solely in RAM.

use std::{collections::BTreeMap, sync::Arc};

use log::trace;

#[derive(Debug)]
pub enum UploadedAsset {
    Image(ImageBuf),
    AnimatedImage(AnimatedImageBuf),
}

/// Holds a non-animated image buffer
#[derive(Debug)]
pub struct ImageBuf {
    image: image::RgbImage,
}

impl ImageBuf {
    /// Create from a buffer containing an image encoded in a supported filetype (e.g, PNG).
    /// The format will be auto detected.
    pub fn from_encoded_buffer(buf: Vec<u8>) -> Self {
        // TODO don't unwrap, check error
        let cursor = std::io::Cursor::new(buf);
        let image_reader = image::ImageReader::new(cursor).with_guessed_format().unwrap();
        let img: image::RgbImage = image_reader.decode().unwrap().into();
        Self { image: img }
    }

    pub fn get_width(&self) -> u32 {
        self.image.width()
    }

    pub fn get_height(&self) -> u32 {
        self.image.height()
    }

    pub fn get_rgb_raw(&self) -> &[u8] {
        &self.image
    }
}

/// Single animation frame represented in RGB form
#[derive(Debug)]
pub struct RgbFrame {
    img: image::RgbImage,
    // May add timing info here eventually
}

impl RgbFrame {
    pub fn get_rgb_raw(&self) -> &[u8] {
        &self.img
    }
}

impl From<image::Frame> for RgbFrame {
    fn from(f: image::Frame) -> Self {
        // They don't have a direct conversion between RgbaImage and RgbImage for some reason, so we have to go to DynamicImage first
        let img = image::RgbImage::from(image::DynamicImage::from(f.into_buffer()));
        Self { img }
    }
}

/// Stores all frames from an animated image.
/// Assumes all frames are the same size (is it even possible for them not to be??)
#[derive(Debug)]
pub struct AnimatedImageBuf {
    frames: Vec<RgbFrame>,
}

impl AnimatedImageBuf {
    /// Create from a buffer containing an image encoded in a supported filetype (e.g, GIF).
    pub fn from_encoded_buffer(buf: Vec<u8>) -> Self {
        use image::AnimationDecoder;

        let cursor = std::io::Cursor::new(buf);
        match image::guess_format(cursor.get_ref()) {
            Ok(image::ImageFormat::Gif) => {
                // TODO don't unwrap, check error
                let decoder = image::codecs::gif::GifDecoder::new(cursor).unwrap();

                //TODO get rid of unwrap
                let frames = decoder.into_frames().collect_frames().unwrap();
                Self {
                    frames: frames.into_iter().map(|x| RgbFrame::from(x)).collect(),
                }
            }
            Ok(_) => todo!("Handle unsupported animated image type"),
            Err(_) => todo!("Handle failed guess format"),
        }
    }

    pub fn get_width(&self) -> u32 {
        self.frames[0].img.width()
    }

    pub fn get_height(&self) -> u32 {
        self.frames[0].img.height()
    }

    pub fn get_frames(&self) -> &[RgbFrame] {
        &self.frames
    }
}

#[derive(Debug)]
pub struct UploadManager {
    /// Name -> asset. Use an Arc to allow readers to get a reference to an asset
    /// that remains valid even if it gets modified or deleted from the map.
    assets: BTreeMap<String, Arc<UploadedAsset>>,
}

impl UploadManager {
    pub fn new() -> Self {
        Self {
            assets: BTreeMap::new(),
        }
    }

    /// If it already existed, replace it and return a reference to the old
    pub fn insert(&mut self, filename: String, asset: UploadedAsset) -> Option<Arc<UploadedAsset>> {
        self.assets.insert(filename, Arc::new(asset))
    }

    pub fn retrieve(&self, filename: &str) -> Option<Arc<UploadedAsset>> {
        Some(self.assets.get(filename)?.clone())
    }

    pub fn list_files(&self) -> Vec<&str> {
        self.assets.keys().map(|s| s.as_str()).collect()
    }

    pub fn try_delete(&mut self, key: &str) -> Result<(), String> {
        match self.assets.remove(key) {
            Some(_) => Ok(()),
            None => Err("Can't delete file as it is not present".to_string()),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    // Load file from assets/test/
    fn load_test_file(name: &str) -> Vec<u8> {
        let path: std::path::PathBuf = [env!("CARGO_MANIFEST_DIR"), "assets", "test", name]
            .iter()
            .collect();
        std::fs::read(path).unwrap()
    }

    fn get_pixel_from_raw(raw_buf: &[u8], x: u32, y: u32, width: u32) -> (u8, u8, u8) {
        let pixel_start = 3 * (y * width + x) as usize;
        (
            raw_buf[pixel_start],
            raw_buf[pixel_start + 1],
            raw_buf[pixel_start + 2],
        )
    }

    #[test]
    fn new_png() {
        const FILENAME: &str = "vertical_gradient.png";
        const WIDTH: u32 = 10;
        const HEIGHT: u32 = 10;
        let buf = load_test_file(FILENAME);
        let image = ImageBuf::from_encoded_buffer(buf);

        assert_eq!(WIDTH, image.get_width());
        assert_eq!(HEIGHT, image.get_height());

        let raw_rgb = image.get_rgb_raw();
        assert_eq!((32, 128, 32), get_pixel_from_raw(raw_rgb, 0, 0, WIDTH));
        assert_eq!((32, 128, 32), get_pixel_from_raw(raw_rgb, 1, 0, WIDTH));
        assert_eq!((88, 152, 106), get_pixel_from_raw(raw_rgb, 6, 3, WIDTH));
    }

    #[test]
    fn new_animated_gif() {
        const FILENAME: &str = "gradient.gif";
        const WIDTH: u32 = 10;
        const HEIGHT: u32 = 10;
        const FRAMES: usize = 2;

        let buf = load_test_file(FILENAME);
        let image = AnimatedImageBuf::from_encoded_buffer(buf);

        assert_eq!(WIDTH, image.get_width());
        assert_eq!(HEIGHT, image.get_height());

        let frames: &[RgbFrame] = image.get_frames();
        assert_eq!(FRAMES, frames.len());

        let frame_0 = &frames[0].get_rgb_raw();
        assert_eq!((32, 128, 32), get_pixel_from_raw(frame_0, 0, 0, WIDTH));
        assert_eq!((32, 128, 32), get_pixel_from_raw(frame_0, 1, 0, WIDTH));
        assert_eq!((88, 152, 106), get_pixel_from_raw(frame_0, 6, 3, WIDTH));

        let frame_1 = &frames[1].get_rgb_raw();
        assert_eq!((32, 128, 32), get_pixel_from_raw(frame_1, 0, 0, WIDTH));
        assert_eq!((32, 128, 32), get_pixel_from_raw(frame_1, 0, 1, WIDTH));
        assert_eq!((88, 152, 106), get_pixel_from_raw(frame_1, 3, 6, WIDTH));
    }
}
