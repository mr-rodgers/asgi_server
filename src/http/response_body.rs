use std::{convert::TryFrom, pin::Pin, task::Poll};

use futures::{channel::mpsc, FutureExt, StreamExt};
use hyper::body::{Bytes, HttpBody};
use pyo3::{types::PyDict, Py, PyErr, Python};

use super::response_message::HttpResponseMessage;

pub struct HttpResponseBody {
    message_stream: Option<mpsc::Receiver<Py<PyDict>>>,
}

impl HttpResponseBody {
    pub fn new() -> Self {
        Self {
            message_stream: None,
        }
    }
}

impl From<mpsc::Receiver<Py<PyDict>>> for HttpResponseBody {
    fn from(message_stream: mpsc::Receiver<Py<PyDict>>) -> Self {
        Self {
            message_stream: Some(message_stream),
        }
    }
}

impl HttpBody for HttpResponseBody {
    type Data = Bytes;
    type Error = PyErr;

    fn poll_data(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Result<Self::Data, Self::Error>>> {
        let body = Pin::<&mut HttpResponseBody>::into_inner(self);

        if let Some(message_stream) = &mut body.message_stream {
            let mut next_message = message_stream.next();

            match next_message.poll_unpin(cx) {
                Poll::Pending => Poll::Pending,
                Poll::Ready(None) => Poll::Ready(None),
                Poll::Ready(Some(py_dict)) => {
                    let result = Python::with_gil(|py| {
                        let message = HttpResponseMessage::try_from(py_dict.as_ref(py))?;

                        if message.is_last_message() {
                            body.message_stream = None;
                        }

                        Ok(Bytes::from(message))
                    });
                    Poll::Ready(Some(result))
                }
            }
        } else {
            Poll::Ready(None)
        }
    }

    fn poll_trailers(
        self: Pin<&mut Self>,
        _: &mut std::task::Context<'_>,
    ) -> Poll<Result<Option<http::HeaderMap>, Self::Error>> {
        Poll::Ready(Ok(None))
    }
}
