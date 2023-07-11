use std::any::Any;
use std::fmt::{Debug, Display};

use downcast_rs::{impl_downcast, Downcast};
use dyn_clonable;
use dyn_clone::DynClone;
use dyn_eq::DynEq;
use dyn_ord;

#[dyn_clonable::clonable]
pub trait Value:
    Any + Debug + Display + DynClone + DynEq + dyn_ord::DynOrd + Clone + Downcast { }
impl_downcast!(Value);

impl From<&str> for Box<dyn Value> {
    fn from(value: &str) -> Self {
        Box::new(value.to_owned())
    }
}

impl Value for String {}
impl Value for bool {}
