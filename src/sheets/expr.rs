use serde_json::Number;
use derive_more::{From, IsVariant, Unwrap};

use super::*;

#[derive(Debug, Clone, From, IsVariant, Unwrap)]
// TODO: implement unwrap_value such that it moves self, reducing cloning
pub enum Expr {
    Value(Box<dyn Value>),
    Ref(Position),
    Form(OpInfo),
    Err(CellError),
}

impl PartialEq for Expr {
    fn eq(&self, rhs: &Expr) -> bool {
        if self.is_err() && rhs.is_err() {
            self.clone().unwrap_err() == rhs.clone().unwrap_err()
        } else if self.clone().is_form() && rhs.clone().is_form() {
            self.clone().unwrap_form() == rhs.clone().unwrap_form()
        } else if self.is_ref() && rhs.is_ref() {
            self.clone().unwrap_ref() == rhs.clone().unwrap_ref()
        } else if self.is_value() == rhs.is_value() {
            (self.clone().unwrap_value() as Box<dyn DynEq>)
                == (rhs.clone().unwrap_value() as Box<dyn DynEq>)
        } else {
            false
        }
    }
}

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