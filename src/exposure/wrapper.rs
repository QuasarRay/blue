//! Blue decoration policy around PyO3-owned Python references and descriptors.
#![allow(non_snake_case)]

use pyo3::exceptions::{PyAttributeError, PyTypeError};
use pyo3::gc::{PyTraverseError, PyVisit};
use pyo3::prelude::*;
use pyo3::types::{PyDict, PySequence};
use std::sync::{Arc, Mutex};

/// The Blue-specific decorator namespace. PyO3 supplies reference ownership,
/// GC visits, ordinary object access and descriptor binding.
pub struct PythonKlass {
    mKlass: Py<PyAny>,
    mVars: Py<PyDict>,
}

impl PythonKlass {
    pub fn new(py: Python<'_>, klass: Py<PyAny>) -> PyResult<Self> {
        let vars = PyDict::new(py);
        for name in ["__persistvars__", "__nonpersistvars__"] {
            let seq = klass
                .bind(py)
                .getattr(name)
                .map_err(|_| PyTypeError::new_err(format!("{name} undeclared or not sequence.")))?;
            let seq = seq
                .cast::<PySequence>()
                .map_err(|_| PyTypeError::new_err(format!("{name} undeclared or not sequence.")))?;
            for i in 0..seq.len()? {
                vars.set_item(seq.get_item(i)?, py.None())?;
            }
        }
        Ok(Self {
            mKlass: klass,
            mVars: vars.unbind(),
        })
    }

    pub fn GetKlass(&self, py: Python<'_>) -> Py<PyAny> {
        self.mKlass.clone_ref(py)
    }

    pub fn GetAttr(
        &self,
        py: Python<'_>,
        name: &str,
        target: &Bound<'_, PyAny>,
    ) -> PyResult<Option<Py<PyAny>>> {
        if name == "__dict__" {
            return Ok(Some(self.mVars.clone_ref(py).into_any()));
        }
        if name == "__class__" {
            return Ok(Some(self.GetKlass(py)));
        }
        if let Some(value) = self.mVars.bind(py).get_item(name)? {
            return Ok(Some(value.unbind()));
        }
        match self.mKlass.bind(py).getattr(name) {
            Ok(value) if value.is_callable() => Ok(Some(
                py.import("types")?
                    .getattr("MethodType")?
                    .call1((value, target))?
                    .unbind(),
            )),
            Ok(value) => Ok(Some(value.unbind())),
            // Source clears attribute lookup errors and falls back to Blue.
            Err(_) => Ok(None),
        }
    }

    pub fn SetAttr(&self, py: Python<'_>, name: &str, value: &Bound<'_, PyAny>) -> PyResult<()> {
        self.mVars.bind(py).set_item(name, value)
    }

    pub fn Persist(&self, py: Python<'_>) -> PyResult<Py<PyDict>> {
        let result = PyDict::new(py);
        let names = self.mKlass.bind(py).getattr("__persistvars__")?;
        for name in names.try_iter()? {
            let name = name?;
            if let Some(value) = self.mVars.bind(py).get_item(&name)? {
                result.set_item(name, value)?;
            }
        }
        Ok(result.unbind())
    }
}

/// Canonical wrapper policy. Generic Python object layout and binding machinery
/// are supplied by PyO3; the rest of BlueWrapper's source inventory is pending.
#[pyclass(module = "blue", subclass, weakref, frozen)]
pub struct BlueWrapper {
    mPythonKlass: Mutex<Option<Arc<PythonKlass>>>,
    class_name: &'static str,
    native_members: &'static [&'static str],
}

