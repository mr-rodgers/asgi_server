mod request_message;
mod response_body;
mod response_head;
mod response_message;
mod scope;

use std::{
    convert::{Infallible, TryFrom},
    net::SocketAddr,
};

use futures::{
    channel::mpsc,
    channel::oneshot,
    future::{self, AbortHandle},
    stream, Future, FutureExt, Stream, StreamExt,
};
use http::{response, Request, Response};
use hyper::Body;
use pyo3::{
    exceptions::PyValueError,
    types::{IntoPyDict, PyDict},
    Py, PyErr, PyResult, Python,
};

use self::{
    request_message::HttpRequestMessage, response_body::HttpResponseBody,
    response_head::HttpResponseStart,
};
use crate::{asgi_driver::AsgiDriver, error};
use crate::{asgi_scope, helpers::TryIntoPyDict};

pub async fn handle_request(
    server_addr: SocketAddr,
    remote_addr: SocketAddr,
    request: Request<Body>,
    asgi_driver: AsgiDriver,
) -> Result<Response<HttpResponseBody>, Infallible> {
    let (parts, request_body) = request.into_parts();
    let req_method = parts.method.to_string();
    let req_path = parts
        .uri
        .path_and_query()
        .map(|pq| pq.as_str())
        .unwrap_or("/")
        .to_string();
    let req_ver = version_as_string(&parts.version);

    let (disconnect_emitter, disconnect_event) = oneshot::channel::<()>();
    let stream_to_py = get_messages_to_py_stream(request_body, disconnect_event);

    // These channels will be used to communicate between python and rust
    let (results_from_py, mut messages_to_rust) = mpsc::channel::<Py<pyo3::types::PyDict>>(1);
    let (results_from_rust, messages_to_py) = mpsc::channel::<Py<pyo3::types::PyDict>>(1);

    let rt = pyo3_asyncio::tokio::get_runtime();

    rt.spawn(stream_to_py.forward(results_from_rust));
    rt.spawn(call_asgi_app(
        scope::build(parts, remote_addr.clone(), server_addr),
        asgi_driver,
        messages_to_py,
        results_from_py,
    ));

    let response_head = messages_to_rust
        .next()
        .await
        .ok_or_else(|| PyValueError::new_err("No response start message received"))
        .and_then(|message_dict| HttpResponseStart::try_from(message_dict));

    if response_head.is_ok() {
        disconnect_emitter.send(()).unwrap_or(());
    }

    response_head
        .and_then(|head| {
            log::info!(
                "{} - \"{} {} {}\" {}",
                remote_addr,
                req_method,
                req_path,
                req_ver,
                head.status_code()
            );
            build_response(head, messages_to_rust)
        })
        .or_else(handle_error)
}

fn call_asgi_app(
    scope_provider: impl asgi_scope::ScopeProvider,
    asgi_driver: AsgiDriver,
    messages_to_py: mpsc::Receiver<Py<PyDict>>,
    results_from_py: mpsc::Sender<Py<PyDict>>,
) -> impl Future<Output = ()> {
    Python::with_gil(|py| {
        scope_provider
            .try_into_py_dict(py)
            .and_then(move |scope_dict| {
                asgi_driver.create_context(scope_dict.as_ref(py), messages_to_py, results_from_py)
            })
            .map(|asgi_context| {
                asgi_context.then(|result| match result {
                    _ => future::ready(()),
                })
            })
            .map(future::Either::Left)
            .map_err(error::ApplicationError::from)
            .or_else(|err| err.handle(future::Either::Right(future::ready(()))))
            .unwrap()
    })
}

fn get_messages_to_py_stream(
    body: Body,
    disconnect_event: oneshot::Receiver<()>,
) -> impl Stream<Item = Result<Py<PyDict>, mpsc::SendError>> {
    let (message_stream, message_stream_abort_handle) = stream::abortable(get_message_stream(body));
    message_stream
        .chain(get_disconnect_stream(
            disconnect_event,
            message_stream_abort_handle,
        ))
        .map(|message_dict| Ok(message_dict))
}

fn get_message_stream(body: Body) -> impl Stream<Item = Py<PyDict>> {
    HttpRequestMessage::stream_body(body).map(|message| {
        Python::with_gil(|py| {
            let message_dict: Py<PyDict> = message.into_py_dict(py).into();
            message_dict
        })
    })
}

fn get_disconnect_stream(
    disconnect_event: oneshot::Receiver<()>,
    message_stream_abort_handle: AbortHandle,
) -> impl Stream<Item = Py<PyDict>> {
    stream::once(async move {
        disconnect_event.await.unwrap_or(());

        // don't send any more request messages into Python
        message_stream_abort_handle.abort();

        Python::with_gil(|py| {
            let message_dict = PyDict::new(py);
            message_dict
                .set_item("type", "http.disconnect")
                .unwrap_or(());
            let message_dict: Py<PyDict> = message_dict.into_py_dict(py).into();
            message_dict
        })
    })
}

fn build_response(
    response_head: HttpResponseStart,
    messages_to_rust: mpsc::Receiver<Py<PyDict>>,
) -> PyResult<Response<HttpResponseBody>> {
    let builder = response::Builder::from(response_head);
    let body = HttpResponseBody::from(messages_to_rust);
    builder.body(body).map_err(|err| {
        PyValueError::new_err(format!(
            "Could not start sending request: {}",
            err.to_string()
        ))
    })
}

fn handle_error(err: PyErr) -> Result<Response<HttpResponseBody>, Infallible> {
    let err = error::ApplicationError::from(err);
    let default_response = response::Response::builder()
        .status(500)
        .body(HttpResponseBody::new())
        .unwrap();
    err.handle(default_response)
}

fn version_as_string(version: &http::Version) -> String {
    let version_name = match version {
        &http::Version::HTTP_09 => "0.9",
        &http::Version::HTTP_10 => "1.0",
        &http::Version::HTTP_11 => "1.1",
        &http::Version::HTTP_2 => "2",
        &http::Version::HTTP_3 => "3",
        _ => "?",
    };
    format!("HTTP/{}", version_name)
}
