use std::fmt;
use std::net::{IpAddr, SocketAddr};

use hyper::server::conn::AddrIncoming;
use hyper::server::Builder;
use hyper::Server;
use pyo3::{class::basic::PyObjectProtocol, exceptions::PyValueError, prelude::*};

#[pyclass(module = "asgi_server")]
pub struct Settings {
    host: IpAddr,
    port: u16,
}

#[pymethods]
impl Settings {
    #[new]
    fn new() -> Self {
        Settings::default()
    }

    #[getter]
    fn get_host(&self) -> String {
        self.host.to_string()
    }

    #[setter]
    fn set_host(&mut self, value: &str) -> PyResult<()> {
        self.host = match value {
            "localhost" => Ok(IpAddr::from([127, 0, 0, 1])),
            _ => value.parse::<IpAddr>().map_err(|_| {
                PyValueError::new_err(format!("'{}' is not a valid ip address", value))
            }),
        }?;
        Ok(())
    }

    #[getter]
    fn get_port(&self) -> u16 {
        self.port
    }

    #[setter]
    fn set_port(&mut self, value: u16) -> PyResult<()> {
        self.port = value;
        Ok(())
    }
}

#[pyproto]
impl PyObjectProtocol for Settings {
    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("{:?}", self))
    }
}

impl Clone for Settings {
    fn clone(&self) -> Self {
        Self {
            host: self.host.clone(),
            port: self.port.clone(),
        }
    }
}

impl From<&Settings> for Builder<AddrIncoming> {
    fn from(settings: &Settings) -> Self {
        let addr = SocketAddr::from(settings);
        Server::bind(&addr)
    }
}

impl From<&Settings> for SocketAddr {
    fn from(settings: &Settings) -> Self {
        SocketAddr::from((settings.host, settings.port))
    }
}

impl fmt::Debug for Settings {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Settings {{ host = '{:?}', port = {:?} }}",
            self.host, self.port
        )
    }
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            host: IpAddr::from([127, 0, 0, 1]),
            port: 3000,
        }
    }
}
