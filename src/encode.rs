use std::borrow::Cow;

use pyo3::prelude::*;

use jpegxl_rs::encode::{ColorEncoding, EncoderFrame, EncoderResult, EncoderSpeed, Metadata};
use jpegxl_rs::encoder_builder;
use jpegxl_rs::parallel::threads_runner::ThreadsRunner;

#[pyclass(module = "pillow_jxl")]
pub struct Encoder {
    num_channels: u32,
    has_alpha: bool,
    lossless: bool,
    quality: f32,
    decoding_speed: i64,
    effort: u32,
    use_container: bool,
    use_original_profile: bool,
    num_threads: isize,
}

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
    ) -> Self {
        let (num_channels, has_alpha) = match mode {
            "RGBA" => (4, true),
            "RGB" => (3, false),
            "LA" => (2, true),
            "L" => (1, false),
            _ => panic!("Only RGB, RGBA, L, LA are supported."),
        };

        let decoding_speed = match decoding_speed {
            0..=4 => decoding_speed,
            _ => panic!("Decoding speed must be between 0 and 4"),
        };

        let use_original_profile = match lossless {
            true => true,
            false => use_original_profile,
        };

        Self {
            num_channels,
            has_alpha,
            lossless,
            quality,
            decoding_speed,
            effort,
            use_container,
            use_original_profile,
            num_threads,
        }
    }

    #[pyo3(signature = (data, width, height, jpeg_encode, exif=None, jumb=None, xmp=None))]
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
    ) -> Cow<'_, [u8]> {
        py.allow_threads(|| self.call_inner(data, width, height, jpeg_encode, exif, jumb, xmp))
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!(
            "Encoder(has_alpha={}, lossless={}, quality={}, decoding_speed={}, effort={}, num_threads={})",
            self.has_alpha, self.lossless, self.quality, self.decoding_speed, self.effort, self.num_threads
        ))
    }
}

impl Encoder {
    fn call_inner(
        &self,
        data: &[u8],
        width: u32,
        height: u32,
        jpeg_encode: bool,
        exif: Option<&[u8]>,
        jumb: Option<&[u8]>,
        xmp: Option<&[u8]>,
    ) -> Cow<'_, [u8]> {
        let parallel_runner = ThreadsRunner::new(
            None,
            if self.num_threads < 0 {
                None
            } else {
                Some(self.num_threads as usize)
            },
        ).unwrap();
        let mut encoder = encoder_builder()
            .parallel_runner(&parallel_runner)
            .jpeg_quality(self.quality)
            .has_alpha(self.has_alpha)
            .lossless(self.lossless)
            .use_container(self.use_container)
            .decoding_speed(self.decoding_speed)
            .build()
            .unwrap();
        encoder.uses_original_profile = self.use_original_profile;
        encoder.color_encoding = match self.num_channels {
            1 | 2 => ColorEncoding::SrgbLuma,
            3 | 4 => ColorEncoding::Srgb,
            _ => panic!("Invalid num channels"),
        };
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
            _ => panic!("Invalid effort"),
        };
        let buffer: EncoderResult<u8> = match jpeg_encode {
            true => encoder.encode_jpeg(&data).unwrap(),
            false => {
                let frame = EncoderFrame::new(data).num_channels(self.num_channels);
                if let Some(exif_data) = exif {
                    encoder
                        .add_metadata(&Metadata::Exif(exif_data), true)
                        .unwrap();
                }
                if let Some(xmp_data) = xmp {
                    encoder
                        .add_metadata(&Metadata::Xmp(xmp_data), true)
                        .unwrap();
                }
                if let Some(jumb_data) = jumb {
                    encoder
                        .add_metadata(&Metadata::Jumb(jumb_data), true)
                        .unwrap();
                }
                encoder.encode_frame(&frame, width, height).unwrap()
            }
        };
        Cow::Owned(buffer.data)
    }
}
