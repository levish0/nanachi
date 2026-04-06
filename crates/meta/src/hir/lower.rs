//! AST → IR lowering pass.
//!
//! Resolves rule references to indices, removes syntactic wrappers,
//! unifies repeat variants, and converts statements into the IR structure.

use std::collections::HashMap;

use super::{Boundary, CharRange, IrExpr, IrProgram, IrRule};
use crate::ast::{self, BuiltinPredicate, Expr, Grammar, Item, RepeatKind, Statement};

/// Lower a validated AST Grammar to an IR Program.
///
/// Panics if the grammar contains unresolved rule references
/// (should not happen after validation).
#[tracing::instrument(skip_all, fields(rules = grammar.items.iter().filter(|i| matches!(i, Item::RuleDef(_))).count()))]
pub fn lower(grammar: &Grammar) -> IrProgram {
    // Build name → index map.
    let rule_indices: HashMap<&str, usize> = grammar
        .items
        .iter()
        .filter_map(|item| match item {
            Item::RuleDef(r) => Some(r.name.as_str()),
            _ => None,
        })
        .enumerate()
        .map(|(i, name)| (name, i))
        .collect();

    let state_decls: Vec<_> = grammar
        .items
        .iter()
        .filter_map(|item| match item {
            Item::StateDecl(decl) => Some(decl.clone()),
            _ => None,
        })
        .collect();

    let rules: Vec<_> = grammar
        .items
        .iter()
        .filter_map(|item| match item {
            Item::RuleDef(rule) => {
                let ir_rule = lower_rule(rule, &rule_indices);
                tracing::trace!(
                    rule = %ir_rule.name,
                    guards = ir_rule.guards.len(),
                    emits = ir_rule.emits.len(),
                    has_error_label = ir_rule.error_label.is_some(),
                    "lowered rule"
                );
                Some(ir_rule)
            }
            _ => None,
        })
        .collect();

    IrProgram { state_decls, rules }
}

fn lower_rule(rule: &ast::RuleDef, indices: &HashMap<&str, usize>) -> IrRule {
    let mut guards = Vec::new();
    let mut emits = Vec::new();

    for stmt in &rule.body.statements {
        match stmt {
            Statement::Guard(g) => guards.push(g.condition.clone()),
            Statement::Emit(e) => emits.push(e.counter.clone()),
        }
    }

    let expr = lower_expr(&rule.body.expr, indices);

    IrRule {
        name: rule.name.clone(),
        inline: false,
        error_label: rule.error_label.clone(),
        guards,
        emits,
        expr,
        ref_count: 0,
    }
}

fn lower_expr(expr: &Expr, indices: &HashMap<&str, usize>) -> IrExpr {
    match expr {
        Expr::StringLit(s) => IrExpr::Literal(s.clone()),

        Expr::CharRange(start, end) => IrExpr::CharSet(vec![CharRange::new(*start, *end)]),

        Expr::Ident(name) => {
            let index = indices[name.as_str()];
            IrExpr::RuleRef(index)
        }

        Expr::Builtin(builtin) => lower_builtin(builtin),

        Expr::Seq(exprs) => {
            let items: Vec<_> = exprs.iter().map(|e| lower_expr(e, indices)).collect();
            if items.len() == 1 {
                items.into_iter().next().unwrap()
            } else {
                IrExpr::Seq(items)
            }
        }

        Expr::Choice(exprs) => {
            let items: Vec<_> = exprs.iter().map(|e| lower_expr(e, indices)).collect();
            if items.len() == 1 {
                items.into_iter().next().unwrap()
            } else {
                IrExpr::Choice(items)
            }
        }

        Expr::Repeat { expr, kind } => {
            let (min, max) = match kind {
                RepeatKind::ZeroOrMore => (0, None),
                RepeatKind::OneOrMore => (1, None),
                RepeatKind::Optional => (0, Some(1)),
                RepeatKind::Exact(n) => (*n, Some(*n)),
                RepeatKind::AtLeast(n) => (*n, None),
                RepeatKind::AtMost(m) => (0, Some(*m)),
                RepeatKind::Range(n, m) => (*n, Some(*m)),
            };
            IrExpr::Repeat {
                expr: Box::new(lower_expr(expr, indices)),
                min,
                max,
            }
        }

        Expr::PosLookahead(inner) => IrExpr::PosLookahead(Box::new(lower_expr(inner, indices))),

        Expr::NegLookahead(inner) => IrExpr::NegLookahead(Box::new(lower_expr(inner, indices))),

        // Group is purely syntactic — unwrap.
        Expr::Group(inner) => lower_expr(inner, indices),

        Expr::Labeled { expr, label } => IrExpr::Labeled {
            expr: Box::new(lower_expr(expr, indices)),
            label: label.clone(),
        },

        Expr::With(w) => IrExpr::WithFlag {
            flag: w.flag.clone(),
            body: Box::new(lower_expr(&w.body, indices)),
        },

        Expr::WithIncrement(w) => IrExpr::WithCounter {
            counter: w.counter.clone(),
            amount: w.amount,
            body: Box::new(lower_expr(&w.body, indices)),
        },

        Expr::When(w) => IrExpr::When {
            condition: w.condition.clone(),
            body: Box::new(lower_expr(&w.body, indices)),
        },

        Expr::DepthLimit(d) => IrExpr::DepthLimit {
            limit: d.limit,
            body: Box::new(lower_expr(&d.body, indices)),
        },
    }
}

fn lower_builtin(builtin: &BuiltinPredicate) -> IrExpr {
    match builtin {
        BuiltinPredicate::Soi => IrExpr::Boundary(Boundary::Soi),
        BuiltinPredicate::Eoi => IrExpr::Boundary(Boundary::Eoi),
        BuiltinPredicate::Any => IrExpr::Any,
        BuiltinPredicate::LineStart => IrExpr::Boundary(Boundary::LineStart),
        BuiltinPredicate::LineEnd => IrExpr::Boundary(Boundary::LineEnd),
    }
}

#[cfg(test)]
#[path = "lower/tests.rs"]
mod tests;
