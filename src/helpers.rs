use pyo3::{types::PyDict, Py, PyResult, Python};

pub trait TryIntoPyDict {
    fn try_into_py_dict(self, py: Python) -> PyResult<Py<PyDict>>;
}
