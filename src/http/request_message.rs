use futures::{stream, Stream, StreamExt};
use hyper::{body::Bytes, Body};
use pyo3::{
    types::{IntoPyDict, PyBytes, PyDict},
    Python,
};

use crate::error;

pub struct HttpRequestMessage {
    data: Bytes,
    more: bool,
}

impl HttpRequestMessage {
    pub fn stream_body(body: Body) -> impl Stream<Item = HttpRequestMessage> {
        body.map(|chunk| {
            chunk
                .map(|data| HttpRequestMessage { data, more: true })
                .map_err(|hyper_err| error::ServerError::from(hyper_err))
                .or_else(|err| {
                    err.handle(HttpRequestMessage {
                        data: Bytes::new(),
                        more: false,
                    })
                })
                .unwrap()
        })
        .chain(stream::once(async {
            HttpRequestMessage {
                data: Bytes::new(),
                more: false,
            }
        }))
    }
}

impl IntoPyDict for HttpRequestMessage {
    fn into_py_dict(self, py: Python) -> &PyDict {
        let data: &[u8] = &self.data;
        let data = PyBytes::new(py, data);
        let dict = PyDict::new(py);

        dict.set_item("type", "http.request").unwrap_or(());
        dict.set_item("body", data).unwrap_or(());
        dict.set_item("more_body", self.more).unwrap_or(());

        dict
    }
}
