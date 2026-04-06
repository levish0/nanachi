use std::collections::HashSet;

use crate::ir::{IrExpr, IrProgram, IrRule};

pub(super) fn inline_trivial_rules(mut program: IrProgram) -> IrProgram {
    let mut referenced_by_others: HashSet<usize> = HashSet::new();
    for rule in &program.rules {
        collect_refs(&rule.expr, &mut referenced_by_others);
    }

    let inline_set: HashSet<usize> = program
        .rules
        .iter()
        .enumerate()
        .filter(|(i, rule)| is_trivial(rule) && referenced_by_others.contains(i))
        .map(|(i, _)| i)
        .collect();

    for (i, rule) in program.rules.iter_mut().enumerate() {
        if inline_set.contains(&i) {
            rule.inline = true;
            tracing::trace!(rule = %rule.name, "inline_trivial_rules: marked for inlining");
        }
    }

    let inline_exprs: Vec<Option<IrExpr>> = program
        .rules
        .iter()
        .map(|r| if r.inline { Some(r.expr.clone()) } else { None })
        .collect();

    for rule in &mut program.rules {
        rule.expr = inline_refs(rule.expr.clone(), &inline_exprs);
    }

    program
}

fn is_trivial(rule: &IrRule) -> bool {
    if !rule.guards.is_empty() || !rule.emits.is_empty() {
        return false;
    }
    matches!(
        &rule.expr,
        IrExpr::Literal(_)
            | IrExpr::CharSet(_)
            | IrExpr::Any
            | IrExpr::Boundary(_)
            | IrExpr::TakeWhile { .. }
    )
}

fn inline_refs(expr: IrExpr, inline_exprs: &[Option<IrExpr>]) -> IrExpr {
    match expr {
        IrExpr::RuleRef(idx) => {
            if let Some(Some(inlined)) = inline_exprs.get(idx) {
                inlined.clone()
            } else {
                IrExpr::RuleRef(idx)
            }
        }
        IrExpr::Seq(items) => IrExpr::Seq(
            items
                .into_iter()
                .map(|e| inline_refs(e, inline_exprs))
                .collect(),
        ),
        IrExpr::Choice(items) => IrExpr::Choice(
            items
                .into_iter()
                .map(|e| inline_refs(e, inline_exprs))
                .collect(),
        ),
        IrExpr::Repeat { expr, min, max } => IrExpr::Repeat {
            expr: Box::new(inline_refs(*expr, inline_exprs)),
            min,
            max,
        },
        IrExpr::PosLookahead(inner) => {
            IrExpr::PosLookahead(Box::new(inline_refs(*inner, inline_exprs)))
        }
        IrExpr::NegLookahead(inner) => {
            IrExpr::NegLookahead(Box::new(inline_refs(*inner, inline_exprs)))
        }
        IrExpr::WithFlag { flag, body } => IrExpr::WithFlag {
            flag,
            body: Box::new(inline_refs(*body, inline_exprs)),
        },
        IrExpr::WithCounter {
            counter,
            amount,
            body,
        } => IrExpr::WithCounter {
            counter,
            amount,
            body: Box::new(inline_refs(*body, inline_exprs)),
        },
        IrExpr::When { condition, body } => IrExpr::When {
            condition,
            body: Box::new(inline_refs(*body, inline_exprs)),
        },
        IrExpr::DepthLimit { limit, body } => IrExpr::DepthLimit {
            limit,
            body: Box::new(inline_refs(*body, inline_exprs)),
        },
        IrExpr::Labeled { expr, label } => IrExpr::Labeled {
            expr: Box::new(inline_refs(*expr, inline_exprs)),
            label,
        },
        other => other,
    }
}

pub(super) fn eliminate_dead_rules(program: IrProgram) -> IrProgram {
    program
}

fn collect_refs(expr: &IrExpr, refs: &mut HashSet<usize>) {
    match expr {
        IrExpr::RuleRef(idx) => {
            refs.insert(*idx);
        }
        IrExpr::Seq(items) | IrExpr::Choice(items) => {
            for item in items {
                collect_refs(item, refs);
            }
        }
        IrExpr::Repeat { expr, .. }
        | IrExpr::PosLookahead(expr)
        | IrExpr::NegLookahead(expr)
        | IrExpr::WithFlag { body: expr, .. }
        | IrExpr::WithCounter { body: expr, .. }
        | IrExpr::When { body: expr, .. }
        | IrExpr::DepthLimit { body: expr, .. }
        | IrExpr::Labeled { expr, .. } => {
            collect_refs(expr, refs);
        }
        _ => {}
    }
}

pub(super) fn compute_ref_counts(mut program: IrProgram) -> IrProgram {
    let mut counts = vec![0usize; program.rules.len()];
    for rule in &program.rules {
        if !rule.inline {
            count_refs(&rule.expr, &mut counts);
        }
    }
    for (i, rule) in program.rules.iter_mut().enumerate() {
        rule.ref_count = counts[i];
    }
    program
}

fn count_refs(expr: &IrExpr, counts: &mut [usize]) {
    match expr {
        IrExpr::RuleRef(idx) => {
            counts[*idx] += 1;
        }
        IrExpr::Seq(items) | IrExpr::Choice(items) => {
            for item in items {
                count_refs(item, counts);
            }
        }
        IrExpr::Repeat { expr, .. }
        | IrExpr::PosLookahead(expr)
        | IrExpr::NegLookahead(expr)
        | IrExpr::WithFlag { body: expr, .. }
        | IrExpr::WithCounter { body: expr, .. }
        | IrExpr::When { body: expr, .. }
        | IrExpr::DepthLimit { body: expr, .. }
        | IrExpr::Labeled { expr, .. } => {
            count_refs(expr, counts);
        }
        _ => {}
    }
}
