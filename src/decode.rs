use std::borrow::Cow;

use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;

use jpegxl_rs::decode::{Data, Metadata, Pixels};
use jpegxl_rs::parallel::threads_runner::ThreadsRunner;
use jpegxl_rs::{decoder_builder, DecodeError};

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
    num_threads: isize,
}

#[pymethods]
impl Decoder {
    #[new]
    #[pyo3(signature = (num_threads = -1))]
    fn new(num_threads: isize) -> Self {
        Self { num_threads }
    }

    #[pyo3(signature = (data))]
    fn __call__(
        &self,
        _py: Python,
        data: &[u8],
    ) -> PyResult<(bool, ImageInfo, Cow<'_, [u8]>, Cow<'_, [u8]>)> {
        _py.allow_threads(|| self.call_inner(data))
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("Decoder"))
    }
}

impl Decoder {
    fn call_inner(&self, data: &[u8]) -> PyResult<(bool, ImageInfo, Cow<'_, [u8]>, Cow<'_, [u8]>)> {
        let parallel_runner = ThreadsRunner::new(
            None,
            if self.num_threads < 0 {
                None
            } else {
                Some(self.num_threads as usize)
            },
        )
        .ok_or_else(|| PyRuntimeError::new_err("Could not create JxlThreadsRunner"))?;
        let decoder = decoder_builder()
            .icc_profile(true)
            .parallel_runner(&parallel_runner)
            .build()
            .map_err(to_pyjxlerror)?;
        let (info, img) = decoder.reconstruct(&data).map_err(to_pyjxlerror)?;
        let (jpeg, img) = match img {
            Data::Jpeg(x) => (true, x),
            Data::Pixels(x) => (false, convert_pixels(x)),
        };
        let icc_profile: Vec<u8> = match &info.icc_profile {
            Some(x) => x.to_vec(),
            None => Vec::new(),
        };
        Ok((
            jpeg,
            ImageInfo::from(info),
            Cow::Owned(img),
            Cow::Owned(icc_profile),
        ))
    }
}

fn to_pyjxlerror(e: DecodeError) -> PyErr {
    PyRuntimeError::new_err(e.to_string())
}
