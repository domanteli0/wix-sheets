use std::convert::Into;

use serde_json::Value as SerdeValue;

pub trait IntoSerdeValue {
    fn into_serde_value(self: Box<Self>) -> SerdeValue;
}

impl<T: Into<SerdeValue>> IntoSerdeValue for T {
    fn into_serde_value(self: Box<Self>) -> SerdeValue {
        (*self).into()
    }
}
