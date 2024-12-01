use pyo3::{create_exception, exceptions::PyRuntimeError, prelude::*};

// it works even if the item is not documented:
mod decode;
mod encode;

create_exception!(my_module, JxlException, PyRuntimeError, "Jxl Error");

#[pymodule]
#[pyo3(name = "pillow_jxl")]
fn pillow_jxl(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<decode::Decoder>()?;
    m.add_class::<encode::Encoder>()?;
    m.add("JxlException", m.py().get_type::<JxlException>())?;
    Ok(())
}
