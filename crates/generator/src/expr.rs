use nanachi_meta::ast::*;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};

pub(crate) fn rule_fn_ident(name: &str, detailed: bool) -> Ident {
    if detailed {
        format_ident!("__nanachi_detailed_{}", name)
    } else {
        format_ident!("{}", name)
    }
}

/// Generate winnow combinator code for an expression.
pub(crate) fn generate_expr(expr: &Expr, detailed: bool) -> TokenStream {
    match expr {
        Expr::StringLit(s) => {
            if detailed {
                quote! {
                    literal(#s).context(StrContext::Expected(StrContextValue::StringLiteral(#s)))
                }
            } else {
                quote! { literal(#s) }
            }
        }

        Expr::CharRange(start, end) => {
            let desc = format!("'{}'..'{}'", start, end);
            if detailed {
                quote! {
                    one_of(#start..=#end)
                        .context(StrContext::Expected(StrContextValue::Description(#desc)))
                }
            } else {
                quote! { one_of(#start..=#end) }
            }
        }

        Expr::Ident(name) => {
            let fn_name = rule_fn_ident(name, detailed);
            quote! { #fn_name }
        }

        Expr::Builtin(builtin) => generate_builtin_expr(builtin),

        Expr::Seq(exprs) => {
            let items: Vec<_> = exprs
                .iter()
                .map(|expr| generate_expr(expr, detailed))
                .collect();
            quote! { (#(#items),*) }
        }

        Expr::Choice(exprs) => {
            let items: Vec<_> = exprs
                .iter()
                .map(|e| {
                    let code = generate_expr(e, detailed);
                    quote! { (#code).void() }
                })
                .collect();
            generate_alt(items)
        }

        Expr::Repeat { expr, kind } => generate_repeat(expr, kind, detailed),

        Expr::PosLookahead(inner) => {
            let inner_code = generate_expr(inner, detailed);
            quote! { peek(#inner_code) }
        }

        Expr::NegLookahead(inner) => {
            let inner_code = generate_expr(inner, detailed);
            quote! { not(#inner_code) }
        }

        Expr::Group(inner) => generate_expr(inner, detailed),

        Expr::With(with_expr) => generate_with_flag(with_expr, detailed),
        Expr::WithIncrement(with_inc) => generate_with_increment(with_inc, detailed),
        Expr::When(when_expr) => generate_when(when_expr, detailed),
        Expr::DepthLimit(dl) => generate_depth_limit(dl, detailed),
    }
}

fn generate_builtin_expr(builtin: &BuiltinPredicate) -> TokenStream {
    match builtin {
        BuiltinPredicate::Soi => {
            // SOI as expression: succeed if at position 0
            quote! {
                winnow::combinator::trace("SOI", |input: &mut Input<'_, ParseState>| {
                    if input.current_token_start() == 0 {
                        Ok(())
                    } else {
                        Err(winnow::error::ErrMode::Backtrack(
                            winnow::error::ContextError::new(),
                        ))
                    }
                })
            }
        }
        BuiltinPredicate::Eoi => {
            quote! { eof.void() }
        }
        BuiltinPredicate::Any => {
            quote! { any.void() }
        }
        BuiltinPredicate::LineStart => {
            quote! {
                winnow::combinator::trace("LINE_START", |input: &mut Input<'_, ParseState>| {
                    let pos = input.current_token_start();
                    if input.state.is_at_line_start(pos) {
                        Ok(())
                    } else {
                        Err(winnow::error::ErrMode::Backtrack(
                            winnow::error::ContextError::new(),
                        ))
                    }
                })
            }
        }
        BuiltinPredicate::LineEnd => {
            quote! {
                winnow::combinator::trace("LINE_END", |input: &mut Input<'_, ParseState>| {
                    let pos = input.current_token_start();
                    if input.state.is_at_line_end(pos) {
                        Ok(())
                    } else {
                        Err(winnow::error::ErrMode::Backtrack(
                            winnow::error::ContextError::new(),
                        ))
                    }
                })
            }
        }
    }
}

fn generate_repeat(expr: &Expr, kind: &RepeatKind, detailed: bool) -> TokenStream {
    let inner = generate_expr(expr, detailed);
    // All repeats collect into () since we .void() at rule level.
    // Use fold to avoid Accumulate<()> ambiguity.
    let fold = quote! { .fold(|| (), |(), _| ()) };
    match kind {
        RepeatKind::ZeroOrMore => quote! { repeat(0.., #inner)#fold },
        RepeatKind::OneOrMore => quote! { repeat(1.., #inner)#fold },
        RepeatKind::Optional => quote! { opt(#inner) },
        RepeatKind::Exact(n) => {
            let n = *n as usize;
            quote! { repeat(#n, #inner)#fold }
        }
        RepeatKind::AtLeast(n) => {
            let n = *n as usize;
            quote! { repeat(#n.., #inner)#fold }
        }
        RepeatKind::AtMost(m) => {
            let m = *m as usize;
            quote! { repeat(..=#m, #inner)#fold }
        }
        RepeatKind::Range(n, m) => {
            let n = *n as usize;
            let m = *m as usize;
            quote! { repeat(#n..=#m, #inner)#fold }
        }
    }
}

fn generate_with_flag(with_expr: &WithExpr, detailed: bool) -> TokenStream {
    let name = &with_expr.flag;
    let body = generate_expr(&with_expr.body, detailed);
    quote! {
        winnow::combinator::trace("with_flag", |input: &mut Input<'_, ParseState>| {
            let prev = input.state.get_flag(#name);
            input.state.set_flag(#name, true);
            let result = (#body).void().parse_next(input);
            input.state.set_flag(#name, prev);
            result
        })
    }
}

fn generate_with_increment(with_inc: &WithIncrementExpr, detailed: bool) -> TokenStream {
    let name = &with_inc.counter;
    let amount = with_inc.amount as usize;
    let body = generate_expr(&with_inc.body, detailed);
    quote! {
        winnow::combinator::trace("with_increment", |input: &mut Input<'_, ParseState>| {
            input.state.increment_counter(#name, #amount);
            let result = (#body).void().parse_next(input);
            input.state.decrement_counter(#name, #amount);
            result
        })
    }
}

fn generate_when(when_expr: &WhenExpr, detailed: bool) -> TokenStream {
    let condition_check = generate_condition_check(&when_expr.condition);
    let body = generate_expr(&when_expr.body, detailed);
    quote! {
        winnow::combinator::trace("when", |input: &mut Input<'_, ParseState>| {
            if #condition_check {
                (#body).void().parse_next(input)
            } else {
                Ok(())
            }
        })
    }
}

fn generate_condition_check(condition: &GuardCondition) -> TokenStream {
    match condition {
        GuardCondition::NotFlag(name) => quote! { !input.state.get_flag(#name) },
        GuardCondition::IsFlag(name) => quote! { input.state.get_flag(#name) },
        GuardCondition::Builtin(BuiltinPredicate::Soi) => {
            quote! { input.current_token_start() == 0 }
        }
        GuardCondition::Builtin(BuiltinPredicate::Eoi) => quote! { input.input.is_empty() },
        GuardCondition::Builtin(BuiltinPredicate::LineStart) => {
            quote! { input.state.is_at_line_start(input.current_token_start()) }
        }
        GuardCondition::Builtin(BuiltinPredicate::LineEnd) => {
            quote! { input.state.is_at_line_end(input.current_token_start()) }
        }
        GuardCondition::Builtin(BuiltinPredicate::Any) => quote! { true },
        GuardCondition::Compare { name, op, value } => {
            let value = *value as usize;
            let cmp = match op {
                CompareOp::Eq => quote! { == },
                CompareOp::Ne => quote! { != },
                CompareOp::Lt => quote! { < },
                CompareOp::Le => quote! { <= },
                CompareOp::Gt => quote! { > },
                CompareOp::Ge => quote! { >= },
            };
            quote! { input.state.get_counter(#name) #cmp #value }
        }
    }
}

fn generate_depth_limit(dl: &DepthLimitExpr, detailed: bool) -> TokenStream {
    let limit = dl.limit as usize;
    let body = generate_expr(&dl.body, detailed);
    quote! {
        winnow::combinator::trace("depth_limit", |input: &mut Input<'_, ParseState>| {
            let depth = input.state.get_counter("__recursion_depth");
            if depth >= #limit {
                return Err(winnow::error::ErrMode::Backtrack(
                    winnow::error::ContextError::new(),
                ));
            }
            input.state.increment_counter("__recursion_depth", 1);
            let result = (#body).void().parse_next(input);
            input.state.decrement_counter("__recursion_depth", 1);
            result
        })
    }
}

/// Generate nested `alt()` calls, chunking into groups of 21 to stay within
/// winnow's tuple size limit.
fn generate_alt(items: Vec<TokenStream>) -> TokenStream {
    const MAX: usize = 21;
    if items.len() <= MAX {
        quote! { alt((#(#items),*)) }
    } else {
        let chunks: Vec<_> = items
            .chunks(MAX)
            .map(|chunk| {
                let chunk = chunk.to_vec();
                quote! { alt((#(#chunk),*)) }
            })
            .collect();
        generate_alt(chunks)
    }
}
