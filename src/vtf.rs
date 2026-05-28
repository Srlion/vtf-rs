use crate::builder::VTFBuilder;
use crate::flags::TextureFlags;
use crate::header::VTFHeader;
use crate::image::{ImageFormat, VTFImage};
use crate::resources::{ResourceList, ResourceType};
use crate::Error;
use image::imageops::FilterType;
use image::DynamicImage;
use std::io::Cursor;
use std::vec::Vec;
use texpresso::{Format, Params};

#[derive(Debug)]
pub struct VTF<'a> {
    pub header: VTFHeader,
    pub lowres_image: VTFImage<'a>,
    pub highres_image: VTFImage<'a>,
}

/// Default flags used when none are explicitly provided
const DEFAULT_FLAGS: u32 = 8972;

/// Compute the full mip chain count for a given width and height.
///
/// e.g. 256x256 -> 9 levels (256, 128, 64, 32, 16, 8, 4, 2, 1)
fn compute_mipmap_count(width: u32, height: u32) -> u8 {
    let max_dim = width.max(height);
    (32 - max_dim.leading_zeros()) as u8
}

/// Generate a downscaled version of the image at the given mip level.
///
/// Level 0 = original size, level 1 = half, etc.
fn generate_mip(image: &DynamicImage, level: u32) -> DynamicImage {
    if level == 0 {
        return image.clone();
    }
    let w = (image.width() >> level).max(1);
    let h = (image.height() >> level).max(1);
    image.resize_exact(w, h, FilterType::Lanczos3)
}

/// Encode a single image (at whatever mip dimensions) into the target format.
///
/// Returns the encoded bytes.
fn encode_image_data(
    image: &DynamicImage,
    image_format: ImageFormat,
    width: usize,
    height: usize,
) -> Result<Vec<u8>, Error> {
    match image_format {
        ImageFormat::Dxt5 => {
            let image_data = image.to_rgba8();
            let size = Format::Bc3.compressed_size(width, height);
            let mut buf = vec![0u8; size];
            Format::Bc3.compress(
                image_data.as_raw(),
                width,
                height,
                Params::default(),
                &mut buf,
            );
            Ok(buf)
        }
        ImageFormat::Dxt1Onebitalpha => {
            let image_data = image.to_rgba8();
            let size = Format::Bc1.compressed_size(width, height);
            let mut buf = vec![0u8; size];
            Format::Bc1.compress(
                image_data.as_raw(),
                width,
                height,
                Params::default(),
                &mut buf,
            );
            Ok(buf)
        }
        ImageFormat::Rgba8888 => {
            let image_data = image.to_rgba8();
            Ok(image_data.into_raw())
        }
        ImageFormat::Rgb888 => {
            let image_data = image.to_rgb8();
            Ok(image_data.into_raw())
        }
        _ => Err(Error::UnsupportedEncodeImageFormat(image_format)),
    }
}

impl<'a> VTF<'a> {
    pub fn create_animated(image_format: ImageFormat) -> VTFBuilder {
        VTFBuilder::new(image_format)
    }

    pub fn read(bytes: &'a [u8]) -> Result<VTF<'a>, Error> {
        let mut cursor = Cursor::new(bytes);

        let header = VTFHeader::read(&mut cursor)?;

        let lowres_offset = match header
            .resources
            .get_by_type(ResourceType::VTF_LEGACY_RSRC_LOW_RES_IMAGE)
        {
            Some(resource) => resource.data,
            None => header.header_size,
        };

        let highres_offset = match header
            .resources
            .get_by_type(ResourceType::VTF_LEGACY_RSRC_IMAGE)
        {
            Some(resource) => resource.data,
            None => {
                lowres_offset
                    + header.lowres_image_format.frame_size(
                        header.lowres_image_width as u32,
                        header.lowres_image_height as u32,
                    )?
            }
        };

        let lowres_image = VTFImage::new(
            header.clone(),
            header.lowres_image_format,
            header.lowres_image_width as u16,
            header.lowres_image_height as u16,
            bytes,
            lowres_offset as usize,
        );

        let highres_image = VTFImage::new(
            header.clone(),
            header.highres_image_format,
            header.width,
            header.height,
            bytes,
            highres_offset as usize,
        );

        Ok(VTF {
            header,
            lowres_image,
            highres_image,
        })
    }

    pub(crate) fn encode(
        frames: &[DynamicImage],
        image_format: ImageFormat,
        first_frame: u16,
        mipmaps: bool,
        flags: Option<TextureFlags>,
        bumpmap_scale: f32,
    ) -> Result<Vec<u8>, Error> {
        if frames.len() > u16::MAX as usize {
            return Err(Error::TooManyFrames);
        }

        let image = &frames[0];
        let width = image.width();
        let height = image.height();

        let mipmap_count = if mipmaps {
            compute_mipmap_count(width, height)
        } else {
            1
        };

        let mut final_flags = flags.map_or(DEFAULT_FLAGS, |f| f.bits());
        if mipmaps {
            final_flags &= !TextureFlags::NO_MIP.bits();
        } else {
            final_flags |= TextureFlags::NO_MIP.bits();
        }

        let header = VTFHeader {
            signature: VTFHeader::SIGNATURE,
            version: [7, 1], // simpler version without resources for now
            header_size: 64,
            width: width as u16,
            height: height as u16,
            flags: final_flags,
            frames: frames.len() as u16,
            first_frame,
            reflectivity: [0.0, 0.0, 0.0],
            bumpmap_scale,
            highres_image_format: image_format,
            mipmap_count,
            lowres_image_format: ImageFormat::Dxt1, // always the case
            lowres_image_width: 0,                  // no lowres for now
            lowres_image_height: 0,
            depth: 1,
            resources: ResourceList::empty(),
        };

        // Calculate total image data size across all mip levels and frames
        let mut total_image_size: usize = 0;
        for mip in 0..mipmap_count as u32 {
            let mw = (width >> mip).max(1) as u32;
            let mh = (height >> mip).max(1) as u32;
            let frame_size = image_format.frame_size(mw, mh)? as usize;
            total_image_size += frame_size * frames.len();
        }

        let header_size = header.size();
        let mut data = Vec::with_capacity(header_size + total_image_size);

        header.write(&mut data)?;

        // Pad header to the declared size if needed
        assert!(data.len() <= header_size, "invalid header size");
        data.resize(header_size, 0);

        // VTF stores mipmaps smallest to largest.
        // For each mip level (smallest first):
        //   For each frame (first to last):
        //     Write the image data
        for mip in (0..mipmap_count as u32).rev() {
            let mw = (width >> mip).max(1) as usize;
            let mh = (height >> mip).max(1) as usize;

            for frame_image in frames {
                let mip_image = generate_mip(frame_image, mip);
                let encoded = encode_image_data(&mip_image, image_format, mw, mh)?;
                data.extend_from_slice(&encoded);
            }
        }

        Ok(data)
    }

    pub fn create(image: DynamicImage, image_format: ImageFormat) -> Result<Vec<u8>, Error> {
        let w = image.width();
        let h = image.height();

        if w == 0 || h == 0 || w > u16::MAX as u32 || h > u16::MAX as u32 {
            return Err(Error::InvalidImageSize);
        }

        if image_format.is_block_compressed() {
            // block-compressed formats need dimensions that are multiples of 4
            if w % 4 != 0 || h % 4 != 0 {
                return Err(Error::InvalidImageSize);
            }
        }

        Self::encode(&[image], image_format, 0, false, None, 1.0)
    }
}
