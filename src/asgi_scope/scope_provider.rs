use pyo3::{types::PyDict, Py, PyResult, Python};

use crate::helpers::TryIntoPyDict;

pub trait ScopeProvider {
    fn add_scope(&self, scope_dict: &PyDict) -> PyResult<()>;
}

impl<T> TryIntoPyDict for T
where
    T: ScopeProvider,
{
    fn try_into_py_dict(self, py: Python) -> PyResult<Py<PyDict>> {
        let scope_dict = PyDict::new(py);
        self.add_scope(scope_dict)?;
        Ok(scope_dict.into())
    }
}
