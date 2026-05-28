use crate::flags::TextureFlags;
use crate::{vtf::VTF, Error, ImageFormat};
use image::{DynamicImage, GenericImageView};

#[derive(Clone, Debug)]
pub struct VTFBuilder {
    frames: Vec<DynamicImage>,
    image_format: ImageFormat,
    first_frame: u16,
    mipmaps: bool,
    flags: Option<TextureFlags>,
    bumpmap_scale: f32,
}

impl VTFBuilder {
    pub fn new(image_format: ImageFormat) -> Self {
        VTFBuilder {
            frames: Vec::new(),
            image_format,
            first_frame: 0,
            mipmaps: false,
            flags: None,
            bumpmap_scale: 1.0,
        }
    }

    pub fn add_frame(mut self, image: DynamicImage) -> Result<Self, Error> {
        let w = image.width();
        let h = image.height();

        if w == 0 || h == 0 || w > u16::MAX as u32 || h > u16::MAX as u32 {
            return Err(Error::InvalidImageSize);
        }

        if self.image_format.is_block_compressed() && (w % 4 != 0 || h % 4 != 0) {
            return Err(Error::InvalidImageSize);
        }

        if let Some(first) = self.frames.first() {
            if image.dimensions() != first.dimensions() {
                return Err(Error::MismatchedFrameDimensions);
            }
        }

        self.frames.push(image);

        Ok(self)
    }

    pub fn set_first_frame(mut self, first_frame: u16) -> Self {
        self.first_frame = first_frame;
        self
    }

    pub fn with_mipmaps(mut self, enabled: bool) -> Self {
        self.mipmaps = enabled;
        self
    }

    pub fn with_flags(mut self, flags: TextureFlags) -> Self {
        self.flags = Some(flags);
        self
    }

    pub fn set_bumpmap_scale(mut self, scale: f32) -> Self {
        self.bumpmap_scale = scale;
        self
    }

    pub fn build(self) -> Result<Vec<u8>, Error> {
        if self.frames.is_empty() {
            return Err(Error::NoFrames);
        }

        if self.first_frame >= self.frames.len() as u16 {
            return Err(Error::InvalidFirstFrame);
        }

        VTF::encode(
            &self.frames,
            self.image_format,
            self.first_frame,
            self.mipmaps,
            self.flags,
            self.bumpmap_scale,
        )
    }
}
