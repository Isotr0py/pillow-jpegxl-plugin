use std::borrow::Cow;

use pyo3::prelude::*;

use jpegxl_rs::encode::{ColorEncoding, EncoderFrame, EncoderResult, EncoderSpeed, Metadata};
use jpegxl_rs::encoder_builder;
use jpegxl_rs::parallel::threads_runner::ThreadsRunner;

#[pyclass(module = "pillow_jxl")]
pub struct Encoder {
    parallel: bool,
    num_channels: u32,
    has_alpha: bool,
    lossless: bool,
    quality: f32,
    decoding_speed: i64,
    effort: u32,
    use_container: bool,
    use_original_profile: bool,
}

#[pymethods]
impl Encoder {
    #[new]
    #[pyo3(signature = (mode, parallel=true, lossless=false, quality=1.0, decoding_speed=0, effort=7, use_container=true, use_original_profile=false))]
    fn new(
        mode: &str,
        parallel: bool,
        lossless: bool,
        quality: f32,
        decoding_speed: i64,
        effort: u32,
        use_container: bool,
        use_original_profile: bool,
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
            parallel,
            num_channels,
            has_alpha,
            lossless,
            quality,
            decoding_speed,
            effort,
            use_container,
            use_original_profile,
        }
    }

    #[pyo3(signature = (data, width, height, jpeg_encode, exif=None, jumb=None, xmp=None))]
    fn __call__(
        &self,
        _py: Python,
        data: &[u8],
        width: u32,
        height: u32,
        jpeg_encode: bool,
        exif: Option<&[u8]>,
        jumb: Option<&[u8]>,
        xmp: Option<&[u8]>,
    ) -> Cow<'_, [u8]> {
        let parallel_runner: ThreadsRunner;
        let mut encoder_builder = encoder_builder();
        let mut encoder = match self.parallel {
            true => {
                parallel_runner = ThreadsRunner::default();
                encoder_builder.set_jpeg_quality(self.quality);
                encoder_builder
                    .parallel_runner(&parallel_runner)
                    .build()
                    .unwrap()
            }
            false => {
                encoder_builder.set_jpeg_quality(self.quality);
                encoder_builder.build().unwrap()
            }
        };
        encoder.uses_original_profile = self.use_original_profile;
        encoder.has_alpha = self.has_alpha;
        encoder.lossless = self.lossless;
        encoder.use_container = self.use_container;
        encoder.decoding_speed = self.decoding_speed;
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

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!(
            "Encoder(parallel={}, has_alpha={}, lossless={}, quality={}, decoding_speed={}, effort={})",
            self.parallel, self.has_alpha, self.lossless, self.quality, self.decoding_speed, self.effort
        ))
    }
}
