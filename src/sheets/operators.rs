//! Contains implementations of operators

use crate::types::box_value::BoxValue;

use super::*;
use std::any::TypeId;
use std::ops::RangeInclusive;

pub type Operator = Box<dyn Fn(&mut Sheet, &mut OpInfo) -> Expr>;
// type ErrHandler = (
//     RangeInclusive<usize>,
//     Box<dyn FnMut(&[Expr]) -> Box<dyn Iterator<Item = CellError>>>,
// );

type TypeHandler = (RangeInclusive<usize>, TypeId, &'static str);

type ValueHandler = Box<dyn FnMut(&[BoxValue]) -> Expr>;
type ExprHandler = Box<dyn FnMut(&[&Expr]) -> Expr>;

/// This guarantees that the number of `Expr`s provided to
/// `Handler` is at least: `handler.0.start() - expect_args.start()`
///
/// `TypeHandler`s are run if no err_handler has returned any errors
/// and argument count matched `expect_args`
///
/// NOTE: `TypeHandler`s expect all `Expr`s to be `Value`
/// otherwise it will panic
///
/// Current issues:
/// * it is possible to accidentally provide incorrect type_handlers
///     which will probably cause panics as ValueHandler will expect certain types
struct ErrorHandler {
    expect_args: RangeInclusive<usize>,
    // err_handlers: Vec<ErrHandler>,
    type_handlers: Vec<TypeHandler>,
    val_handler: ValueHandler,
    expr_handler: ExprHandler,
}

const MAX_ARGS: usize = u32::MAX as usize;

trait GetRange<T> {
    // fn get_range_mut(&mut self, range: &RangeInclusive<usize>) -> &mut [T];
    fn get_range(&self, range: &RangeInclusive<usize>) -> &[T];
}

impl<T> GetRange<T> for Vec<T> {
    fn get_range(&self, range: &RangeInclusive<usize>) -> &[T] {
        let split1 = range.start();
        let split2 = range.end() - range.start();
        let len = self.len() as usize - 1;
        let (_, right) = self.split_at(len.min(*split1));

        let len = right.len();
        let (left, _) = right.split_at(len.min(split2));

        left
    }
}

impl ErrorHandler {
    fn new(expect_args: RangeInclusive<usize>) -> Self {
        ErrorHandler {
            expect_args,
            // err_handlers: Vec::new(),
            type_handlers: Vec::new(),
            val_handler: Box::new(|_| panic!("Forgot to add val_handler")),
            expr_handler: Box::new(|_| panic!("Forgot to add val_handler")),
        }
    }

    // fn add_err_handler(&mut self, handler: ErrHandler) -> &mut Self {
    //     self.err_handlers.push(handler);
    //     self
    // }

    fn add_type_handler(mut self, handler: TypeHandler) -> Self {
        self.type_handlers.push(handler);
        self
    }

    fn add_value_handler(mut self, handler: ValueHandler) -> Self {
        self.val_handler = Box::new(handler);
        self
    }

    fn add_expr_handler(mut self, handler: ExprHandler) -> Self {
        self.expr_handler = Box::new(handler);
        self
    }

    fn handle_as_expr(mut self, op_info: &OpInfo) -> Expr {
        let len = op_info.args.len();
        let expect = &self.expect_args;

        if !expect.contains(&len) {
            return Expr::Err(CellError::InvalidArgCount(expect.clone(), len));
        }

        // for (range, handler) in self.err_handlers.iter_mut() {
        //     let args = op_info.args.get_range(range);
        //     errs.extend(handler(args));
        // }

        // if errs.len() > 0 {
        //     return Expr::from(CellError::FormError(errs));
        // }

        let mut errs: Vec<(usize, _)> = Vec::new();
        for (range, expected_type, expected_type_name) in self.type_handlers.iter_mut() {
            let enumerated = op_info.args.iter().enumerate().collect::<Vec<_>>();
            let args = enumerated.get_range(range);
            for (pos, expr) in args.iter() {
                let expr = expr.unwrap_value_ref();
                if (*expr).type_id() != *expected_type {
                    errs.push((*pos, CellError::TypeMismatch(expected_type_name)));
                }
            }
        }

        if errs.len() > 0 {
            return Expr::Err(CellError::FormError(errs));
        }

        let mut handler = self.expr_handler;
        handler(&op_info.args.iter().collect::<Vec<_>>()[..])
    }

    fn handle(mut self, op_info: &OpInfo) -> Expr {
        let len = op_info.args.len();
        let expect = &self.expect_args;

        if !expect.contains(&len) {
            return Expr::Err(CellError::InvalidArgCount(expect.clone(), len));
        }

        // for (range, handler) in self.err_handlers.iter_mut() {
        //     let args = op_info.args.get_range(range);
        //     errs.extend(handler(args));
        // }

        // if errs.len() > 0 {
        //     return Expr::from(CellError::FormError(errs));
        // }

        let mut errs: Vec<(usize, _)> = Vec::new();
        for (range, expected_type, expected_type_name) in self.type_handlers.iter_mut() {
            let enumerated = op_info.args.iter().enumerate().collect::<Vec<_>>();
            let args = enumerated.get_range(range);
            for (pos, expr) in args.iter() {
                let expr = expr.unwrap_value_ref();
                if (*expr).type_id() != *expected_type {
                    errs.push((*pos, CellError::TypeMismatch(expected_type_name)));
                }
            }
        }

        if errs.len() > 0 {
            return Expr::Err(CellError::FormError(errs));
        }

        let mut handler = self.val_handler;
        handler(
            &op_info
                .args
                .iter()
                .map(|e| e.unwrap_value_ref().clone())
                .collect::<Vec<_>>()[..],
        )
    }
}

fn find_errors<'a>(self_: &'a OpInfo) -> impl Iterator<Item = (usize, &'a CellError)> {
    self_
        .args
        .iter()
        .enumerate()
        .filter(|(_, expr)| expr.is_err())
        .map(|(ix, expr): (_, &Expr)| (ix, expr.unwrap_err_ref()))
}

