use std::borrow::Cow;

use pyo3::prelude::*;

use jpegxl_rs::decode::{Data, Metadata, Pixels};
use jpegxl_rs::decoder_builder;
use jpegxl_rs::parallel::threads_runner::ThreadsRunner;

// it works even if the item is not documented:

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

pub fn convert_pixels(pixels: Pixels) -> Vec<u8> {
    let mut result = Vec::new();
    match pixels {
        Pixels::Uint8(pixels) => {
            for pixel in pixels {
                result.push(pixel);
            }
        }
        Pixels::Uint16(pixels) => {
            for pixel in pixels {
                result.push((pixel >> 8) as u8);
                result.push(pixel as u8);
            }
        }
        Pixels::Float(pixels) => {
            for pixel in pixels {
                result.push((pixel * 255.0) as u8);
            }
        }
        Pixels::Float16(_) => panic!("Float16 is not supported yet"),
    }
    result
}

#[pyclass(module = "pillow_jxl")]
pub struct Decoder {
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
    fn __call__(
        &self,
        _py: Python,
        data: &[u8],
    ) -> (bool, ImageInfo, Cow<'_, [u8]>, Cow<'_, [u8]>) {
        let parallel_runner: ThreadsRunner;
        let decoder = match self.parallel {
            true => {
                parallel_runner = ThreadsRunner::default();
                decoder_builder()
                    .icc_profile(true)
                    .parallel_runner(&parallel_runner)
                    .build()
                    .unwrap()
            }
            false => decoder_builder().icc_profile(true).build().unwrap(),
        };
        let (info, img) = decoder.reconstruct(&data).unwrap();
        let (jpeg, img) = match img {
            Data::Jpeg(x) => (true, x),
            Data::Pixels(x) => (false, convert_pixels(x)),
        };
        let icc_profile: Vec<u8> = match &info.icc_profile {
            Some(x) => x.to_vec(),
            None => Vec::new(),
        };
        (
            jpeg,
            ImageInfo::from(info),
            Cow::Owned(img),
            Cow::Owned(icc_profile),
        )
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("Decoder(parallel={})", self.parallel))
    }
}
