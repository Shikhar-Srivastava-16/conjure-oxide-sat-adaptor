use conjure_core::{
    ast::{Constant as Const, Expression as Expr},
    metadata::Metadata,
    rule::{ApplicationError, ApplicationResult, Reduction},
};
use conjure_rules::register_rule;

/************************************************************************/
/*        Rules for translating to Minion-supported constraints         */
/************************************************************************/

fn is_nested_sum(exprs: &Vec<Expr>) -> bool {
    for e in exprs {
        if let Expr::Sum(_) = e {
            return true;
        }
    }
    false
}

/**
 * Helper function to get the vector of expressions from a sum (or error if it's a nested sum - we need to flatten it first)
 */
fn sum_to_vector(expr: &Expr) -> Result<Vec<Expr>, ApplicationError> {
    match expr {
        Expr::Sum(exprs) => {
            if is_nested_sum(exprs) {
                Err(ApplicationError::RuleNotApplicable)
            } else {
                Ok(exprs.clone())
            }
        }
        _ => Err(ApplicationError::RuleNotApplicable),
    }
}

/**
 * Convert a Geq to a SumGeq if the left hand side is a sum:
 * ```text
 * sum([a, b, c]) >= d => sum_geq([a, b, c], d)
 * ```
 */
#[register_rule]
fn flatten_sum_geq(expr: &Expr) -> ApplicationResult {
    match expr {
        Expr::Geq(a, b) => {
            let exprs = sum_to_vector(a)?;
            Ok(Reduction::pure(Expr::SumGeq(exprs, b.clone())))
        }
        _ => Err(ApplicationError::RuleNotApplicable),
    }
}

/**
 * Convert a Leq to a SumLeq if the left hand side is a sum:
 * ```text
 * sum([a, b, c]) <= d => sum_leq([a, b, c], d)
 * ```
 */
#[register_rule]
fn sum_leq_to_sumleq(expr: &Expr) -> ApplicationResult {
    match expr {
        Expr::Leq(a, b) => {
            let exprs = sum_to_vector(a)?;
            Ok(Reduction::pure(Expr::SumLeq(exprs, b.clone())))
        }
        _ => Err(ApplicationError::RuleNotApplicable),
    }
}

/**
 * Convert a 'Eq(Sum([...]))' to a SumEq
 * ```text
 * eq(sum([a, b]), c) => sumeq([a, b], c)
 * ```
*/
#[register_rule]
fn sum_eq_to_sumeq(expr: &Expr) -> ApplicationResult {
    match expr {
        Expr::Eq(a, b) => {
            let exprs = sum_to_vector(a)?;
            Ok(Reduction::pure(Expr::SumEq(exprs, b.clone())))
        }
        _ => Err(ApplicationError::RuleNotApplicable),
    }
}

/**
 * Convert a `SumEq` to an `And(SumGeq, SumLeq)`
 * This is a workaround for Minion not having support for a flat "equals" operation on sums
 * ```text
 * sumeq([a, b], c) -> watched_and({
 *   sumleq([a, b], c),
 *   sumgeq([a, b], c)
 * })
 * ```
 * I. e.
 * ```text
 * ((a + b) >= c) && ((a + b) <= c)
 * a + b = c
 * ```
 */
#[register_rule]
fn sumeq_to_minion(expr: &Expr) -> ApplicationResult {
    match expr {
        Expr::SumEq(exprs, eq_to) => Ok(Reduction::pure(Expr::And(vec![
            Expr::SumGeq(exprs.clone(), Box::from(*eq_to.clone())),
            Expr::SumLeq(exprs.clone(), Box::from(*eq_to.clone())),
        ]))),
        _ => Err(ApplicationError::RuleNotApplicable),
    }
}

/**
* Convert a Lt to an Ineq:

* ```text
* a < b => a - b < -1
* ```
*/
#[register_rule]
fn lt_to_ineq(expr: &Expr) -> ApplicationResult {
    match expr {
        Expr::Lt(a, b) => Ok(Reduction::pure(Expr::Ineq(
            a.clone(),
            b.clone(),
            Box::new(Expr::Constant(Metadata::new(), Const::Int(-1))),
        ))),
        _ => Err(ApplicationError::RuleNotApplicable),
    }
}

/**
* Convert a Gt to an Ineq:
*
* ```text
* a > b => b - a < -1
* ```
*/
#[register_rule]
fn gt_to_ineq(expr: &Expr) -> ApplicationResult {
    match expr {
        Expr::Gt(a, b) => Ok(Reduction::pure(Expr::Ineq(
            b.clone(),
            a.clone(),
            Box::new(Expr::Constant(Metadata::new(), Const::Int(-1))),
        ))),
        _ => Err(ApplicationError::RuleNotApplicable),
    }
}

/**
* Convert a Geq to an Ineq:
*
* ```text
* a >= b => b - a < 0
* ```
*/
#[register_rule]
fn geq_to_ineq(expr: &Expr) -> ApplicationResult {
    match expr {
        Expr::Geq(a, b) => Ok(Reduction::pure(Expr::Ineq(
            b.clone(),
            a.clone(),
            Box::new(Expr::Constant(Metadata::new(), Const::Int(0))),
        ))),
        _ => Err(ApplicationError::RuleNotApplicable),
    }
}

/**
* Convert a Leq to an Ineq:
*
* ```text
* a <= b => a - b < 0
* ```
*/
#[register_rule]
fn leq_to_ineq(expr: &Expr) -> ApplicationResult {
    match expr {
        Expr::Leq(a, b) => Ok(Reduction::pure(Expr::Ineq(
            a.clone(),
            b.clone(),
            Box::new(Expr::Constant(Metadata::new(), Const::Int(0))),
        ))),
        _ => Err(ApplicationError::RuleNotApplicable),
    }
}