fn find_errs<C: Value + Clone>(
    self_: &mut OpInfo,
    type_err: &'static str,
) -> Vec<(usize, CellError)> {
    self_
        .args
        .iter()
        .enumerate()
        .filter(|(_, expr)| expr.is_err() || expr.unwrap_value_ref().downcast_ref::<C>().is_none())
        .map(|(ix, expr): (_, &Expr)| match expr {
            Expr::Value(_) => (ix + 1, CellError::TypeMismatch(type_err)),
            Expr::Err(e) => (ix + 1, e.clone()),
            _ => unreachable!(),
        })
        .collect::<Vec<_>>()
}

fn expect_n<'a, T: Value + Clone>(
    self_: &'a mut OpInfo,
    type_err: &'static str,
    acceptable_range: RangeInclusive<usize>,
) -> Result<impl Iterator<Item = T> + 'a, CellError> {
    let errors = find_errs::<T>(self_, type_err);
    if errors.len() > 0 {
        return Err(CellError::FormError(errors));
    }

    if !acceptable_range.contains(&self_.args.len()) {
        return Err(CellError::InvalidArgCount(
            acceptable_range,
            self_.args.len(),
        ));
    }

    Ok(self_
        .args
        .iter()
        .map(|e| e.unwrap_value_ref().downcast_ref::<T>().unwrap().clone()))
}

fn expect_two<'a, T: Value + Clone>(
    self_: &'a mut OpInfo,
    type_err: &'static str,
) -> Result<(T, T), CellError> {
    let mut arg_iter = expect_n::<T>(self_, type_err, 2..=2)?;
    let arg1: T = arg_iter.next().unwrap();
    let arg2: T = arg_iter.next().unwrap();

    Ok((arg1, arg2))
}

fn foldr<C: Value + Clone, T>(
    self_: &mut OpInfo,
    init: T,
    f: impl Fn(T, C) -> T,
    type_err: &'static str,
) -> Result<T, CellError> {
    // find and return errors
    let errors = find_errs::<C>(self_, type_err);

    if errors.len() > 0 {
        return Err(CellError::FormError(errors));
    }

    Ok(self_.args.iter().fold(init, |acc, e| {
        f(
            acc,
            e.unwrap_value_ref().downcast_ref::<C>().unwrap().clone(),
        )
    }))
}

