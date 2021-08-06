mod asgi_driver;
mod asgi_message;
mod asgi_scope;
mod error;
mod helpers;
mod http;
mod server;

use pyo3::prelude::*;
use pyo3::wrap_pyfunction;
use server::Settings;

use crate::asgi_driver::*;

#[pyfunction]
fn start_server(py: Python, asgi_app: Py<PyAny>, settings: Py<Settings>) -> PyResult<PyObject> {
    let driver = AsgiDriver::new(asgi_app);
    let py_none = py.None();
    pyo3_asyncio::tokio::into_coroutine(py, async {
        server::start_http_server(driver, settings).await;
        Ok(py_none)
    })
}

/// A Python module implemented in Rust.
#[pymodule(asgi_server)]
fn asgi_server(py: Python, m: &PyModule) -> PyResult<()> {
    pyo3_asyncio::try_init(py)?;
    // Tokio needs explicit initialization before any pyo3-asyncio conversions.
    // The module import is a prime place to do this.
    pyo3_asyncio::tokio::init_multi_thread_once();

    m.add_function(wrap_pyfunction!(start_server, m)?)?;
    m.add_class::<server::Settings>()?;

    Ok(())
}
