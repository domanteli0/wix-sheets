use std::ops::{Bound, RangeBounds};

use super::*;

pub type Operator = Box<dyn Fn(&mut Sheet, &mut OpInfo) -> Expr>;

fn to_owned_bound(b: Bound<&usize>) -> Bound<usize> {
    match b {
        Bound::Included(i) => Bound::Included(*i),
        Bound::Excluded(i) => Bound::Excluded(*i),
        Bound::Unbounded => Bound::Unbounded,
    }
}

fn find_errs<C: Value + Clone>(
    self_: &mut OpInfo,
    type_err: &'static str,
) -> Vec<(usize, CellError)> {
    self_
        .args
        .iter()
        .enumerate()
        .filter(|(_, expr)| {
            expr.is_err()
                || expr
                    .clone()
                    .clone()
                    .unwrap_value()
                    .downcast_ref::<C>()
                    .is_none()
        })
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
    acceptable_range: impl RangeBounds<usize>,
) -> Result<impl Iterator<Item = T> + 'a, CellError> {
    let errors = find_errs::<T>(self_, type_err);
    if errors.len() > 0 {
        return Err(CellError::FormError(errors));
    }

    if !acceptable_range.contains(&self_.args.len()) {
        return Err(CellError::InvalidArgCount(
            to_owned_bound(acceptable_range.start_bound()),
            self_.args.len(),
        ));
    }

    Ok(self_.args.iter().map(|e| {
        e.clone()
            .unwrap_value()
            .downcast_ref::<T>()
            .unwrap()
            .clone()
    }))
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
            e.clone()
                .unwrap_value()
                .downcast_ref::<C>()
                .unwrap()
                .clone(),
        )
    }))
}

fn foldr_with_check<C: Value + Clone, T: Value>(
    self_: &mut OpInfo,
    init: T,
    f: impl Fn(T, C) -> T,
    type_err: &'static str,
    acceptable_range: impl RangeBounds<usize>,
) -> Expr {
    if !acceptable_range.contains(&self_.args.len()) {
        return Expr::Err(CellError::InvalidArgCount(
            to_owned_bound(acceptable_range.start_bound()),
            self_.args.len(),
        ));
    }

    foldr(self_, init, f, type_err)
        .map(|n| Expr::Value(Box::new(n)))
        .unwrap_or_else(|e| Expr::Err(e))
}