fn foldr_with_check<C: Value + Clone, T: Value>(
    self_: &mut OpInfo,
    init: T,
    f: impl Fn(T, C) -> T,
    type_err: &'static str,
    acceptable_range: RangeInclusive<usize>,
) -> Expr {
    if !acceptable_range.contains(&self_.args.len()) {
        return Expr::Err(CellError::InvalidArgCount(
            acceptable_range,
            self_.args.len(),
        ));
    }

    foldr(self_, init, f, type_err)
        .map(|n| Expr::Value(n.into()))
        .unwrap_or_else(|e| Expr::Err(e))
}

pub fn get_default_op_map<'a>() -> HashMap<&'a str, Operator> {
    let sum = Box::new(|_: &mut _, info: &mut OpInfo| {
        foldr_with_check(info, Num::I(0), |acc, n| acc + n, "Num", 1..=MAX_ARGS)
    });

    let mul: Operator = Box::new(|_, info| {
        foldr_with_check(info, Num::I(1), |acc, n| acc * n, "Num", 1..=MAX_ARGS)
    });

    let div: Operator = Box::new(|_, info| {
        ErrorHandler::new(2..=2)
            .add_type_handler((
                2..=2,
                {
                    let b: Box<dyn Value> = Box::new(Num::I(0));
                    b.type_id()
                },
                "Num",
            ))
            .add_value_handler(Box::new(|args: &[BoxValue]| {
                let arg1 = &args[0].downcast_ref::<Num>().unwrap();
                let arg2 = &args[1].downcast_ref::<Num>().unwrap();

                if arg2 == &&Num::I(0) || arg2 == &&Num::F(0.0) {
                    return Expr::Err(CellError::DivByZero);
                }

                Expr::Value((**arg1 / **arg2).into())
            }))
            .handle(info)
    });

    let gt: Operator = Box::new(|_, info| {
        ErrorHandler::new(2..=2)
            .add_value_handler(Box::new(|args: &[BoxValue]| {
                let arg1 = &args[0];
                let arg2 = &args[1];
                if arg1.type_id() != arg2.type_id() {
                    return Expr::Err(CellError::BinaryTypeMismatch.into());
                }
                Expr::Value((
                    arg1 > arg2
                ).into())
            }))
            .handle(info)
    });

    let eq: Operator = Box::new({
        |_, info| {
            ErrorHandler::new(2..=2)
                .add_expr_handler(Box::new(|args: &[&Expr]| {
                    let arg1 = args[0].unwrap_value_ref();
                    let arg2 = args[1].unwrap_value_ref();
                    if arg1.type_id() != arg2.type_id() {
                        return Expr::Err(CellError::BinaryTypeMismatch.into());
                    }
                    Expr::Value((args.get(0) == args.get(1)).into())
                }))
                .handle_as_expr(info)
        }
    });

    let not: Operator = Box::new(|_, info| {
        let res = expect_n::<bool>(info, "Boolean", 1..=1);

        match res {
            Ok(mut arg_iter) => Expr::Value((!arg_iter.next().unwrap()).into()),
            Err(e) => Expr::Err(e),
        }
    });

    let and: Operator = Box::new(|_, info| {
        foldr_with_check(info, true, |acc, n: bool| acc && n, "String", 1..=MAX_ARGS)
    });

    let or: Operator = Box::new(|_, info| {
        foldr_with_check(info, false, |acc, n: bool| acc || n, "String", 1..=MAX_ARGS)
    });

    let r#if: Operator = Box::new(|_, info| {
        ErrorHandler::new(3..=3)
            .add_type_handler((
                0..=0,
                {
                    let b: Box<dyn Value> = Box::new(false);
                    b.type_id()
                },
                "Boolean"
            ))
            .add_value_handler(Box::new(|args| {
                let cond = args[0].downcast_ref::<bool>();
                let cond = cond.unwrap();
        
                let arg1 = &args[1];
                let arg2 = &args[2];
                if arg1.type_id() != arg2.type_id() {
                    return Expr::Err(CellError::BinaryTypeMismatch.into());
                }
                Expr::Value(if *cond { arg1.clone().into() } else { arg2.clone().into() })
            }))
            .handle(info)
    });

    let concat: Operator = Box::new(|_, info| {
        foldr_with_check(
            info,
            String::from(""),
            |acc, n: String| acc + &n[..],
            "String",
            0..=MAX_ARGS,
        ) // Spec does not mention min args, I'll assume 0
    });

    HashMap::from([
        ("MULTIPLY", mul),
        ("SUM", sum),
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
