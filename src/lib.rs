use pyo3::prelude::*;

// it works even if the item is not documented:
mod decode;
mod encode;

#[pymodule]
#[pyo3(name = "pillow_jxl")]
fn pillow_jxl(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<decode::Decoder>()?;
    m.add_class::<encode::Encoder>()?;
    Ok(())
}
