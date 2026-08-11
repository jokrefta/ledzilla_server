//! Thread safe upload manager to store and retrieve user's uploaded files
//! Also handles conversions such as resizing images.
//!
//! At least in the initial iteration, everything will be stored solely in RAM.

use std::{collections::BTreeMap, sync::Arc};

use thiserror::Error;

#[derive(Debug)]
pub enum UploadedAsset {
    Image(ImageBuf),
    AnimatedImage(AnimatedImageBuf),
}

#[derive(Debug, Error)]
pub enum UploadError {
    #[error("Image error: {0}")]
    ImageError(image::ImageError),
    #[error("I/O error?? {0}")]
    IoError(std::io::Error),
    #[error("Detected image format is not a supported type: {0:?}")]
    UnsupportedImageType(image::ImageFormat),
}

pub type ResizeFilter = image::imageops::FilterType;
pub fn resize_filter_from_str(s: &str) -> Option<ResizeFilter> {
    match s.to_ascii_lowercase().as_str() {
        "nearest_neighbor" => Some(ResizeFilter::Nearest),
        "bilinear" => Some(ResizeFilter::Triangle),
        "bicubic" => Some(ResizeFilter::CatmullRom),
        _ => None,
    }
}

#[derive(Debug, Clone)]
pub struct ResizeOptions {
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub filter: ResizeFilter,
}

fn resize_image(img: image::DynamicImage, opts: &ResizeOptions) -> image::DynamicImage {
    if let Some(width) = opts.width
        && let Some(height) = opts.height
    {
        // Both dimensions provided - resize exact
        img.resize_exact(width, height, opts.filter)
    } else if opts.width.is_some() || opts.height.is_some() {
        // Single dimension provided - maintain aspect ratio
        let width = opts.width.unwrap_or_else(|| img.width());
        let height = opts.height.unwrap_or_else(|| img.height());
        img.resize(width, height, opts.filter)
    } else {
        // No resizing to do
        img
    }
}

/// Holds a non-animated image buffer
#[derive(Debug)]
pub struct ImageBuf {
    image: image::RgbImage,
}

impl ImageBuf {
    /// Create from a buffer containing an image encoded in a supported filetype (e.g, PNG).
    /// The format will be auto detected.
    #[allow(unused)]
    pub fn from_encoded_buffer(buf: Vec<u8>) -> Result<Self, UploadError> {
        Self::from_encoded_buffer_resize(buf, &None)
    }

    /// Create from a buffer containing an image encoded in a supported filetype (e.g, PNG).
    /// The format will be auto detected.
    pub fn from_encoded_buffer_resize(
        buf: Vec<u8>,
        resize: &Option<ResizeOptions>,
    ) -> Result<Self, UploadError> {
        let cursor = std::io::Cursor::new(buf);
        let image_reader = image::ImageReader::new(cursor)
            .with_guessed_format()
            .map_err(UploadError::IoError)?;

        let mut d_img: image::DynamicImage = image_reader.decode().map_err(UploadError::ImageError)?;

        if let Some(opts) = resize {
            d_img = resize_image(d_img, opts);
        }
        Ok(Self { image: d_img.into() })
    }

    pub fn get_width(&self) -> u32 {
        self.image.width()
    }

    #[allow(dead_code)]
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

    pub fn from_frame(f: image::Frame, resize: &Option<ResizeOptions>) -> Self {
        // They don't have a direct conversion between RgbaImage and RgbImage for some reason, so we have to go to DynamicImage first
        // But that's okay, we can use the handy DynamicImage::resize which preserves aspect ratio.
        let mut d_img = image::DynamicImage::from(f.into_buffer());
        if let Some(opts) = resize {
            d_img = resize_image(d_img, opts);
        }

        Self { img: d_img.into() }
    }
}

/// Stores all frames from an animated image.
/// Assumes all frames are the same size (is it even possible for them not to be??)
#[derive(Debug)]
pub struct AnimatedImageBuf {
    frames: Vec<RgbFrame>,
}

impl AnimatedImageBuf {
    #[allow(unused)]
    pub fn from_encoded_buffer(buf: Vec<u8>) -> Result<Self, UploadError> {
        Self::from_encoded_buffer_resize(buf, &None)
    }

