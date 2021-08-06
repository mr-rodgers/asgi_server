use pyo3::{types::PyDict, PyResult};

pub trait HttpScopeProvider {
    fn add_scope(&self, parts: &http::request::Parts, scope_dict: &PyDict) -> PyResult<()>;
}
