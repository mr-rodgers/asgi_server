use pyo3::{types::PyDict, PyResult, Python};

use super::scope_provider::ScopeProvider;

pub enum Type {
    HTTP,
}

impl ScopeProvider for Type {
    fn add_scope(&self, scope_dict: &PyDict) -> PyResult<()> {
        match self {
            Self::HTTP => scope_dict.set_item("type", "http"),
        }
    }
}

pub struct AsgiVersion;
impl ScopeProvider for AsgiVersion {
    fn add_scope(&self, scope_dict: &PyDict) -> PyResult<()> {
        Python::with_gil(|py| {
            if !scope_dict.contains("asgi")? {
                scope_dict.set_item("asgi", PyDict::new(py))?;
            }

            let asgi_meta: &PyDict = scope_dict.get_item("asgi").unwrap().downcast()?;
            asgi_meta.set_item("version", "3.0")
        })
    }
}
