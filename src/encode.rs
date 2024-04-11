use std::collections::HashMap;

use pyo3::prelude::*;
use pyo3::types::PyBytes;

use jpegxl_rs::encode::{ColorEncoding, EncoderFrame, EncoderResult, EncoderSpeed, Metadata as EncoderMetadata};
use jpegxl_rs::parallel::threads_runner::ThreadsRunner;
use jpegxl_rs::encoder_builder;


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
        Self {
            parallel: parallel,
            num_channels: match mode {
                "RGBA" => 4,
                "RGB" => 3,
                "LA" => 2,
                "L" => 1,
                _ => panic!("Only RGB, RGBA, L, LA are supported."),
            },
            has_alpha: match mode {
                "RGBA" | "LA" => true,
                "RGB" | "L" => false,
                _ => panic!("Only RGB, RGBA, L, LA are supported."),
            },
            lossless: lossless,
            quality: quality,
            decoding_speed: match decoding_speed {
                0...4 => decoding_speed,
                _ => panic!("Decoding speed must be between 0 and 4"),
            },
            effort: effort,
            use_container: use_container,
            use_original_profile: match lossless {
                true => true,
                false => use_original_profile,
            },
        }
    }

    #[pyo3(signature = (data, width, height, jpeg_encode, metadata))]
    fn __call__<'a>(
        &'a self,
        _py: Python<'a>,
        data: &[u8],
        width: u32,
        height: u32,
        jpeg_encode: bool,
        metadata: HashMap<String, &[u8]>
    ) -> &PyBytes {
        let parallel_runner: ThreadsRunner;
        let mut encoder = match self.parallel {
            true => {
                parallel_runner = ThreadsRunner::default();
                encoder_builder()
                    .parallel_runner(&parallel_runner)
                    .build()
                    .unwrap()
            }
            false => encoder_builder().build().unwrap(),
        };
        encoder.uses_original_profile = self.use_original_profile;
        encoder.has_alpha = self.has_alpha;
        encoder.lossless = self.lossless;
        encoder.quality = self.quality;
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
                let metadata = EncoderMetadata::new().exif(metadata["exif"]).jumb(metadata["jumb"]).xmp(metadata["xmp"]);
                encoder.encode_frame_with_metadata(&frame, width, height, metadata).unwrap()
            }
        };
        PyBytes::new(_py, &buffer.data)
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!(
            "Encoder(parallel={}, has_alpha={}, lossless={}, quality={}, decoding_speed={}, effort={})",
            self.parallel, self.has_alpha, self.lossless, self.quality, self.decoding_speed, self.effort
        ))
    }
}