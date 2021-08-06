use std::convert::TryFrom;

use http::{header::HeaderName, response, HeaderMap, HeaderValue, StatusCode};
use pyo3::{
    exceptions::{PyTypeError, PyValueError},
    types::{PyBytes, PyDict, PyIterator},
    Py, PyAny, PyErr, PyResult, Python,
};

use crate::asgi_message::AsgiMessage;
use crate::error;

pub struct HttpResponseStart {
    status: StatusCode,
    headers: HeaderMap,
}

impl AsgiMessage for HttpResponseStart {
    fn message_type() -> &'static str {
        "http.response.start"
    }
}

impl HttpResponseStart {
    fn get_status_code(message_dict: &PyDict) -> PyResult<StatusCode> {
        let status_code = PyAny::get_item(&message_dict, "status")?.extract::<u16>()?;
        StatusCode::from_u16(status_code).map_err(|status_problem| {
            PyValueError::new_err(format!("Invalid status code: {}", status_problem))
        })
    }

    fn get_headers(message_dict: &PyDict) -> PyResult<HeaderMap> {
        Python::with_gil(|py| {
            let py_headers_iterator =
                PyIterator::from_object(py, PyAny::get_item(&message_dict, "headers")?)?;

            let header_lines = py_headers_iterator.filter_map(HeaderLine::extract);

            let mut header_map = HeaderMap::new();

            for header_line in header_lines {
                let (header_name, header_value) = header_line.into();
                header_map.append(header_name, header_value);
            }

            Ok(header_map)
        })
    }
}

impl TryFrom<Py<PyDict>> for HttpResponseStart {
    type Error = PyErr;

    fn try_from(message_dict: Py<PyDict>) -> Result<Self, Self::Error> {
        Python::with_gil(|py| {
            let message_dict = message_dict.as_ref(py);
            HttpResponseStart::validate_message_type(message_dict)
                .and_then(|_| HttpResponseStart::get_status_code(message_dict))
                .and_then(|status| {
                    HttpResponseStart::get_headers(message_dict)
                        .map(|headers| HttpResponseStart { status, headers })
                })
        })
    }
}

impl From<HttpResponseStart> for response::Builder {
    fn from(start_message: HttpResponseStart) -> Self {
        let HttpResponseStart { status, headers } = start_message;

        let mut builder = response::Builder::new();
        builder
            .headers_mut()
            .map(|header_map| header_map.extend(headers));
        builder.status(status)
    }
}

struct HeaderLine(HeaderName, HeaderValue);

impl HeaderLine {
    fn extract(items: PyResult<&PyAny>) -> Option<Self> {
        match HeaderLine::try_from(items) {
            Err(py_err) => {
                let app_err = error::ApplicationError::from(py_err);
                app_err.handle(()).unwrap();
                None
            }
            Ok(header_line) => Some(header_line),
        }
    }

    fn unwrap_header_line_item(value: Option<PyResult<&PyAny>>) -> PyResult<&PyBytes> {
        match value {
            None => Err(PyValueError::new_err(
                "Unexpected end of header item iterator",
            )),
            Some(Err(py_err)) => Err(py_err),
            Some(Ok(item)) => Ok(item.downcast::<PyBytes>().map_err(|err| {
                PyTypeError::new_err(format!(
                    "Cannot convert header item to bytes: {}: {}",
                    item, &err
                ))
            })?),
        }
    }
}

impl TryFrom<PyResult<&PyAny>> for HeaderLine {
    type Error = PyErr;

    fn try_from(value: PyResult<&PyAny>) -> Result<Self, Self::Error> {
        value.and_then(|items| {
            Python::with_gil(|py| {
                let mut items_iter = PyIterator::from_object(py, items)?;

                let name = HeaderLine::unwrap_header_line_item(items_iter.next())?;
                let value = HeaderLine::unwrap_header_line_item(items_iter.next())?;

                let name = HeaderName::from_bytes(name.as_bytes()).map_err(|invalid_err| {
                    PyValueError::new_err(format!("Invalid header name: '{}'", invalid_err))
                })?;

                let value = HeaderValue::from_bytes(value.as_bytes()).map_err(|invalid_err| {
                    PyValueError::new_err(format!("Invalid header value: '{}'", invalid_err))
                })?;

                Ok(HeaderLine(name, value))
            })
        })
    }
}

impl From<HeaderLine> for (HeaderName, HeaderValue) {
    fn from(header_line: HeaderLine) -> Self {
        let HeaderLine(name, value) = header_line;
        (name, value)
    }
}
