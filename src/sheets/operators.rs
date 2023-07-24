//! Contains implementations of operators

use crate::types::box_value::BoxValue;

use super::*;
use std::ops::RangeInclusive;

/// A Type designed to handle type conversions (and errors when converting them)
/// and handle'ing arity of operators
#[derive(Debug)]
struct MyHandler<T> {
    err_state: Vec<CellError>,
    op_info: OpInfo,
    inner: T,
}

// helper function
// finds all type downcasting errors within specified range
// NOTE: it does not handle argument arity
fn find_type_errors<'a, T: Value + Clone>(
    info: &'a OpInfo,
    range: RangeInclusive<usize>,
    type_name: &'static str,
) -> impl Iterator<Item = CellError> + 'a {
    info.args
        .iter()
        .enumerate()
        .skip(*range.start())
        .take(*range.end())
        .filter(|(_, e)| !e.is_err())
        .map(|(u, e): (_, &Expr)| {
            (
                u,
                e.clone()
                    .unwrap_value()
                    .move_inner()
                    .downcast_ref::<T>()
                    .map(|v: &T| v.clone()),
            )
        })
        .filter(|(_, e)| e.is_none())
        .map(move |(u, _)| CellError::ArgError(u, Box::new(CellError::TypeMismatch(type_name))))
}

impl MyHandler<()> {
    fn new(op_info: OpInfo) -> MyHandler<()> {
        MyHandler {
            err_state: op_info
                .args
                .iter()
                .enumerate()
                .filter(|(_, e): &(_, &Expr)| e.is_err())
                .map(|(u, e)| CellError::ArgError(u, Box::new(e.unwrap_err_ref().clone())))
                .collect::<Vec<_>>(),
            inner: (),
            op_info,
        }
    }
}

impl<T> MyHandler<T> {
    /// Checks if constant amount of arguments exist within the `L..=U` range
    ///
    /// If you want to also check if the underlying types within the `Expr`
    /// use `handle_type_*` functions
    fn handle_const<const L: usize, const U: usize>(
        mut self: MyHandler<T>,
    ) -> Result<MyHandler<([BoxValue; U - L + 1], T)>, Vec<CellError>> {
        // handle arg count errors
        let len = (&self).op_info.args.len();
        if len < (U + 1) {
            (&mut self)
                .err_state
                .push(CellError::InvalidArgCount(L..=U, len))
        }

        if self.err_state.len() > 0 {
            Err(self.err_state)
        } else {
            let new_inner: [BoxValue; U - L + 1] = self.op_info.args[L..=U]
                .iter()
                .map(|e| e.clone().unwrap_value().clone())
                .collect::<Vec<BoxValue>>()
                .try_into()
                .unwrap();

            Ok(MyHandler {
                err_state: self.err_state,
                op_info: self.op_info,
                inner: (new_inner, self.inner),
            })
        }
    }

    /// Type-checks arguments within the specified range `L..=U`
    ///
    /// NOTE: because it handles a constant-length array
    /// if where are less than `U` arguments
    /// Err([Cell::InvalidArgCount]) will be returned amongst other errors
    /// if you want to handle non-constant amount of arguments use [handle_type]
    fn handle_type_const<V: Value + Clone, const L: usize, const U: usize>(
        mut self: MyHandler<T>,
        type_name: &'static str,
    ) -> Result<MyHandler<([V; U - L + 1], T)>, Vec<CellError>> {
        // handle arg count errors
        let len = (&self).op_info.args.len();
        if len < U + 1 {
            (&mut self)
                .err_state
                .push(CellError::InvalidArgCount(L..=U, len))
        }

        let op_info_ref = &self.op_info;
        let self_err_mut = &mut self.err_state;
        self_err_mut.extend(find_type_errors::<V>(op_info_ref, L..=U, type_name));

        if self.err_state.len() > 0 {
            Err(self.err_state)
        } else {
            let new_inner: [V; U - L + 1] = (&self.op_info.args[L..=U])
                .iter()
                .map(|e: &Expr| -> V { e.unwrap_downcast_ref::<V>().clone() })
                .collect::<Vec<_>>()
                .try_into()
                .unwrap();

            Ok(MyHandler {
                err_state: self.err_state,
                op_info: self.op_info,
                inner: (new_inner, self.inner),
            })
        }
    }