impl BlueWrapper {
    pub(crate) fn new(class_name: &'static str, native_members: &'static [&'static str]) -> Self {
        Self {
            mPythonKlass: Mutex::new(None),
            class_name,
            native_members,
        }
    }
    fn klass(&self, py: Python<'_>) -> Option<Arc<PythonKlass>> {
        py.detach(|| {
            self.mPythonKlass
                .lock()
                .expect("Blue decoration poisoned")
                .clone()
        })
    }
    fn PyGetAttr(slf: &Bound<'_, Self>, name: &str) -> PyResult<Py<PyAny>> {
        let py = slf.py();
        let parent = name.starts_with("_parent_");
        let name = name.strip_prefix("_parent_").unwrap_or(name);
        let klass = slf.get().klass(py);
        if !parent {
            if let Some(klass) = &klass {
                if let Some(value) = klass.GetAttr(py, name, slf.as_any())? {
                    return Ok(value);
                }
            }
        }
        match name {
            "__klass__" => {
                return klass
                    .map(|k| k.GetKlass(py))
                    .ok_or_else(|| PyAttributeError::new_err(name.to_owned()));
            }
            "__bluetype__" => {
                return slf
                    .get()
                    .class_name
                    .into_pyobject(py)
                    .map(|v| v.unbind().into_any())
                    .map_err(Into::into);
            }
            "__typename__" => {
                return slf
                    .get()
                    .class_name
                    .rsplit('.')
                    .next()
                    .unwrap()
                    .into_pyobject(py)
                    .map(|v| v.unbind().into_any())
                    .map_err(Into::into);
            }
            "__dict__" => {
                let dict = PyDict::new(py);
                for name in ["__doc__", "__typename__", "__bluetype__", "__iroot__"]
                    .into_iter()
                    .chain(slf.get().native_members.iter().copied())
                {
                    dict.set_item(name, py.None())?;
                }
                return Ok(dict.into_any().unbind());
            }
            _ => {}
        }
        py.import("builtins")?
            .getattr("object")?
            .getattr("__getattribute__")?
            .call1((slf, name))
            .map(Bound::unbind)
    }

    fn PySetAttr(slf: &Bound<'_, Self>, name: &str, value: &Bound<'_, PyAny>) -> PyResult<()> {
        let py = slf.py();
        if name == "__klass__" {
            let replacement = Arc::new(PythonKlass::new(py, value.clone().unbind())?);
            let wrapper = slf.get();
            let old = py.detach(|| {
                wrapper
                    .mPythonKlass
                    .lock()
                    .expect("Blue decoration poisoned")
                    .replace(replacement)
            });
            drop(old); // A Python destructor may reenter Blue. No lock is held.
            return Ok(());
        }
        let parent = name.starts_with("_parent_");
        let name = name.strip_prefix("_parent_").unwrap_or(name);
        let name = name.strip_prefix("_nosetattr_").unwrap_or(name);
        // Reuse Python descriptors for the actual native property implementation.
        for base in slf.get_type().getattr("__mro__")?.try_iter()? {
            if let Some(descriptor) = base?
                .getattr("__dict__")?
                .call_method1("get", (name,))?
                .extract::<Option<Py<PyAny>>>()?
            {
                if descriptor.bind(py).hasattr("__set__")? {
                    descriptor.bind(py).call_method1("__set__", (slf, value))?;
                    return Ok(());
                }
            }
        }
        if !parent {
            if let Some(klass) = slf.get().klass(py) {
                return klass.SetAttr(py, name, value);
            }
        }
        Err(PyAttributeError::new_err(format!(
            "{} has no writable attribute {name}",
            slf.get().class_name
        )))
    }
}

#[pymethods]
impl BlueWrapper {
    fn __getattribute__(slf: &Bound<'_, Self>, name: &str) -> PyResult<Py<PyAny>> {
        Self::PyGetAttr(slf, name)
    }
    fn __setattr__(slf: &Bound<'_, Self>, name: &str, value: &Bound<'_, PyAny>) -> PyResult<()> {
        Self::PySetAttr(slf, name, value)
    }
    fn __traverse__(&self, visit: PyVisit<'_>) -> Result<(), PyTraverseError> {
        if let Some(klass) = self
            .mPythonKlass
            .lock()
            .expect("Blue decoration poisoned")
            .as_ref()
        {
            visit.call(&klass.mKlass)?;
            visit.call(&klass.mVars)?;
        }
        Ok(())
    }
    fn __clear__(&self) {
        let old = self
            .mPythonKlass
            .lock()
            .expect("Blue decoration poisoned")
            .take();
        drop(old);
    }
}
