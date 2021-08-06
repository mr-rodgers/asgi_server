use pyo3::{
    exceptions::PyValueError,
    types::{PyBytes, PyDict, PyList, PyTuple},
    PyResult, Python,
};
use std::net::SocketAddr;

use crate::asgi_scope;

use super::scope_provider::HttpScopeProvider;

pub struct HttpVersion;
impl HttpScopeProvider for HttpVersion {
    fn add_scope(&self, parts: &http::request::Parts, scope_dict: &PyDict) -> PyResult<()> {
        let version = match parts.version {
            http::version::Version::HTTP_10 => Some("1.0"),
            http::version::Version::HTTP_11 => Some("1.1"),
            http::version::Version::HTTP_2 => Some("2"),
            _ => None,
        };
        let version = version.ok_or_else(|| PyValueError::new_err("Unsupported HTTP Version"))?;
        scope_dict.set_item("http_version", version)
    }
}

pub struct HttpMethod;
impl HttpScopeProvider for HttpMethod {
    fn add_scope(&self, parts: &http::request::Parts, scope_dict: &PyDict) -> PyResult<()> {
        let method = parts.method.as_str().to_ascii_uppercase();
        scope_dict.set_item("method", method)
    }
}

pub struct HttpScheme;
impl HttpScopeProvider for HttpScheme {
    fn add_scope(&self, parts: &http::request::Parts, scope_dict: &PyDict) -> PyResult<()> {
        if let Some(scheme) = parts.uri.scheme_str() {
            scope_dict.set_item("scheme", scheme)
        } else {
            Ok(())
        }
    }
}

pub struct HttpPath;
impl HttpScopeProvider for HttpPath {
    fn add_scope(&self, parts: &http::request::Parts, scope_dict: &PyDict) -> PyResult<()> {
        scope_dict.set_item("path", parts.uri.path())
    }
}

pub struct HttpQueryString;
impl HttpScopeProvider for HttpQueryString {
    fn add_scope(&self, parts: &http::request::Parts, scope_dict: &PyDict) -> PyResult<()> {
        if let Some(query_str) = parts.uri.query() {
            let percent_encoded = percent_encoding::utf8_percent_encode(
                query_str,
                percent_encoding::NON_ALPHANUMERIC,
            );
            scope_dict.set_item("method", percent_encoded.to_string().as_bytes())
        } else {
            Ok(())
        }
    }
}

pub struct HttpHeaders;
impl HttpScopeProvider for HttpHeaders {
    fn add_scope(&self, parts: &http::request::Parts, scope_dict: &PyDict) -> PyResult<()> {
        Python::with_gil(|py| {
            let py_headers_list = PyList::empty(py);
            for (header_name, header_value) in &parts.headers {
                let header_name = PyBytes::new(py, header_name.as_str().to_lowercase().as_bytes());
                let header_value = PyBytes::new(py, header_value.as_bytes());
                py_headers_list.append((header_name, header_value))?;
            }
            scope_dict.set_item("headers", py_headers_list)
        })
    }
}

pub enum HttpAddress {
    ClientSocket(SocketAddr),
    ServerSocket(SocketAddr),
}

impl asgi_scope::ScopeProvider for HttpAddress {
    fn add_scope(&self, scope_dict: &PyDict) -> PyResult<()> {
        match self {
            HttpAddress::ClientSocket(addr) => {
                scope_dict.set_item("client", (addr.ip().to_string(), addr.port()))
            }
            HttpAddress::ServerSocket(addr) => {
                scope_dict.set_item("server", (addr.ip().to_string(), addr.port()))
            }
        }
    }
}

pub struct HttpServer(Vec<SocketAddr>);
impl asgi_scope::ScopeProvider for HttpServer {
    fn add_scope(&self, scope_dict: &PyDict) -> PyResult<()> {
        let HttpServer(server_addresses) = self;
        Python::with_gil(|py| {
            let py_server_tuple = PyTuple::new(
                py,
                server_addresses
                    .into_iter()
                    .map(|addr| (addr.ip().to_string(), addr.port())),
            );
            scope_dict.set_item("server", py_server_tuple)
        })
    }
}
