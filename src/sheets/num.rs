use derive_more::{self, Display, From};
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign};

use super::{value::Value, Expr};

impl Value for Num {}
// This newtype allows to change the underlying implementation
#[derive(Debug, Clone, Copy, Display, From)]
pub enum Num {
    #[display(fmt = "{}", _0.display())]
    F(f64),
    #[display(fmt = "{}", _0.display())]
    I(i64),
}

impl PartialOrd for Num {
    fn partial_cmp(&self, rhs: &Num) -> Option<std::cmp::Ordering> {
        match self {
            Num::I(i1) => match rhs {
                Num::I(i2) => i1.partial_cmp(&i2),
                Num::F(f2) => ( *i1 as f64 ).partial_cmp(&f2),
            },
            Num::F(f1) => match rhs {
                Num::F(f2) => f1.partial_cmp(f2),
                Num::I(i2) => ( *f1 ).partial_cmp(&( *i2 as f64) ),
            },
        }
    }
}

impl PartialEq for Num {
    fn eq(&self, rhs: &Num) -> bool {
        match self {
            Num::I(i1) => match rhs {
                Num::I(i2) => i1 == i2,
                Num::F(f2) => *i1 as f64 == *f2,
            },
            Num::F(f1) => match rhs {
                Num::F(f2) => f1 == f2,
                Num::I(i2) => *f1 == (*i2 as f64),
            },
        }
    }
}

impl Eq for Num {}

impl Add for Num {
    type Output = Num;

    fn add(self, rhs: Self) -> Self::Output {
        match self {
            Num::I(i1) => match rhs {
                Num::I(i2) => Num::I(i1 + i2),
                Num::F(f2) => Num::F(i1 as f64 + f2),
            },
            Num::F(f1) => match rhs {
                Num::F(f2) => Num::F(f1 + f2),
                Num::I(i2) => Num::F(f1 + (i2 as f64)),
            },
        }
    }
}

impl Mul for Num {
    type Output = Num;

    fn mul(self, rhs: Self) -> Self::Output {
        match self {
            Num::I(i1) => match rhs {
                Num::I(i2) => Num::I(i1 * i2),
                Num::F(f2) => Num::F(i1 as f64 * f2),
            },
            Num::F(f1) => match rhs {
                Num::F(f2) => Num::F(f1 * f2),
                Num::I(i2) => Num::F(f1 * (i2 as f64)),
            },
        }
    }
}

impl Div for Num {
    type Output = Num;

    fn div(self, rhs: Self) -> Self::Output {
        match self {
            Num::I(i1) => match rhs {
                Num::I(i2) => {
                    if i1 % i2 == 0 {
                        Num::I(i1 / i2)
                    } else {
                        Num::F(i1 as f64 / i2 as f64)
                    }
                }
                Num::F(f2) => Num::F(i1 as f64 / f2),
            },
            Num::F(f1) => match rhs {
                Num::F(f2) => Num::F(f1 / f2),
                Num::I(i2) => Num::F(f1 / (i2 as f64)),
            },
        }
    }
}

impl AddAssign for Num {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs
    }
}

impl MulAssign for Num {
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs
    }
}

impl DivAssign for Num {
    fn div_assign(&mut self, rhs: Self) {
        *self = *self / rhs
    }
}

impl Into<f64> for Num {
    fn into(self) -> f64 {
        match self {
            Num::F(f) => f,
            Num::I(i) => i as f64,
        }
    }
}

impl Into<Box<dyn Value>> for Num {
    fn into(self) -> Box<dyn Value> {
        Box::new(self)
    }
}

impl From<Num> for Expr {
    fn from(value: Num) -> Self {
        Expr::Value(value.into())
    }
}
