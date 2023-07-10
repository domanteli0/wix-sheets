use std::any::Any;
use std::fmt::{ Debug, Display };

use dyn_ord;
use dyn_eq::DynEq;
use dyn_clone::DynClone;
use dyn_clonable;

#[dyn_clonable::clonable]
pub trait Value: Any + Debug + Display + DynClone + DynEq + dyn_ord::DynOrd + Clone { }
// dyn_eq::eq_trait_object!(Value);

impl dyn Value {
    // Once trait upcasting is stabilized (https://github.com/rust-lang/rust/issues/65991) this should work on rust-stable
    // this function is copy - pasted from here: (https://doc.rust-lang.org/src/alloc/boxed.rs.html#1700)
    pub fn downcast_ref<T: Any>(&self) -> Option<&T> {
        let self_ = self as &dyn Any;
        match self_.is::<T>() {
            true => unsafe { Some(self_.downcast_ref_unchecked()) }
            false => None,
        }
    }

    pub fn is<T: Any>(&self) -> bool {
        let self_ = self as &dyn Any;
        self_.is::<T>()
    }
}

impl From<&str> for Box<dyn Value> {
    fn from(value: &str) -> Self {
        Box::new(value.to_owned())
    }
}

impl Value for String {}
impl Value for bool {}