use pyo3::prelude::*;
use pyo3::types::PyBytes;

use jpegxl_rs::decode::{Metadata, Pixels};
use jpegxl_rs::encode::EncoderResult;
use jpegxl_rs::parallel::threads_runner::ThreadsRunner;
use jpegxl_rs::{decoder_builder, encoder_builder};
// it works even if the item is not documented:

#[pyclass(module = "pillow_jxl")]
struct Encoder {
    parallel: bool,
    has_alpha: bool,
    lossless: bool,
    quality: f32,
    use_container: bool,
    use_original_profile: bool,
    decoding_speed: i64,
}

#[pymethods]
impl Encoder {
    #[new]
    #[pyo3(signature = (mode, parallel=true, lossless=false, quality=1.0, decoding_speed=0, use_container=true, use_original_profile=false))]
    fn new(
        mode: &str,
        parallel: bool,
        lossless: bool,
        quality: f32,
        decoding_speed: i64,
        use_container: bool,
        use_original_profile: bool,
    ) -> Self {
        Self {
            parallel: parallel,
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
            use_container: use_container,
            use_original_profile: match lossless {
                true => true,
                false => use_original_profile,
            },
        }
    }

    #[pyo3(signature = (data, width, height))]
    fn __call__<'a>(&'a self, _py: Python<'a>, data: &[u8], width: u32, height: u32) -> &PyBytes {
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
        let buffer: EncoderResult<u8> = encoder.encode(&data, width, height).unwrap();
        PyBytes::new(_py, &buffer.data)
    }
}

#[pyclass(module = "pillow_jxl")]
struct ImageInfo {
    #[pyo3(get, set)]
    mode: String, // Mode of the image
    #[pyo3(get, set)]
    width: u32, // Width of the image
    #[pyo3(get, set)]
    height: u32, // Height of the image
    #[pyo3(get, set)]
    num_channels: u32, // Number of color channels per pixel
    #[pyo3(get, set)]
    has_alpha_channel: bool,
}

impl ImageInfo {
    fn from(item: Metadata) -> ImageInfo {
        ImageInfo {
            mode: Self::mode(item.num_color_channels, item.has_alpha_channel),
            width: item.width,
            height: item.height,
            num_channels: item.num_color_channels,
            has_alpha_channel: item.has_alpha_channel,
        }
    }

    fn mode(num_channels: u32, has_alpha_channel: bool) -> String {
        match (num_channels, has_alpha_channel) {
            (1, false) => "L".to_string(),
            (1, true) => "LA".to_string(),
            (3, false) => "RGB".to_string(),
            (3, true) => "RGBA".to_string(),
            _ => panic!("Unsupported number of channels"),
        }
    }
}

#[pyclass(module = "pillow_jxl")]
struct Decoder {
    parallel: bool,
}

#[pymethods]
impl Decoder {
    #[new]
    #[pyo3(signature = (parallel=true))]
    fn new(parallel: bool) -> Self {
        Self { parallel: parallel }
    }

    #[pyo3(signature = (data))]
    fn __call__<'a>(&'a self, _py: Python<'a>, data: &[u8]) -> (ImageInfo, &PyBytes) {
        let parallel_runner: ThreadsRunner;
        let decoder = match self.parallel {
            true => {
                parallel_runner = ThreadsRunner::default();
                decoder_builder()
                    .parallel_runner(&parallel_runner)
                    .build()
                    .unwrap()
            }
            false => decoder_builder().build().unwrap(),
        };
        let (info, img) = decoder.decode(&data).unwrap();
        let img: Vec<u8> = match img {
            Pixels::Uint8(x) => x,
            _ => panic!("Unsupported dtype for decoding"),
        };
        (ImageInfo::from(info), PyBytes::new(_py, &img))
    }
}

#[pymodule]
#[pyo3(name = "pillow_jxl")]
fn pillow_jxl(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Encoder>()?;
    m.add_class::<Decoder>()?;
    Ok(())
}
