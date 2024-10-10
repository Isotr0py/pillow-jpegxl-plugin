use std::borrow::Cow;
use std::u8;

use pyo3::exceptions::{PyRuntimeError, PyValueError};
use pyo3::prelude::*;

use jpegxl_rs::decode::{Data, Metadata, Pixels};
use jpegxl_rs::parallel::threads_runner::ThreadsRunner;
use jpegxl_rs::{decoder_builder, DecodeError};

use bytemuck::*;

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
    fn from(item: &Metadata, mode: String) -> ImageInfo {
        ImageInfo {
            mode: mode,
            width: item.width,
            height: item.height,
            num_channels: item.num_color_channels,
            has_alpha_channel: item.has_alpha_channel,
        }
    }
}

fn mode_8_bits(info: &Metadata, pixel_format: &'static str) -> PyResult<&'static str> {
    let mode = match (info.num_color_channels, info.has_alpha_channel) {
        (3, false) => "RGB",
        (3, true) => "RGBA",
        (1, false) => "L",
        (1, true) => "LA",
        (channels, has_alpha) => {
            return Err(PyValueError::new_err(format!(
            "Unsupported number of channels for {pixel_format}: {channels}, has_alpha: {has_alpha}"
        )))
        }
    };
    Ok(mode)
}

pub fn convert_pixels(pixels: Pixels, info: &Metadata) -> PyResult<(Vec<u8>, &'static str)> {
    let mut result = Vec::new();
    let mode = match (pixels, info.num_color_channels, info.has_alpha_channel) {
        (Pixels::Uint8(pixels), _, _) => {
            // 8 bits RGB(A) and L(A)
            result.extend_from_slice(&pixels);
            mode_8_bits(info, "Uint8")
        }
        (Pixels::Uint16(pixels), 1, false) => {
            // 16 bits: I;16
            result.extend_from_slice(
                try_cast_slice(&pixels).map_err(|e| PyValueError::new_err(e.to_string()))?,
            );
            Ok("I;16")
        }
        (Pixels::Uint16(pixels), _, _) => {
            // RGB(A) and LA must be converted to 8 bits
            result.reserve(pixels.len());
            result.extend(pixels.into_iter().map(|pixel| (pixel >> 8) as u8));
            mode_8_bits(info, "Uint16")
        }
        (Pixels::Float(pixels), 1, false) => {
            // 32 bits: F
            result.extend_from_slice(
                try_cast_slice(&pixels).map_err(|e| PyValueError::new_err(e.to_string()))?,
            );
            Ok("F")
        }
        (Pixels::Float(pixels), _, _) => {
            // RGB(A) and LA must be converted to 8 bits
            result.reserve(pixels.len());
            result.extend(pixels.into_iter().map(|pixel| (pixel * 255.0) as u8));
            mode_8_bits(info, "Float")
        }
        (Pixels::Float16(pixels), 1, false) => {
            // Convert to f32 (F)
            result.reserve(pixels.len() * 4);
            for pixel in pixels {
                result.extend_from_slice(&f32::from(pixel).to_ne_bytes());
            }
            Ok("F")
        }
        (Pixels::Float16(pixels), _, _) => {
            // RGB(A) and LA must be converted to 8 bits
            result.reserve(pixels.len());
            result.extend(
                pixels
                    .into_iter()
                    .map(|pixel| (f32::from(pixel) * 255.0) as u8),
            );
            mode_8_bits(info, "Float16")
        }
    }?;
    Ok((result, mode))
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
        let (metadata, img) = decoder.reconstruct(&data).map_err(to_pyjxlerror)?;
        let (jpeg, (img, mode)) = match img {
            Data::Jpeg(x) => (true, (x, "cf_jpeg")),
            Data::Pixels(x) => (false, convert_pixels(x, &metadata)?),
        };
        let info = ImageInfo::from(&metadata, mode.to_string());
        let icc_profile = metadata.icc_profile.unwrap_or_else(|| Vec::new());
        Ok((jpeg, info, Cow::Owned(img), Cow::Owned(icc_profile)))
    }
}

fn to_pyjxlerror(e: DecodeError) -> PyErr {
    PyRuntimeError::new_err(e.to_string())
}