    /// Type-checks arguments within the specified range
    ///
    /// NOTE: This method allows for there to less arguments
    /// than in the specified range.
    ///
    /// If you want strictly the amount of arguments in
    /// the specified range use [handle_type_const]
    fn handle_type_variadic<V: Value + Clone>(
        mut self: MyHandler<T>,
        range: RangeInclusive<usize>,
        type_name: &'static str,
    ) -> Result<MyHandler<(Vec<V>, T)>, Vec<CellError>> {
        self.err_state.extend(
            find_type_errors::<V>(&(&self).op_info, range.clone(), type_name).collect::<Vec<_>>(),
        );

        if self.err_state.len() > 0 {
            Err(self.err_state)
        } else {
            let new_inner: Vec<V> = self
                .op_info
                .args
                .iter()
                .skip(*range.start())
                .take(*range.end() - *range.start() + 1)
                .map(|e: &Expr| -> V { e.unwrap_downcast_ref::<V>().clone() })
                .collect::<Vec<_>>();

            Ok(MyHandler {
                err_state: self.err_state,
                op_info: self.op_info,
                inner: (new_inner, self.inner),
            })
        }
    }

    fn finish(self) -> T {
        self.inner
    }
}

const MAX_ARGS: usize = u32::MAX as usize;
pub type Operator = Box<dyn Fn(&mut Sheet, &mut OpInfo) -> Result<Expr, Vec<CellError>>>;

pub fn get_default_op_map<'a>() -> HashMap<&'a str, Operator> {
    let sum: Operator = Box::new(|_, info: &mut OpInfo| {
        Ok(MyHandler::new(info.clone())
            .handle_type_variadic::<Num>(0..=MAX_ARGS, "Num")?
            .finish()
            .0
            .into_iter()
            .fold(Num::I(0), |n1, n2| n1 + n2)
            .into())
    });

    let mul: Operator = Box::new(|_, info| {
        Ok(MyHandler::new(info.clone())
            .handle_type_variadic::<Num>(0..=MAX_ARGS, "Num")?
            .finish()
            .0
            .into_iter()
            .fold(Num::I(1), |n1, n2| n1 * n2)
            .into())
    });

    let div: Operator = Box::new(|_, info| {
        let [l, r] = MyHandler::new(info.clone())
            .handle_type_const::<Num, 0, 1>("Num")?
            .finish()
            .0;

        if r == Num::I(0) || r == Num::F(0.0) {
            Err(vec![CellError::DivByZero])
        } else {
            Ok((l / r).into())
        }
    });

    let gt: Operator = Box::new(|_, info| {
        let [l, r] = MyHandler::new(info.clone())
            .handle_const::<0, 1>()?
            .finish()
            .0;
        if l.type_id() != r.type_id() {
            Err(vec![(CellError::BinaryTypeMismatch)])
        } else {
            Ok((l > r).into())
        }
    });

    let eq: Operator = Box::new(|_, info| {
        let [l, r] = MyHandler::new(info.clone())
            .handle_const::<0, 1>()?
            .finish()
            .0;

        if l.type_id() != r.type_id() {
            Err(vec![(CellError::BinaryTypeMismatch)])
        } else {
            Ok((l == r).into())
        }
    });

    let not: Operator = Box::new(|_, info| {
        Ok({
            let [bool] = MyHandler::new(info.clone())
                .handle_type_const::<bool, 0, 0>("Boolean")?
                .finish()
                .0;

            (!bool).into()
        })
    });

    let and: Operator = Box::new(|_, info: &mut OpInfo| {
        Ok(MyHandler::new(info.clone())
            .handle_type_variadic::<bool>(0..=MAX_ARGS, "Boolean")?
            .finish()
            .0
            .into_iter()
            .fold(true, |n1, n2| n1 && n2)
            .into())
    });

    let or: Operator = Box::new(|_, info: &mut OpInfo| {
        Ok(MyHandler::new(info.clone())
            .handle_type_variadic::<bool>(0..=MAX_ARGS, "Boolean")?
            .finish()
            .0
            .into_iter()
            .fold(false, |n1, n2| n1 || n2)
            .into())
    });

    let r#if: Operator = Box::new(|_, info| {
        let ([arg1, arg2], ([cond], ())) = MyHandler::new(info.clone())
            .handle_type_const::<bool, 0, 0>("Boolean")?
            .handle_const::<1, 2>()?
            .finish();

        Ok({
            if cond {
                arg1.into()
            } else {
                arg2.into()
            }
        })
    });

    let concat: Operator = Box::new(|_, info| {
        Ok(MyHandler::new(info.clone())
            .handle_type_variadic::<String>(0..=MAX_ARGS, "String")?
            .finish()
            .0
            .into_iter()
            .fold("".to_owned(), |n1, n2| n1 + &n2[..])
            .into())
    });

    HashMap::from([
        ("SUM", sum),
        ("MULTIPLY", mul),
        ("DIVIDE", div),
        ("GT", gt),
        ("EQ", eq),
        ("NOT", not),
        ("AND", and),
        ("OR", or),
        ("IF", r#if),
        ("CONCAT", concat),
    ])
}
