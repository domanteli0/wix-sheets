use std::{str::FromStr, ops::{Add, AddAssign}};
use derive_more::{self, Display, From};

use super::{Expr, value::Value};

impl Value for Num {}
// This newtype allows to change the underlying implementation
#[derive(Debug, PartialEq, Eq, PartialOrd, Clone, Copy, Display, From)]
pub enum Num{
    // #[display(fmt = "{}", _0.display())]
    // F(f64),
    #[display(fmt = "{}", _0.display())]
    I(i64)
}

impl Add for Num {
    type Output = Num;

    fn add(self, rhs: Self) -> Self::Output {
        match self {
            Num::I(i1) => match rhs {
                Num::I(i2) => Num::I( i1 + i2 ),
            },
        }
    }
}

// impl Add for Num {
//     type Output = Num;
//     fn add(self, rhs: Self) -> Self::Output {
//         match self {
//             Num::F(f1) => {
//                 match rhs {
//                     Num::F(f2) => Num::F(f1 + f2),
//                     Num::I(i2) => Num::F(f1 + ( i2 as f64 )),
//                 }
//             },
//             Num::I(i1) => 
//                 match rhs {
//                     Num::F(f2) => Num::F((i1 as f64) + f2),
//                     Num::I(i2) => Num::I(i1 + i2 ),
//                 }
//         }
//     }
// }

impl AddAssign for Num {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs
    }
}

impl Into<f64> for Num {
    fn into(self) -> f64 {
        match self {
            // Num::F(f) => f,
            Num::I(i) => i as f64,
        }
    }
}

impl Into<Box<dyn Value>> for Num {
    fn into(self) -> Box<dyn Value> {
        Box::new(self)
    }
}

impl<'a> Into<Expr<'a>> for Num {
    fn into(self) -> Expr<'a> {
        Expr::Value(self.into())
    }
}

impl FromStr for Num {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let f = s.parse::<f64>();
        let i = s.parse::<i64>();

        // if let Ok(f) = f {
        //     return Ok(Num::F(f));
        // }
        if let Ok(i) = i {
            return Ok(Num::I(i));
        }

        Err(())
    }
}