    /// Create from a buffer containing an image encoded in a supported filetype (e.g, GIF).
    pub fn from_encoded_buffer_resize(
        buf: Vec<u8>,
        resize: &Option<ResizeOptions>,
    ) -> Result<Self, UploadError> {
        use image::AnimationDecoder;

        log::trace!("Creating AnimatedImageBuf... resize={:?}", resize);

        let cursor = std::io::Cursor::new(buf);
        match image::guess_format(cursor.get_ref()) {
            Ok(image::ImageFormat::Gif) => {
                let decoder = image::codecs::gif::GifDecoder::new(cursor).map_err(UploadError::ImageError)?;

                let frames = decoder
                    .into_frames()
                    .collect_frames()
                    .map_err(UploadError::ImageError)?;
                Ok(Self {
                    frames: frames
                        .into_iter()
                        .map(|f| RgbFrame::from_frame(f, resize))
                        .collect(),
                })
            }
            Ok(f) => Err(UploadError::UnsupportedImageType(f)),
            Err(_) => todo!("Handle failed guess format"),
        }
    }

    pub fn get_width(&self) -> u32 {
        self.frames[0].img.width()
    }

    #[allow(dead_code)]
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

    fn init_logger() {
        // Uncomment below to use logger - only really works when running a single test at a time
        /*
        simple_logger::SimpleLogger::new()
            .with_level(log::LevelFilter::Trace)
            .init()
            .unwrap();
        */
        log::info!("LOG IS RUNNING");
    }
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
        init_logger();
        const FILENAME: &str = "vertical_gradient.png";
        const WIDTH: u32 = 10;
        const HEIGHT: u32 = 10;
        let buf = load_test_file(FILENAME);
        let image = ImageBuf::from_encoded_buffer(buf).unwrap();

        assert_eq!(WIDTH, image.get_width());
        assert_eq!(HEIGHT, image.get_height());

        let raw_rgb = image.get_rgb_raw();
        assert_eq!((32, 128, 32), get_pixel_from_raw(raw_rgb, 0, 0, WIDTH));
        assert_eq!((32, 128, 32), get_pixel_from_raw(raw_rgb, 1, 0, WIDTH));
        assert_eq!((88, 152, 106), get_pixel_from_raw(raw_rgb, 6, 3, WIDTH));
    }

    #[test]
    fn new_png_resize() {
        init_logger();
        const FILENAME: &str = "vertical_gradient.png";
        const EXP_RESIZED_WIDTH: u32 = 6;
        const EXP_RESIZED_HEIGHT: u32 = 6;
        let buf = load_test_file(FILENAME);
        let resize = ResizeOptions {
            width: Some(EXP_RESIZED_WIDTH),
            height: None,
            filter: ResizeFilter::Nearest,
        };
        let image = ImageBuf::from_encoded_buffer_resize(buf, &Some(resize)).unwrap();

        assert_eq!(EXP_RESIZED_WIDTH, image.get_width());
        assert_eq!(EXP_RESIZED_HEIGHT, image.get_height());
        // Just checking dimensions for now.
    }

    #[test]
    fn new_animated_gif() {
        init_logger();
        const FILENAME: &str = "gradient.gif";
        const WIDTH: u32 = 10;
        const HEIGHT: u32 = 10;
        const FRAMES: usize = 2;

        let buf = load_test_file(FILENAME);
        let image = AnimatedImageBuf::from_encoded_buffer(buf).unwrap();

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

    #[test]
    fn new_animated_gif_resize() {
        init_logger();
        const FILENAME: &str = "gradient.gif";
        const EXP_RESIZED_WIDTH: u32 = 6;
        const EXP_RESIZED_HEIGHT: u32 = 6;
        const FRAMES: usize = 2;

        let resize = ResizeOptions {
            width: Some(EXP_RESIZED_WIDTH),
            height: None,
            filter: ResizeFilter::Nearest,
        };

        let buf = load_test_file(FILENAME);
        let image = AnimatedImageBuf::from_encoded_buffer_resize(buf, &Some(resize)).unwrap();

        assert_eq!(EXP_RESIZED_WIDTH, image.get_width());
        assert_eq!(EXP_RESIZED_HEIGHT, image.get_height());

        let frames: &[RgbFrame] = image.get_frames();
        assert_eq!(FRAMES, frames.len());
    }

    #[test]
    fn new_animated_gif_resize_squish() {
        init_logger();
        const FILENAME: &str = "gradient.gif";
        const EXP_RESIZED_WIDTH: u32 = 10;
        const EXP_RESIZED_HEIGHT: u32 = 6;
        const FRAMES: usize = 2;

        let resize = ResizeOptions {
            width: Some(EXP_RESIZED_WIDTH),
            height: Some(EXP_RESIZED_HEIGHT),
            filter: ResizeFilter::Nearest,
        };

        let buf = load_test_file(FILENAME);
        let image = AnimatedImageBuf::from_encoded_buffer_resize(buf, &Some(resize)).unwrap();

        assert_eq!(EXP_RESIZED_WIDTH, image.get_width());
        assert_eq!(EXP_RESIZED_HEIGHT, image.get_height());

        let frames: &[RgbFrame] = image.get_frames();
        assert_eq!(FRAMES, frames.len());
    }
}
