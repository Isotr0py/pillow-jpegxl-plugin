use std::borrow::Cow;

use pyo3::exceptions::{PyNotImplementedError, PyRuntimeError, PyValueError};
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
    fn from(item: Metadata, pixel: &Data) -> ImageInfo {
        let pixel_type = match &pixel {
            Data::Pixels(pixels) => Some(pixels),
            Data::Jpeg(_) => None,
        };
        ImageInfo {
            mode: Self::mode(item.num_color_channels, item.has_alpha_channel, pixel_type).unwrap(),
            width: item.width,
            height: item.height,
            num_channels: item.num_color_channels,
            has_alpha_channel: item.has_alpha_channel,
        }
    }

    fn mode(
        num_channels: u32,
        has_alpha_channel: bool,
        pixel_type: Option<&Pixels>,
    ) -> PyResult<String> {
        let mode = match (num_channels, has_alpha_channel) {
            (1, false) => "L".to_string(),
            (1, true) => "LA".to_string(),
            (3, false) => "RGB".to_string(),
            (3, true) => "RGBA".to_string(),
            _ => return Err(PyNotImplementedError::new_err("Unsupported color mode")),
        };
        if let Some(Pixels::Uint16(_)) = pixel_type {
            if mode == "L" {
                return Ok("I;16".to_string());
            }
        }
        if let Some(Pixels::Float(_)) = pixel_type {
            if mode == "L" {
                return Ok("F".to_string());
            }
        }
        Ok(mode)
    }
}

#[pyclass(module = "pillow_jxl")]
pub struct JxlBox {
    #[pyo3(get, set)]
    box_type: [u8; 4],
    #[pyo3(get, set)]
    data: Vec<u8>,
}

#[pymethods]
impl JxlBox {
    fn __repr__(&self) -> PyResult<String> {
        Ok(format!(
            "JxlBox(type={:?}, size={})",
            String::from_utf8_lossy(&self.box_type),
            self.data.len()
        ))
    }
}

// refer to https://github.com/Fraetor/jxl_decode/blob/902cd5d479f89f93df6105a22dc92f297ab77541/src/jxl_decode/jxl.py#L88-L110
fn extract_boxes(data: &[u8]) -> PyResult<Vec<JxlBox>> {
    let mut boxes = Vec::new();
    let mut pos = 32;
    while pos + 4 <= data.len() {
        let box_size = u32::from_be_bytes(data[pos..pos + 4].try_into().unwrap()) as usize;
        if pos + box_size > data.len() {
            break;
        }
        if 1 < box_size && box_size <= 8 {
            pos += 4;
            continue;
        }
        let header_length;
        let box_length;
        if box_size == 1 {
            header_length = 16;
            box_length = u32::from_be_bytes(data[pos + 8..pos + 16].try_into().unwrap()) as usize;
            if box_length <= 16 {
                pos += 4;
                continue;
            }
        } else {
            header_length = 8;
            box_length = if box_size == 0 {
                data.len() - pos
            } else {
                box_size
            };
        }
        if pos + header_length + box_length > data.len() {
            break;
        }
        let box_type = data[pos + 4..pos + 8].try_into().unwrap();
        let box_data = data[pos + header_length..pos + header_length + box_length].to_vec();
        boxes.push(JxlBox {
            box_type: box_type,
            data: box_data,
        });
        pos += box_length;
    }
    Ok(boxes)
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
    ) -> PyResult<(bool, ImageInfo, Cow<'_, [u8]>, Cow<'_, [u8]>, Vec<JxlBox>)> {
        _py.detach(|| self.call_inner(data))
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("Decoder"))
    }
}

impl Decoder {
    fn pixels_to_bytes_8bit(&self, pixels: Pixels) -> PyResult<Vec<u8>> {
        // Convert pixels to bytes with 8-bit casting
        let mut result = Vec::new();
        match pixels {
            Pixels::Uint8(pixels) => {
                return Ok(pixels);
            }
            Pixels::Uint16(pixels) => {
                for pixel in pixels {
                    result.push((pixel >> 8) as u8);
                }
            }
            Pixels::Float(pixels) => {
                for pixel in pixels {
                    result.push((pixel * 255.0) as u8);
                }
            }
            Pixels::Float16(_) => {
                return Err(PyNotImplementedError::new_err(
                    "Float16 is not supported yet",
                ))
            }
        }
        Ok(result)
    }

    fn pixels_to_bytes(&self, pixels: Pixels) -> PyResult<Vec<u8>> {
        // Convert pixels to bytes without casting
        let mut result = Vec::new();
        match pixels {
            Pixels::Uint8(pixels) => {
                return Ok(pixels);
            }
            Pixels::Uint16(pixels) => {
                for pixel in pixels {
                    let pix_bytes = pixel.to_ne_bytes();
                    for byte in pix_bytes.iter() {
                        result.push(*byte);
                    }
                }
            }
            Pixels::Float(pixels) => {
                for pixel in pixels {
                    let pix_bytes = pixel.to_ne_bytes();
                    for byte in pix_bytes.iter() {
                        result.push(*byte);
                    }
                }
            }
            Pixels::Float16(_) => {
                return Err(PyNotImplementedError::new_err(
                    "Float16 is not supported yet",
                ))
            }
        }
        Ok(result)
    }

    fn convert_pil_pixels(&self, pixels: Pixels, num_channels: u32) -> PyResult<Vec<u8>> {
        let result = match num_channels {
            1 => self.pixels_to_bytes(pixels)?,
            3 => self.pixels_to_bytes_8bit(pixels)?,
            _ => return Err(PyValueError::new_err("image color channels must be 1 or 3")),
        };
        Ok(result)
    }
}

impl Decoder {
    fn call_inner(
        &self,
        data: &[u8],
    ) -> PyResult<(bool, ImageInfo, Cow<'_, [u8]>, Cow<'_, [u8]>, Vec<JxlBox>)> {
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
        let boxes = extract_boxes(&data)?;
        let icc_profile: Vec<u8> = match &info.icc_profile {
            Some(x) => x.to_vec(),
            None => Vec::new(),
        };
        let img_info = ImageInfo::from(info, &img);
        let (jpeg, img) = match img {
            Data::Jpeg(x) => (true, x),
            Data::Pixels(x) => (false, self.convert_pil_pixels(x, img_info.num_channels)?),
        };
        Ok((
            jpeg,
            img_info,
            Cow::Owned(img),
            Cow::Owned(icc_profile),
            boxes,
        ))
    }
}

fn to_pyjxlerror(e: DecodeError) -> PyErr {
    PyRuntimeError::new_err(e.to_string())
}
