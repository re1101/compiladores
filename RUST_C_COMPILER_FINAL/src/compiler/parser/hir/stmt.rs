use crate::compiler::common::token::Token;
use crate::compiler::parser::hir::{decl::*, expr::*};

use super::expr::PrintIndent;

#[derive(PartialEq, Clone, Debug)]
pub enum Stmt {
    Declaration(Declaration),
    Expr(ExprKind),
    Block(Vec<Stmt>),
    If(Token, ExprKind, Box<Stmt>, Option<Box<Stmt>>),
    While(Token, ExprKind, Box<Stmt>),
    Do(Token, Box<Stmt>, ExprKind),
    For(
        Token,
        Option<Box<Stmt>>,
        Option<ExprKind>,
        Option<ExprKind>,
        Box<Stmt>,
    ),
    Return(Token, Option<ExprKind>),
    Break(Token),
    Continue(Token),
    Switch(Token, ExprKind, Box<Stmt>),
    Case(Token, ExprKind, Box<Stmt>),
    Default(Token, Box<Stmt>),
    Goto(Token),
    Label(Token, Box<Stmt>),
}

// provides printable default for empty iterator produced string
pub trait OrEmpty {
    fn or_empty(self, indent_level: usize) -> String;
}

impl OrEmpty for String {
    fn or_empty(self, indent_level: usize) -> String {
        if self.is_empty() {
            let indent = "-".repeat(indent_level);
            format!("{}Empty", indent)
        } else {
            self
        }
    }
}

impl PrintIndent for Stmt {
    fn print_indent(&self, indent_level: usize) -> String {
        match self {
            Stmt::Expr(expr) => format!("Expr:\n{}", indent_fmt(expr, indent_level + 1)),
            Stmt::Declaration(decl) => decl.print_indent(indent_level + 1),
            Stmt::Block(body) => {
                let body = body
                    .iter()
                    .map(|s| indent_fmt(s, indent_level + 1))
                    .collect::<Vec<String>>()
                    .join("\n")
                    .or_empty(indent_level + 1);

                format!("Block:\n{}", body)
            }
            Stmt::If(_, cond, then, else_branch) => format!(
                "If:\n{}\n{}{}",
                indent_fmt(cond, indent_level + 1),
                indent_fmt(then.as_ref(), indent_level + 1),
                display_option(else_branch.as_ref().map(|t| t.as_ref()), indent_level + 1, true)
            ),
            Stmt::While(_, cond, body) => format!(
                "While:\n{}\n{}",
                indent_fmt(cond, indent_level + 1),
                indent_fmt(body.as_ref(), indent_level + 1)
            ),
            Stmt::Do(_, body, cond) => format!(
                "Do:\n{}\n{}",
                indent_fmt(cond, indent_level + 1),
                indent_fmt(body.as_ref(), indent_level + 1)
            ),
            Stmt::For(_, init, cond, inc, body) => format!(
                "For:{}{}{}\n{}",
                display_option(init.as_ref().map(|t| t.as_ref()), indent_level + 1, true),
                display_option(cond.as_ref(), indent_level + 1, true),
                display_option(inc.as_ref(), indent_level + 1, true),
                indent_fmt(body.as_ref(), indent_level + 1)
            ),
            Stmt::Return(_, expr) => {
                let mut expr = display_option(expr.as_ref(), indent_level + 1, true);
                if !expr.is_empty() {
                    expr.insert(0, ':');
                }
                format!("Return{}", expr)
            }
            Stmt::Break(_) => "Break".to_string(),
            Stmt::Continue(_) => "Continue".to_string(),
            Stmt::Switch(_, cond, body) => format!(
                "Switch:\n{}\n{}",
                indent_fmt(cond, indent_level + 1),
                indent_fmt(body.as_ref(), indent_level + 1)
            ),
            Stmt::Case(_, value, body) => format!(
                "Case:\n{}\n{}",
                indent_fmt(value, indent_level + 1),
                indent_fmt(body.as_ref(), indent_level + 1)
            ),
            Stmt::Default(_, body) => {
                format!("Default:\n{}", indent_fmt(body.as_ref(), indent_level + 1))
            }
            Stmt::Goto(t) => format!("Goto: '{}'", t.unwrap_string()),
            Stmt::Label(t, body) => format!(
                "Label: '{}'\n{}",
                t.unwrap_string(),
                indent_fmt(body.as_ref(), indent_level + 1)
            ),
        }
    }
}

impl std::fmt::Display for Stmt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", indent_fmt(self, 0))
    }
}

fn display_option<T: PrintIndent>(object: Option<&T>, indent_level: usize, newline: bool) -> String {
    if let Some(object) = object {
        format!(
            "{}{}",
            if newline { "\n" } else { "" },
            indent_fmt(object, indent_level)
        )
    } else {
        "".to_string()
    }
}
