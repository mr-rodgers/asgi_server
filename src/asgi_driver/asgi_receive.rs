use std::sync::Arc;

use futures::{channel::mpsc::Receiver, future, lock::Mutex, StreamExt};
use pyo3::{exceptions::*, prelude::*, types::*};

/// A python callable that wraps a Rust asynchronous stream. It's meant to
/// be passed as `receive` into an ASGI 3.0 context
///
/// `AsgiReceive` wraps a Rust mpsc::Receive<Py<PyDict>> in a python callable
/// returns an asyncio future that resolves to an ASGI event (dict).
/// The messages are yielded from the underlying Receiver, which can be
/// fed using its bound Sender.
#[pyclass]
pub struct AsgiReceive {
    receiver: Arc<Mutex<Receiver<Py<PyDict>>>>,
}

#[pymethods]
impl AsgiReceive {
    #[call]
    fn __call__(&self, py: Python) -> PyResult<PyObject> {
        let receiver = self.receiver.clone();
        pyo3_asyncio::tokio::into_coroutine(py, async move {
            let mut receiver = receiver.lock().await;
            let message = receiver.next().await;
            if let Some(m) = message {
                let event: PyObject = m.into();
                Ok(event)
            } else {
                future::pending::<()>().await;
                Err(PyValueError::new_err("ASGI driver ran out of messages"))
            }
        })
    }
}

impl AsgiReceive {
    /// Create a new `AsgiReceiver` from an owned Receiver
    pub fn new(receiver: Receiver<Py<PyDict>>, py: Python) -> PyResult<Py<AsgiReceive>> {
        Py::new(
            py,
            AsgiReceive {
                receiver: Arc::new(Mutex::new(receiver)),
            },
        )
    }
}
