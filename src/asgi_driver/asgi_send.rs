use futures::{channel::mpsc::Sender, SinkExt};
use pyo3::{exceptions::*, prelude::*, types::*};

/// A python callable that wraps a Rust asynchronous sink. It's meant to
/// be passed as `send` into an ASGI 3.0 context
///
/// `AsgiSend` wraps a Rust mpsc::Sender<Py<PyDict>> in a python callable
/// that accepts an asgi event (dict) and dispatches it to Rust.
/// The messages are passed into the underlying Sender, which are then
/// receivable on its bound Receiver.
#[pyclass]
pub struct AsgiSend {
    sender: Sender<Py<PyDict>>,
}

#[pymethods]
impl AsgiSend {
    #[call]
    fn __call__(&self, py: Python, dict: &PyDict) -> PyResult<Py<PyAny>> {
        let mut clone = self.sender.clone();
        let dict: Py<PyDict> = dict.into();
        let py_none = py.None();

        pyo3_asyncio::tokio::into_coroutine(py, async move {
            let result = clone.send(dict).await;
            result.map_err(|err| PyValueError::new_err(err.to_string()))?;
            Ok(py_none)
        })
    }
}

impl AsgiSend {
    /// Create a new `AsgiSend` from an owned Sender
    pub fn new(sender: Sender<Py<PyDict>>, py: Python) -> PyResult<Py<AsgiSend>> {
        Py::new(py, AsgiSend { sender })
    }
}
