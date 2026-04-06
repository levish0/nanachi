use crate::ir::{CharRange, DispatchArm, IrExpr, IrProgram, IrRule};

pub(super) fn recognize_dispatch(mut program: IrProgram) -> IrProgram {
    let snapshot = program.rules.clone();
    for rule in &mut program.rules {
        let before = rule.expr.clone();
        rule.expr = recognize_dispatch_expr(rule.expr.clone(), &snapshot);
        if rule.expr != before {
            tracing::trace!(rule = %rule.name, "recognize_dispatch: lowered choice");
        }
    }
    program
}

fn recognize_dispatch_expr(expr: IrExpr, rules: &[IrRule]) -> IrExpr {
    match expr {
        IrExpr::Seq(items) => IrExpr::Seq(
            items
                .into_iter()
                .map(|item| recognize_dispatch_expr(item, rules))
                .collect(),
        ),
        IrExpr::Choice(items) => {
            let items: Vec<_> = items
                .into_iter()
                .map(|item| recognize_dispatch_expr(item, rules))
                .collect();
            if let Some(arms) = build_dispatch_arms(&items, rules) {
                IrExpr::Dispatch(arms)
            } else {
                IrExpr::Choice(items)
            }
        }
        IrExpr::Dispatch(arms) => IrExpr::Dispatch(
            arms.into_iter()
                .map(|arm| DispatchArm {
                    ranges: arm.ranges,
                    expr: Box::new(recognize_dispatch_expr(*arm.expr, rules)),
                })
                .collect(),
        ),
        IrExpr::Repeat { expr, min, max } => IrExpr::Repeat {
            expr: Box::new(recognize_dispatch_expr(*expr, rules)),
            min,
            max,
        },
        IrExpr::PosLookahead(inner) => {
            IrExpr::PosLookahead(Box::new(recognize_dispatch_expr(*inner, rules)))
        }
        IrExpr::NegLookahead(inner) => {
            IrExpr::NegLookahead(Box::new(recognize_dispatch_expr(*inner, rules)))
        }
        IrExpr::WithFlag { flag, body } => IrExpr::WithFlag {
            flag,
            body: Box::new(recognize_dispatch_expr(*body, rules)),
        },
        IrExpr::WithCounter {
            counter,
            amount,
            body,
        } => IrExpr::WithCounter {
            counter,
            amount,
            body: Box::new(recognize_dispatch_expr(*body, rules)),
        },
        IrExpr::When { condition, body } => IrExpr::When {
            condition,
            body: Box::new(recognize_dispatch_expr(*body, rules)),
        },
        IrExpr::DepthLimit { limit, body } => IrExpr::DepthLimit {
            limit,
            body: Box::new(recognize_dispatch_expr(*body, rules)),
        },
        IrExpr::Labeled { expr, label } => IrExpr::Labeled {
            expr: Box::new(recognize_dispatch_expr(*expr, rules)),
            label,
        },
        other => other,
    }
}

#[derive(Debug, Clone)]
struct FirstChars {
    ranges: Vec<CharRange>,
    nullable: bool,
}

fn build_dispatch_arms(items: &[IrExpr], rules: &[IrRule]) -> Option<Vec<DispatchArm>> {
    let mut arms = Vec::with_capacity(items.len());
    let mut seen: Vec<CharRange> = Vec::new();

    for item in items {
        let first = first_chars(item, rules, &mut Vec::new())?;
        if first.nullable || first.ranges.is_empty() {
            return None;
        }

        let ranges = super::coalesce_ranges(first.ranges);
        if ranges_overlap_any(&ranges, &seen) {
            return None;
        }
        seen.extend(ranges.iter().copied());

        arms.push(DispatchArm {
            ranges,
            expr: Box::new(item.clone()),
        });
    }

    Some(arms)
}

fn first_chars(expr: &IrExpr, rules: &[IrRule], visiting: &mut Vec<usize>) -> Option<FirstChars> {
    match expr {
        IrExpr::Literal(s) => {
            let mut chars = s.chars();
            if let Some(ch) = chars.next() {
                Some(FirstChars {
                    ranges: vec![CharRange::single(ch)],
                    nullable: false,
                })
            } else {
                Some(FirstChars {
                    ranges: Vec::new(),
                    nullable: true,
                })
            }
        }
        IrExpr::CharSet(ranges) => Some(FirstChars {
            ranges: super::coalesce_ranges(ranges.clone()),
            nullable: false,
        }),
        IrExpr::Any => Some(FirstChars {
            ranges: vec![CharRange::new(char::MIN, char::MAX)],
            nullable: false,
        }),
        IrExpr::Boundary(_) => Some(FirstChars {
            ranges: Vec::new(),
            nullable: true,
        }),
        IrExpr::RuleRef(idx) => {
            if visiting.contains(idx) {
                return None;
            }
            visiting.push(*idx);
            let result = first_chars(&rules[*idx].expr, rules, visiting);
            visiting.pop();
            result
        }
        IrExpr::Seq(items) => {
            let mut ranges = Vec::new();
            let mut nullable = true;

            for item in items {
                let first = first_chars(item, rules, visiting)?;
                ranges.extend(first.ranges);
                if !first.nullable {
                    nullable = false;
                    break;
                }
            }

            Some(FirstChars {
                ranges: super::coalesce_ranges(ranges),
                nullable,
            })
        }
        IrExpr::Choice(items) => {
            let mut ranges = Vec::new();
            let mut nullable = false;
            for item in items {
                let first = first_chars(item, rules, visiting)?;
                ranges.extend(first.ranges);
                nullable |= first.nullable;
            }
            Some(FirstChars {
                ranges: super::coalesce_ranges(ranges),
                nullable,
            })
        }
        IrExpr::Dispatch(arms) => {
            let mut ranges = Vec::new();
            for arm in arms {
                ranges.extend(arm.ranges.iter().copied());
            }
            Some(FirstChars {
                ranges: super::coalesce_ranges(ranges),
                nullable: false,
            })
        }
        IrExpr::Repeat { expr, min, .. } => {
            let first = first_chars(expr, rules, visiting)?;
            Some(FirstChars {
                ranges: first.ranges,
                nullable: *min == 0 || first.nullable,
            })
        }
        IrExpr::PosLookahead(inner) => {
            let first = first_chars(inner, rules, visiting)?;
            Some(FirstChars {
                ranges: first.ranges,
                nullable: true,
            })
        }
        IrExpr::NegLookahead(_) => None,
        IrExpr::WithFlag { body, .. }
        | IrExpr::WithCounter { body, .. }
        | IrExpr::DepthLimit { body, .. }
        | IrExpr::Labeled { expr: body, .. } => first_chars(body, rules, visiting),
        IrExpr::When { .. } => None,
        IrExpr::TakeWhile { ranges, min, .. } => Some(FirstChars {
            ranges: super::coalesce_ranges(ranges.clone()),
            nullable: *min == 0,
        }),
    }
}

fn ranges_overlap_any(left: &[CharRange], right: &[CharRange]) -> bool {
    left.iter()
        .any(|l| right.iter().any(|r| ranges_overlap(*l, *r)))
}

fn ranges_overlap(left: CharRange, right: CharRange) -> bool {
    left.start <= right.end && right.start <= left.end
}
