use std::any::Any;
use std::fmt::{Debug, Display};

use downcast_rs::{impl_downcast, Downcast};
use dyn_clonable;
use dyn_clone::DynClone;
use dyn_eq::DynEq;
use dyn_ord;

use super::into_serde_value::IntoSerdeValue;

#[dyn_clonable::clonable]
/// This trait is implemented for any type which can be used as a value in a cell
pub trait Value:
    Any + Debug + Display + DynClone + DynEq + dyn_ord::DynOrd + Clone + Downcast + IntoSerdeValue
{
}
impl_downcast!(Value);

impl From<&str> for Box<dyn Value> {
    fn from(value: &str) -> Self {
        Box::new(value.to_owned())
    }
}

impl Value for String {}
impl Value for bool {}
