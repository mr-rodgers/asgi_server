use std::convert::Infallible;

#[derive(Debug)]
pub struct ServerError(hyper::Error);

impl ServerError {
    pub fn handle<T>(self, fallback: T) -> Result<T, Infallible> {
        let ServerError(hyper_error) = self;

        // FIX ME: log an error
        eprintln!("Internal server error: {}", hyper_error);

        Ok(fallback)
    }
}

impl From<hyper::Error> for ServerError {
    fn from(hyper_error: hyper::Error) -> Self {
        Self(hyper_error)
    }
}

impl From<ServerError> for pyo3::PyErr {
    fn from(error: ServerError) -> pyo3::PyErr {
        let ServerError(hyper_error) = error;
        pyo3::exceptions::PyOSError::new_err(hyper_error.to_string())
    }
}

pub struct ApplicationError(pyo3::PyErr);

impl From<pyo3::PyErr> for ApplicationError {
    fn from(app_err: pyo3::PyErr) -> Self {
        Self(app_err)
    }
}

impl ApplicationError {
    pub fn handle<T>(self, fallback: T) -> Result<T, Infallible> {
        let ApplicationError(py_err) = self;

        // throw the error into Python
        eprintln!("Application error: {}", py_err);

        Ok(fallback)
    }
}