pub fn get_form_map<'a>() -> HashMap<&'a str, Operator> {
    let mut map = HashMap::<&str, Operator>::new();

    let sum = Box::new(|_: &mut _, info: &mut OpInfo| {
        foldr_with_check(info, Num::I(0), |acc, n| acc + n, "Num", 1..)
    });

    let mul: Operator =
        Box::new(|_, info| foldr_with_check(info, Num::I(1), |acc, n| acc * n, "Num", 1..));

    let div: Operator = Box::new(|_, info| {
        let res = expect_two::<Num>(info, "Num");
        match res {
            Ok((arg1, arg2)) => {
                if arg2 == Num::I(0) || arg2 == Num::F(0.0) {
                    return Expr::Err(CellError::DivByZero);
                }

                Expr::Value(Box::new(arg1 / arg2))
            }
            Err(e) => Expr::Err(e),
        }
    });

    let gt: Operator = Box::new(|_, info| {
        let info = dbg!(info);
        let errors = info
            .args
            .iter()
            .enumerate()
            .filter(|(_, expr)| expr.is_err())
            .map(|(ix, expr): (_, &Expr)| (ix + 1, expr.clone().unwrap_err().clone()))
            .collect::<Vec<_>>();

        if errors.len() > 0 {
            return Expr::Err(CellError::FormError(errors));
        }
        if info.args.len() != 2 {
            return Expr::Err(CellError::InvalidArgCount(Bound::Included(2), 2));
        }

        let arg1 = info.args.get(0).unwrap().clone().unwrap_value();
        let arg2 = info.args.get(1).unwrap().clone().unwrap_value();
        if arg1.type_id() != arg2.type_id() {
            return Expr::Err(CellError::BinaryTypeMismatch.into());
        }
        Expr::Value(Box::new(
            (arg1 as Box<dyn DynOrd>) > (arg2 as Box<dyn DynOrd>),
        ))
    });

    let eq: Operator = Box::new({
        |_, info| {
            let errors = info
                .args
                .iter()
                .enumerate()
                .filter(|(_, expr)| expr.is_err())
                .map(|(ix, expr): (_, &Expr)| (ix + 1, expr.clone().unwrap_err().clone()))
                .collect::<Vec<_>>();

            if errors.len() > 0 {
                return Expr::Err(CellError::FormError(errors));
            }
            if info.args.len() != 2 {
                return Expr::Err(CellError::InvalidArgCount(Bound::Included(2), 2));
            }

            let arg1 = info.args.get(0).unwrap().clone().unwrap_value();
            let arg2 = info.args.get(1).unwrap().clone().unwrap_value();
            if arg1.type_id() != arg2.type_id() {
                return Expr::Err(CellError::BinaryTypeMismatch.into());
            }
            Expr::Value(Box::new(info.args.get(0) == info.args.get(1)))
        }
    });

    let not: Operator = Box::new(|_, info| {
        let res = expect_n::<bool>(info, "Boolean", 1..=1);

        match res {
            Ok(mut arg_iter) => Expr::Value(Box::new(!arg_iter.next().unwrap())),
            Err(e) => Expr::Err(e),
        }
    });

    let and: Operator = Box::new(|_, info| {
        let res = expect_two::<bool>(info, "Boolean");
        match res {
            Ok((arg1, arg2)) => Expr::Value(Box::new(arg1 && arg2)),
            Err(e) => Expr::Err(e),
        }
    });

    let or: Operator = Box::new(|_, info| {
        let res = expect_two::<bool>(info, "Boolean");
        match res {
            Ok((arg1, arg2)) => Expr::Value(Box::new(arg1 || arg2)),
            Err(e) => Expr::Err(e),
        }
    });

    let r#if: Operator = Box::new(|_, info| {
        let errors = info
            .args
            .iter()
            .enumerate()
            .filter(|(_, expr)| expr.is_err())
            .map(|(ix, expr): (_, &Expr)| (ix + 1, expr.clone().unwrap_err().clone()))
            .collect::<Vec<_>>();

        if errors.len() > 0 {
            return Expr::Err(CellError::FormError(errors));
        }

        if info.args.len() != 3 {
            return Expr::Err(CellError::InvalidArgCount(Bound::Included(2), 2));
        }

        let binding = info.args[0].clone().unwrap_value();
        let cond = binding.downcast_ref::<bool>();
        if cond.is_none() {
            return Expr::Err(CellError::TypeMismatch("String"));
        }
        let cond = cond.unwrap();

        let arg1 = info.args[1].clone().unwrap_value();
        let arg2 = info.args[2].clone().unwrap_value();
        if arg1.type_id() != arg2.type_id() {
            return Expr::Err(CellError::BinaryTypeMismatch.into());
        }
        Expr::Value(if *cond { arg1.clone() } else { arg2.clone() })
    });

    let concat: Operator = Box::new(|_, info| {
        foldr_with_check(
            info,
            String::from(""),
            |acc, n: String| acc + &n[..],
            "String",
            0..,
        ) // Spec does not mention min args, I'll assume 0
    });

    map.insert("SUM", sum);
    map.insert("MULTIPLY", mul);
    map.insert("DIVIDE", div);
    map.insert("GT", gt);
    map.insert("EQ", eq);
    map.insert("NOT", not);
    map.insert("AND", and);
    map.insert("OR", or);
    map.insert("IF", r#if);
    map.insert("CONCAT", concat);

    map
}
