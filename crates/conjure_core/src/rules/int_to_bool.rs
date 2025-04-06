/*****************************************************************************************************/
/*        This file contains rules for converting integer expressions to logical expressions         */
/*****************************************************************************************************/

use conjure_core::rule_engine::register_rule_set;
use conjure_core::solver::SolverFamily;

use conjure_core::ast::Expression as Expr;
use conjure_core::metadata::Metadata;
use conjure_core::rule_engine::{
    register_rule, ApplicationError::RuleNotApplicable, ApplicationResult, Reduction,
};

use Expr::*;

use crate::ast::{DecisionVariable, Domain, SymbolTable, Range, Declaration, Name};
use crate::matrix_expr;

register_rule_set!("IntBool", ("Base", "CNF"), (SolverFamily::SAT));

#[register_rule(("IntBool", 4100))]
fn integer_to_boolean_vars(expr: &Expr, symbols: &SymbolTable) -> ApplicationResult {
    let Expr::Imply(_, x, y) = expr else {
        return Err(RuleNotApplicable);
    };
// TODO: Make this only run once, and remove/ignore non boolean integers afterwards
    let new_symbol_table = SymbolTable::new();
    let new_constraints: Vec<Expr> = Vec::new();

    for symbol in symbols.into_iter() {
        let name = symbol.0;

        if let DeclarationKind::DecisionVariable = symbol.1.kind() {
            if let Some(Domain::IntDomain(ranges)) = symbol.1.domain() {
                if ranges.iter().all(|x| matches!(x, Range::Bounded(_, _))) {
                    (new_expr, new_symbols) = bounded_ranges_to_symbol_table(ranges, symbols);
                    new_constraints.push(new_expr);
                    new_symbol_table.extend(new_symbols);
                }
                else {
                    return Err(RuleNotApplicable)
                }
            }
        }
    }

    symbols.Ok(Reduction::new()
}

fn bounded_ranges_to_symbol_table(ranges: Vec<Range<i32>>,global_symbols: &SymbolTable) -> (Expr, SymbolTable){
    let symbols = SymbolTable::new();
    
    highest_val = 0;
    for range in ranges{
        Range::Bounded(low, high) = range;
        if (low<0 || high < 0){
            return Err(RuleNotApplicable) // TODO: Support negative ranges
        }
        highest_val = max(highest_val, high)
    }
    let num_bits = if value == 0 { 1 } else { 32 - value.leading_zeros() };

    // generate a new decision variable for each bit
    let bits: Vec<Name> = Vec::new(); // holds
    for i in 0..bits{
    let name = global_symbols.gensym();
    symbols.insert(Decleration::new_var(name,BoolDomain));
    // TODO: store vars in format best for generating the expressions
    bits.push(symbols.lookup(&name).unwrap()) // store bits in list for generating expressions 
}

    // TODO: create expression for each range for the bounds

    (Expr::Or(_, _), SymbolTable::new())
}