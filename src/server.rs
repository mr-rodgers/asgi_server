mod settings;

use std::convert::Infallible;
use std::net::SocketAddr;

use crate::asgi_driver::AsgiDriver;
use crate::http;
use hyper::server::conn::{AddrIncoming, AddrStream};
use hyper::server::Builder;
use hyper::service::{make_service_fn, service_fn};
use pyo3::Python;
pub use settings::Settings;

pub async fn start_http_server(driver: AsgiDriver, settings: pyo3::Py<Settings>) {
    let settings = Python::with_gil(|py| {
        let settings: &Settings = &*settings.borrow(py);
        settings.clone()
    });
    let settings_clone = settings.clone();

    let make_service = make_service_fn(move |conn: &AddrStream| {
        let driver = driver.clone();
        let remote_addr = conn.remote_addr();
        let server_addr = SocketAddr::from(&settings_clone);
        let service = service_fn(move |request| {
            http::handle_request(server_addr, remote_addr, request, driver.clone())
        });

        async move { Ok::<_, Infallible>(service) }
    });

    let server = Builder::<AddrIncoming>::from(&settings).serve(make_service);

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}
