#![allow(dead_code)]

use serde::Serialize;

use super::{variables::VariableRef, SubroutineCall};

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum Expr {
    Constant(Constant),
    VarRef(VariableRef),
    UnaryExpr(UnaryOp, Box<Expr>),
    BinaryExpr {
        lhs: Box<Expr>,
        op: BinaryOp,
        rhs: Box<Expr>,
    },
    BracketedExpr(Box<Expr>),
    Call(SubroutineCall),
}

impl Expr {
    pub fn binary_op(lhs: Expr, op: BinaryOp, rhs: Expr) -> Expr {
        Expr::BinaryExpr {
            lhs: Box::new(lhs),
            op,
            rhs: Box::new(rhs),
        }
    }

    pub fn unary_op(op: UnaryOp, rhs: Expr) -> Expr {
        Expr::UnaryExpr(op, Box::new(rhs))
    }

    pub fn var(var: VariableRef) -> Expr {
        Expr::VarRef(var)
    }

    pub fn brackets(expr: Expr) -> Expr {
        Expr::BracketedExpr(Box::new(expr))
    }

    pub fn int(val: i32) -> Expr {
        Expr::Constant(Constant::Int(val))
    }

    pub fn string(val: &str) -> Expr {
        Expr::Constant(Constant::String(val.to_owned()))
    }

    pub fn true_c() -> Expr {
        Expr::Constant(Constant::Keyword(KeywordConstant::True))
    }
    pub fn false_c() -> Expr {
        Expr::Constant(Constant::Keyword(KeywordConstant::False))
    }
    pub fn null() -> Expr {
        Expr::Constant(Constant::Keyword(KeywordConstant::Null))
    }
    pub fn this() -> Expr {
        Expr::Constant(Constant::Keyword(KeywordConstant::This))
    }

    pub fn call() -> SubroutineCall {
        SubroutineCall::new()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum Constant {
    Int(i32),
    String(String),
    Keyword(KeywordConstant),
}

impl Constant {
    pub fn as_expr(self) -> Expr {
        Expr::Constant(self)
    }
}

#[derive(Debug, PartialEq, Clone, Copy, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum KeywordConstant {
    True,
    False,
    Null,
    This,
}

#[derive(Debug, PartialEq, Clone, Copy, Serialize)]
pub enum BinaryOp {
    Plus,
    Minus,
    Mult,
    Div,
    And,
    Or,
    Lt,
    Gt,
    Eq,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq)]
pub enum UnaryOp {
    Minus,
    Not,
}
