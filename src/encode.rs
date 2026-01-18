use std::borrow::Cow;

use pyo3::exceptions::{PyRuntimeError, PyValueError};
use pyo3::prelude::*;

use jpegxl_rs::encode::{ColorEncoding, EncoderFrame, EncoderSpeed, Metadata};
use jpegxl_rs::parallel::threads_runner::ThreadsRunner;
use jpegxl_rs::{encoder_builder, EncodeError};

/// Represents different pixel types for encoding
#[derive(Clone, Copy)]
enum PixelType {
    /// 8-bit unsigned integer (RGB, RGBA, L, LA modes)
    Uint8 { num_channels: u32, has_alpha: bool },
    /// 16-bit unsigned integer (I;16 mode, grayscale only)
    Uint16,
}

impl PixelType {
    fn color_encoding(&self) -> ColorEncoding {
        match self {
            PixelType::Uint8 { num_channels, .. } => match num_channels {
                1 | 2 => ColorEncoding::SrgbLuma,
                3 | 4 => ColorEncoding::Srgb,
                _ => unreachable!("Invalid number of channels for Uint8 pixel type"),
            },
            PixelType::Uint16 => ColorEncoding::LinearSrgbLuma,
        }
    }

    fn has_alpha(&self) -> bool {
        match self {
            PixelType::Uint8 { has_alpha, .. } => *has_alpha,
            PixelType::Uint16 => false,
        }
    }
}

#[pyclass(module = "pillow_jxl")]
pub struct Encoder {
    pixel_type: PixelType,
    lossless: bool,
    quality: f32,
    decoding_speed: i64,
    effort: u32,
    use_container: bool,
    use_original_profile: bool,
    num_threads: isize,
}

#[allow(clippy::too_many_arguments)]
#[pymethods]
impl Encoder {
    #[new]
    #[pyo3(signature = (mode, lossless=false, quality=1.0, decoding_speed=0, effort=7, use_container=false, use_original_profile=false, num_threads=-1))]
    fn new(
        mode: &str,
        lossless: bool,
        quality: f32,
        decoding_speed: i64,
        effort: u32,
        use_container: bool,
        use_original_profile: bool,
        num_threads: isize,
    ) -> PyResult<Self> {
        let pixel_type = match mode {
            "RGBA" => PixelType::Uint8 {
                num_channels: 4,
                has_alpha: true,
            },
            "RGB" => PixelType::Uint8 {
                num_channels: 3,
                has_alpha: false,
            },
            "LA" => PixelType::Uint8 {
                num_channels: 2,
                has_alpha: true,
            },
            "L" => PixelType::Uint8 {
                num_channels: 1,
                has_alpha: false,
            },
            "I;16" => PixelType::Uint16,
            _ => {
                return Err(PyValueError::new_err(
                    "Only RGB, RGBA, L, LA, I;16 are supported.",
                ))
            }
        };

        let decoding_speed = match decoding_speed {
            0..=4 => decoding_speed,
            _ => {
                return Err(PyValueError::new_err(
                    "Decoding speed must be between 0 and 4",
                ))
            }
        };

        let use_original_profile = match lossless {
            true => true,
            false => use_original_profile,
        };

        Ok(Self {
            pixel_type,
            lossless,
            quality,
            decoding_speed,
            effort,
            use_container,
            use_original_profile,
            num_threads,
        })
    }

    #[allow(clippy::too_many_arguments)]
    #[pyo3(signature = (data, width, height, jpeg_encode, exif=None, jumb=None, xmp=None, compress=false))]
    fn __call__(
        &self,
        py: Python,
        data: &[u8],
        width: u32,
        height: u32,
        jpeg_encode: bool,
        exif: Option<&[u8]>,
        jumb: Option<&[u8]>,
        xmp: Option<&[u8]>,
        compress: bool,
    ) -> PyResult<Cow<'_, [u8]>> {
        py.detach(|| self.call_inner(data, width, height, jpeg_encode, exif, jumb, xmp, compress))
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!(
            "Encoder(has_alpha={}, lossless={}, quality={}, decoding_speed={}, effort={}, num_threads={})",
            self.pixel_type.has_alpha(), self.lossless, self.quality, self.decoding_speed, self.effort, self.num_threads
        ))
    }
}

