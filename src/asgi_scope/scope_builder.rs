use pyo3::{types::PyDict, PyResult};

use super::scope_provider::ScopeProvider;

pub struct ScopeBuilder<'a> {
    providers: Vec<Box<dyn ScopeProvider + 'a>>,
}

impl<'a> ScopeBuilder<'a> {
    pub fn new() -> Self {
        ScopeBuilder {
            providers: Vec::new(),
        }
    }

    pub fn add_provider(mut self, provider: impl ScopeProvider + 'a) -> Self {
        self.providers.push(Box::new(provider));
        self
    }
}

impl<'a> ScopeProvider for ScopeBuilder<'a> {
    fn add_scope(&self, scope_dict: &PyDict) -> PyResult<()> {
        for provider in &self.providers {
            provider.add_scope(scope_dict)?;
        }
        Ok(())
    }
}
