mod asgi_receive;
mod asgi_send;

use futures::channel::mpsc;
use pyo3::{
    prelude::{Py, PyObject, PyResult, Python},
    types::*,
};

use asgi_receive::AsgiReceive;
use asgi_send::AsgiSend;

/// Rust handle for a Python asgi app.
///
/// `AsgiDriver` is used to orchestrate running an ASGI 3.0 app from Rust
/// code. It can be used by several reactors to spawn ASGI contexts which
/// represent a single asynchronous call into the application, using
/// `create_context()`
#[derive(Clone)]
pub struct AsgiDriver {
    asgi_app: PyObject,
}

impl AsgiDriver {
    /// Create a new instance of `AsgiDriver`
    ///
    /// `asgi_app` is required to be a ASGI 3.0 callable:
    /// `app(scope, receive, send) -> Awaitable[None]`, however the constructor
    /// does not type check this. If this constraint fails, then an exception
    /// will be thrown into Python when `create_context` is used.
    pub fn new(asgi_app: PyObject) -> AsgiDriver {
        AsgiDriver { asgi_app: asgi_app }
    }

    /// Start an ASGI 3.0 context, providing an ASGI scope, a stream of messages
    /// to send to Python as well as a sink which messages received from Python are
    /// put into.
    ///
    /// A context is created by calling the wrapped ASGI 3.0 application with
    /// a scope, and two coroutine functions: receive and send respectively.
    /// (See the ASGI spec for more details).
    ///
    /// The return value is a future that wraps the completion of the Python
    /// coroutine.
    pub fn create_context(
        &self,
        scope: &PyDict,
        messages_to_py: mpsc::Receiver<Py<PyDict>>,
        results_from_py: mpsc::Sender<Py<PyDict>>,
    ) -> PyResult<impl futures::Future<Output = PyResult<PyObject>> + Send> {
        let result = Python::with_gil(|py| {
            let receive = AsgiReceive::new(messages_to_py, py)?;
            let send = AsgiSend::new(results_from_py, py)?;

            AsgiDriver::ensure_asgi_field(scope, py)?;
            let coro = self.asgi_app.call1(py, (scope, receive, send))?;
            pyo3_asyncio::into_future(coro.as_ref(py))
        });
        result
    }

    fn ensure_asgi_field(dict: &PyDict, py: Python) -> PyResult<()> {
        if !dict.contains("asgi")? {
            dict.set_item("asgi", PyDict::new(py))?;
        }

        let asgi_meta: &PyDict = dict.get_item("asgi").unwrap().downcast()?;
        asgi_meta.set_item("version", "3.0")?;

        Ok(())
    }
}
