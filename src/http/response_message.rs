use std::convert::TryFrom;

use hyper::body::Bytes;
use pyo3::{
    types::{PyBytes, PyDict},
    PyAny, PyErr, PyResult,
};

use crate::asgi_message::AsgiMessage;

pub struct HttpResponseMessage {
    data: Bytes,
    more: bool,
}

impl HttpResponseMessage {
    fn get_bytes(message_dict: &PyDict) -> PyResult<Bytes> {
        let py_bytes = PyAny::get_item(&message_dict, "body")?
            .downcast::<PyBytes>()
            .map_err(PyErr::from)?;

        Ok(Bytes::copy_from_slice(py_bytes.as_bytes()))
    }

    fn get_is_more_data(message_dict: &PyDict) -> PyResult<bool> {
        PyAny::get_item(&message_dict, "more_body")?.extract::<bool>()
    }

    pub fn is_last_message(&self) -> bool {
        return !self.more;
    }
}

impl AsgiMessage for HttpResponseMessage {
    fn message_type() -> &'static str {
        "http.response.body"
    }
}

impl TryFrom<&PyDict> for HttpResponseMessage {
    type Error = PyErr;

    fn try_from(message_dict: &PyDict) -> Result<Self, Self::Error> {
        HttpResponseMessage::validate_message_type(message_dict)
            .and_then(|_| HttpResponseMessage::get_bytes(message_dict))
            .map(|data| {
                let more = HttpResponseMessage::get_is_more_data(message_dict).unwrap_or(false);
                HttpResponseMessage { data, more }
            })
    }
}

impl From<HttpResponseMessage> for Bytes {
    fn from(message: HttpResponseMessage) -> Self {
        message.data
    }
}
