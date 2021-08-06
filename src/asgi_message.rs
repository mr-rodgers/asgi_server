use pyo3::{
    exceptions::PyValueError,
    types::{PyDict, PyUnicode},
    PyAny, PyResult,
};

pub trait AsgiMessage {
    fn message_type() -> &'static str;

    fn validate_message_type(message_dict: &PyDict) -> PyResult<()> {
        let message_type = PyAny::get_item(&message_dict, "type")?
            .downcast::<PyUnicode>()?
            .to_str()?;

        if !message_type.eq(Self::message_type()) {
            Err(PyValueError::new_err(format!(
                "Unexpected message type: '{}'",
                message_type
            )))
        } else {
            Ok(())
        }
    }
}