impl Encoder {
    #[allow(clippy::too_many_arguments)]
    fn call_inner(
        &self,
        data: &[u8],
        width: u32,
        height: u32,
        jpeg_encode: bool,
        exif: Option<&[u8]>,
        jumb: Option<&[u8]>,
        xmp: Option<&[u8]>,
        compress: bool,
    ) -> PyResult<Cow<'_, [u8]>> {
        let parallel_runner = ThreadsRunner::new(
            None,
            if self.num_threads < 0 {
                None
            } else {
                Some(self.num_threads as usize)
            },
        )
        .ok_or_else(|| PyRuntimeError::new_err("Could not create JxlThreadsRunner"))?;
        let mut encoder = encoder_builder()
            .parallel_runner(&parallel_runner)
            .jpeg_quality(self.quality)
            .has_alpha(self.pixel_type.has_alpha())
            .lossless(self.lossless)
            .use_container(self.use_container)
            .decoding_speed(self.decoding_speed)
            .build()
            .map_err(to_pyjxlerror)?;
        encoder.uses_original_profile = self.use_original_profile;
        encoder.color_encoding = Some(self.pixel_type.color_encoding());
        encoder.speed = match self.effort {
            1 => EncoderSpeed::Lightning,
            2 => EncoderSpeed::Thunder,
            3 => EncoderSpeed::Falcon,
            4 => EncoderSpeed::Cheetah,
            5 => EncoderSpeed::Hare,
            6 => EncoderSpeed::Wombat,
            7 => EncoderSpeed::Squirrel,
            8 => EncoderSpeed::Kitten,
            9 => EncoderSpeed::Tortoise,
            _ => return Err(PyValueError::new_err("Invalid effort")),
        };

        // Add metadata if provided (for non-JPEG encode)
        if !jpeg_encode {
            if let Some(exif_data) = exif {
                encoder
                    .add_metadata(&Metadata::Exif(exif_data), compress)
                    .map_err(to_pyjxlerror)?
            }
            if let Some(xmp_data) = xmp {
                encoder
                    .add_metadata(&Metadata::Xmp(xmp_data), compress)
                    .map_err(to_pyjxlerror)?
            }
            if let Some(jumb_data) = jumb {
                encoder
                    .add_metadata(&Metadata::Jumb(jumb_data), compress)
                    .map_err(to_pyjxlerror)?
            }
        }

        let buffer: Vec<u8> = if jpeg_encode {
            encoder.encode_jpeg(data).map_err(to_pyjxlerror)?.data
        } else {
            match self.pixel_type {
                PixelType::Uint8 { num_channels, .. } => {
                    let frame = EncoderFrame::new(data).num_channels(num_channels);
                    encoder
                        .encode_frame::<u8, u8>(&frame, width, height)
                        .map_err(to_pyjxlerror)?
                        .data
                }
                PixelType::Uint16 => {
                    let data_u16: &[u16] = bytemuck::try_cast_slice(data).map_err(|e| {
                        PyValueError::new_err(format!("Failed to cast I;16 data to u16 slice: {e}"))
                    })?;
                    let frame = EncoderFrame::new(data_u16).num_channels(1);
                    encoder
                        .encode_frame::<u16, u16>(&frame, width, height)
                        .map_err(to_pyjxlerror)?
                        .data
                }
            }
        };
        Ok(Cow::Owned(buffer))
    }
}

fn to_pyjxlerror(e: EncodeError) -> PyErr {
    PyRuntimeError::new_err(e.to_string())
}
