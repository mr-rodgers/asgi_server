use pyo3::{types::PyDict, PyResult};

use super::scope_provider::HttpScopeProvider;
use crate::asgi_scope;

pub struct HttpScopeBuilder<'a> {
    parts: http::request::Parts,
    providers: Vec<Box<dyn HttpScopeProvider + 'a>>,
}

impl<'a> HttpScopeBuilder<'a> {
    pub fn new(parts: http::request::Parts) -> Self {
        HttpScopeBuilder {
            parts,
            providers: Vec::new(),
        }
    }

    pub fn add_provider(mut self, provider: impl HttpScopeProvider + 'a) -> Self {
        self.providers.push(Box::new(provider));
        self
    }
}

impl<'a> asgi_scope::ScopeProvider for HttpScopeBuilder<'a> {
    fn add_scope(&self, scope_dict: &PyDict) -> PyResult<()> {
        for provider in &self.providers {
            provider.add_scope(&self.parts, scope_dict)?;
        }
        Ok(())
    }
}
