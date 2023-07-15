use derive_more::{Deref, DerefMut};
// use dyn_eq::DynEq;
use dyn_ord::{DynOrd, DynEq};

use super::value::Value;

#[derive(Debug, Clone, Deref, DerefMut)]
pub struct BoxValue(Box<dyn Value>);

impl BoxValue {
    pub fn move_inner(self) -> Box<dyn Value> {
        self.0
    }
}

impl<V> From<V> for BoxValue
where
    V: Value,
{
    fn from(value: V) -> Self {
        BoxValue(Box::new(value))
    }
}

impl From<Box<dyn Value>> for BoxValue {
    fn from(value: Box<dyn Value>) -> Self {
        BoxValue(value)
    }
}

impl PartialEq for BoxValue {
    fn eq(&self, rhs: &BoxValue) -> bool {
        let self_: &Box<_> = self;
        let rhs_: &Box<_> = rhs;
        ((self_.clone()) as Box<dyn DynEq>) == ((rhs_.clone()) as Box<dyn DynEq>)
    }
}

impl Eq for BoxValue {}

impl PartialOrd for BoxValue {
    fn partial_cmp(&self, rhs: &BoxValue) -> std::option::Option<std::cmp::Ordering> {
        let self_: Box<dyn DynOrd> = self.0.clone() as Box<dyn DynOrd>;
        let rhs_: Box<dyn DynOrd> = rhs.0.clone() as Box<dyn DynOrd>;
        self_.partial_cmp(&rhs_)
    }
}
