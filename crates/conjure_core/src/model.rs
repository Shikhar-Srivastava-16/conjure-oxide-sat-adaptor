use std::collections::VecDeque;
use std::fmt::{Debug, Display};
use std::sync::{Arc, RwLock};

use derivative::Derivative;
use serde::{Deserialize, Serialize};
use uniplate::{Biplate, Tree, Uniplate};

use crate::ast::{DecisionVariable, Domain, Expression, Name, SymbolTable};
use crate::context::Context;

use crate::ast::pretty::{
    pretty_domain_letting_declaration, pretty_expressions_as_top_level,
    pretty_value_letting_declaration, pretty_variable_declaration,
};

/// Represents a computational model containing variables, constraints, and a shared context.
///
/// The `Model` struct holds a set of variables and constraints for manipulating and evaluating symbolic expressions.
///
/// # Fields
/// - `constraints`:
///   - Type: `Vec<Expression>`
///   - Represents the logical constraints applied to the model's variables.
///   - Can be a single constraint or a combination of various expressions, such as logical operations (e.g., `AND`, `OR`),
///     arithmetic operations (e.g., `SafeDiv`, `UnsafeDiv`), or specialized constraints like `SumEq`.
///
/// - `context`:
///   - Type: `Arc<RwLock<Context<'static>>>`
///   - A shared object that stores global settings and state for the model.
///   - Can be safely read or changed by multiple parts of the program at the same time, making it good for multi-threaded use.
///
/// # Usage
/// This struct is typically used to:
/// - Define a set of variables and constraints for rule-based evaluation.
/// - Have transformations, optimizations, and simplifications applied to it using a set of rules.
#[derive(Derivative, Clone, Debug, Serialize, Deserialize)]
#[derivative(PartialEq, Eq)]
pub struct Model {
    pub constraints: Vec<Expression>,

    symbols: SymbolTable,

    #[serde(skip)]
    #[derivative(PartialEq = "ignore")]
    pub context: Arc<RwLock<Context<'static>>>,
}

impl Model {
    /// Creates a new model.
    pub fn new(
        symbols: SymbolTable,
        constraints: Vec<Expression>,
        context: Arc<RwLock<Context<'static>>>,
    ) -> Model {
        Model {
            symbols,
            constraints,
            context,
        }
    }

    pub fn new_empty(context: Arc<RwLock<Context<'static>>>) -> Model {
        Model::new(Default::default(), Vec::new(), context)
    }

    /// The global symbol table for this model.
    pub fn symbols(&self) -> &SymbolTable {
        &self.symbols
    }

    /// The global symbol table for this model, as a mutable reference.
    pub fn symbols_mut(&mut self) -> &mut SymbolTable {
        &mut self.symbols
    }

    // Function to update a DecisionVariable based on its Name
    pub fn update_domain(&mut self, name: &Name, new_domain: Domain) {
        if let Some(decision_var) = self.symbols_mut().get_var_mut(name) {
            decision_var.domain = new_domain;
        }
    }

    /// Gets the domain of `name` if it exists and has one.
    pub fn get_domain(&self, name: &Name) -> Option<&Domain> {
        self.symbols().domain_of(name)
    }

    /// Adds a decision variable to the model.
    ///
    /// Returns `None` if there is a decision variable or other object with that name in the symbol
    /// table.
    pub fn add_variable(&mut self, name: Name, decision_var: DecisionVariable) -> Option<()> {
        self.symbols_mut().add_var(name, decision_var)
    }

    pub fn get_constraints_vec(&self) -> Vec<Expression> {
        self.constraints.clone()
    }

    pub fn set_constraints(&mut self, constraints: Vec<Expression>) {
        if constraints.is_empty() {
            self.constraints = Vec::new();
        } else {
            self.constraints = constraints;
        }
    }

    pub fn set_context(&mut self, context: Arc<RwLock<Context<'static>>>) {
        self.context = context;
    }

    pub fn add_constraint(&mut self, expression: Expression) {
        // ToDo (gs248) - there is no checking whatsoever
        // We need to properly validate the expression but this is just for testing
        let mut constraints = self.get_constraints_vec();
        constraints.push(expression);
        self.set_constraints(constraints);
    }

    pub fn add_constraints(&mut self, expressions: Vec<Expression>) {
        let mut constraints = self.get_constraints_vec();
        constraints.extend(expressions);
        self.set_constraints(constraints);
    }

    /// Returns an arbitrary variable name that is not in the model.
    pub fn gensym(&self) -> Name {
        self.symbols().gensym()
    }

    /// Extends the models symbol table with the given symbol table, updating the gensym counter if
    /// necessary.
    pub fn extend_sym_table(&mut self, other: SymbolTable) {
        self.symbols_mut().extend(other);
    }
}

// At time of writing (03/02/2025), the Uniplate derive macro doesn't like the lifetimes inside
// context, and we do not yet have a way of ignoring this field.
impl Uniplate for Model {
    fn uniplate(&self) -> (Tree<Self>, Box<dyn Fn(Tree<Self>) -> Self>) {
        // Model contains no sub-models.
        let self2 = self.clone();
        (Tree::Zero, Box::new(move |_| self2.clone()))
    }
}

// TODO: replace with derive macro when possible.
impl Biplate<Expression> for Model {
    fn biplate(&self) -> (Tree<Expression>, Box<dyn Fn(Tree<Expression>) -> Self>) {
        let (symtab_tree, symtab_ctx) = self.symbols.biplate();
        let (constraints_tree, constraints_ctx) = self.constraints.biplate();

        let tree = Tree::Many(VecDeque::from([symtab_tree, constraints_tree]));

        let self2 = self.clone();
        let ctx = Box::new(move |tree| {
            let Tree::Many(fields) = tree else {
                panic!("number of children changed!");
            };

            let mut self3 = self2.clone();
            self3.symbols = (symtab_ctx)(fields[0].clone());
            self3.constraints = (constraints_ctx)(fields[1].clone());
            self3
        });

        (tree, ctx)
    }
}

impl Display for Model {
    #[allow(clippy::unwrap_used)] // [rustdocs]: should only fail iff the formatter fails
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (name, _) in self.symbols().iter_value_letting() {
            writeln!(
                f,
                "{}",
                pretty_value_letting_declaration(self.symbols(), name).unwrap()
            )?;
        }
        for (name, _) in self.symbols().iter_domain_letting() {
            writeln!(
                f,
                "{}",
                pretty_domain_letting_declaration(self.symbols(), name).unwrap()
            )?;
        }

        for (name, _) in self.symbols().iter_var() {
            writeln!(
                f,
                "find {}",
                pretty_variable_declaration(self.symbols(), name).unwrap()
            )?;
        }

        writeln!(f, "\nsuch that\n")?;

        writeln!(f, "{}", pretty_expressions_as_top_level(&self.constraints))?;

        Ok(())
    }
}
