//! PyO3 supplies binding machinery. These adapters retain Blue's exposed names
//! and its source-specific conversion/return policies.
#![allow(non_snake_case)]

use crate::{BlueRuntime, ICcpStream, MemStream, ObjectHandle, SeekOrigin};
use pyo3::exceptions::{PyImportError, PyNotImplementedError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyModule};
pub(crate) mod events;
pub mod wrapper;
use wrapper::BlueWrapper;

#[pyclass(name = "MemStream", module = "blue", extends=BlueWrapper, frozen)]
struct PyMemStream {
    handle: ObjectHandle<MemStream>,
}

#[pymethods]
impl PyMemStream {
    #[new]
    fn new(py: Python<'_>) -> PyClassInitializer<Self> {
        PyClassInitializer::from(BlueWrapper::new("blue.MemStream", &["size", "pos"])).add_subclass(
            Self {
                handle: py
                    .detach(|| BlueRuntime::shared().create("blue.MemStream", MemStream::new())),
            },
        )
    }
    #[pyo3(signature = (size=-1))]
    fn Read<'py>(&self, py: Python<'py>, size: isize) -> PyResult<Bound<'py, PyBytes>> {
        let data = py.detach(|| {
            self.handle.with(|stream| {
                let available = stream.GetSize() - stream.GetPosition();
                let len = if size < 0 {
                    available
                } else {
                    available.min(size as usize)
                };
                let mut out = vec![0; len];
                stream.Read(&mut out, size)?;
                Ok::<_, std::io::Error>(out)
            })
        })?;
        Ok(PyBytes::new(py, &data))
    }
    #[pyo3(signature = (value, size=-1))]
    fn Write(&self, py: Python<'_>, value: &Bound<'_, PyAny>, size: isize) -> PyResult<()> {
        let buffer = pyo3::buffer::PyUntypedBuffer::get(value)?;
        if !buffer.is_c_contiguous() && !buffer.is_fortran_contiguous() {
            return Err(PyNotImplementedError::new_err(
                "Support for non-contiguous buffers is currently not implemented",
            ));
        }
        let bytes = py
            .import("builtins")?
            .getattr("memoryview")?
            .call1((value,))?
            .call_method1("tobytes", ("A",))?;
        let value: &[u8] = bytes.extract()?;
        // Source writes at least the buffer length. Larger explicit sizes read
        // past the source buffer in C++; native Rust rejects that pointer UB.
        if size > value.len() as isize {
            return Err(PyValueError::new_err("size exceeds the provided buffer"));
        }
        py.detach(|| self.handle.with(|stream| stream.Write(value)))?;
        Ok(())
    }
    #[pyo3(signature = (pos, whence=0))]
    fn Seek(&self, py: Python<'_>, pos: i32, whence: i32) -> PyResult<()> {
        let method = match whence {
            0 => SeekOrigin::SO_BEGIN,
            1 => SeekOrigin::SO_CURRENT,
            _ => SeekOrigin::SO_END,
        };
        py.detach(|| self.handle.with(|stream| stream.Seek(pos as isize, method)))?;
        Ok(())
    }
    #[pyo3(signature = (size=-1))]
    fn read<'py>(&self, py: Python<'py>, size: isize) -> PyResult<Bound<'py, PyBytes>> {
        self.Read(py, size)
    }
    #[pyo3(signature = (value, size=-1))]
    fn write(&self, py: Python<'_>, value: &Bound<'_, PyAny>, size: isize) -> PyResult<()> {
        self.Write(py, value, size)
    }
    #[pyo3(signature = (pos, whence=0))]
    fn seek(&self, py: Python<'_>, pos: i32, whence: i32) -> PyResult<()> {
        self.Seek(py, pos, whence)
    }
    #[getter]
    fn size(&self, py: Python<'_>) -> usize {
        py.detach(|| self.handle.with(|s| s.GetSize()))
    }
    #[getter]
    fn pos(&self, py: Python<'_>) -> usize {
        py.detach(|| self.handle.with(|s| s.GetPosition()))
    }
}

#[pyclass(name = "BlueClasses", module = "blue", frozen)]
struct PyBlueClasses;
#[pymethods]
impl PyBlueClasses {
    fn CreateInstance(&self, py: Python<'_>, name: &str) -> PyResult<Py<PyAny>> {
        crate::BlueClasses::PyCreateInstance(py, name)
    }
    fn LiveCount(&self, py: Python<'_>) -> std::collections::BTreeMap<String, usize> {
        py.detach(|| BlueRuntime::shared().LiveCount())
    }
}

#[pymodule(gil_used = false)]
fn blue(m: &Bound<'_, PyModule>) -> PyResult<()> {
    let py = m.py();
    let configured: Option<i32> = py
        .import("sysconfig")?
        .call_method1("get_config_var", ("Py_GIL_DISABLED",))?
        .extract()?;
    let gil: bool = py
        .import("sys")?
        .call_method0("_is_gil_enabled")?
        .extract()?;
    if configured != Some(1) || gil {
        return Err(PyImportError::new_err(
            "Blue requires free-threaded CPython with the GIL disabled; use tools/build.py and tools/run_original_tests.py",
        ));
    }
    m.add_class::<BlueWrapper>()?;
    m.add_class::<PyMemStream>()?;
    m.add("classes", Py::new(py, PyBlueClasses)?)?;
    events::register(m)?;
    Ok(())
}
