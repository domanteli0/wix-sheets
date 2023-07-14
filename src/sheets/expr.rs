use derive_more::{Deref, DerefMut, IsVariant, Unwrap};
use serde_json::Number;

use super::*;

/// `Expr`, short for expression, which represents
/// all possible values which a cell can contain
#[derive(Debug, Clone, IsVariant, Unwrap, PartialEq)]
pub enum Expr {
    Value(BoxValue),
    Ref(Position),
    Form(OpInfo),
    Err(CellError),
}

impl<V> From<V> for Expr
where
    V: Value,
{
    fn from(value: V) -> Self {
        Expr::Value(value.into())
    }
}

impl From<Box<dyn Value>> for Expr {
    fn from(value: Box<dyn Value>) -> Self {
        Expr::Value(value.into())
    }
}

impl From<CellError> for Expr {
    fn from(value: CellError) -> Self {
        Expr::Err(value)
    }
}

impl From<BoxValue> for Expr {
    fn from(value: BoxValue) -> Self {
        Expr::Value(value)
    }
}

impl Expr {
    pub fn unwrap_value_ref(&self) -> &BoxValue {
        match self {
            Expr::Value(v) => v,
            _ => panic!("unwrap_value on a Expr with is not Value"),
        }
    }

    pub fn unwrap_downcast_ref<T: Value>(&self) -> &T {
        self.unwrap_value_ref().downcast_ref::<T>().unwrap()
    }

    pub fn unwrap_err_ref(&self) -> &CellError {
        match self {
            Expr::Err(e) => e,
            _ => panic!("unwrap_err on a Expr with is not Err"),
        }
    }

    pub fn map_value_mut(&mut self, f: impl FnOnce(&mut BoxValue)) {
        match self {
            Expr::Value(v) => f(v),
            _ => {}
        }
    }
}

/// This impl is used for serialization
impl Into<SerdeValue> for Expr {
    fn into(self) -> SerdeValue {
        match self {
            Expr::Value(v) => {
                if let Some(b) = v.downcast_ref::<bool>() {
                    return SerdeValue::Bool(*b);
                }
                if let Some(n) = v.downcast_ref::<Num>() {
                    return SerdeValue::Number(Number::from_f64((*n).into()).unwrap());
                }
                SerdeValue::String(v.to_string())
            }
            Expr::Err(e) => serde_json::value::Value::String(e.to_string()),
            _ => unreachable!("Assumed resolved"),
        }
    }
}

#[derive(Debug, Clone, Deref, DerefMut)]
pub struct BoxValue(Box<dyn Value>);

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
    fn partial_cmp(&self, rhs: &expr::BoxValue) -> std::option::Option<std::cmp::Ordering> {
        let self_: Box<dyn DynOrd> = self.0.clone() as Box<dyn DynOrd>;
        let rhs_: Box<dyn DynOrd> = rhs.0.clone() as Box<dyn DynOrd>;
        self_.partial_cmp(&rhs_)
    }
}
