use serde::Serialize;

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum SubroutineType {
    Constructor,
    Function,
    Method,
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

#[derive(Debug, PartialEq, Serialize)]
pub struct VariableRef {
    pub name: String,
    pub index: Option<Box<Expr>>,
}

#[derive(Debug, PartialEq, Clone, Copy, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum KeywordConstant {
    True,
    False,
    Null,
    This,
}

#[derive(Debug, PartialEq, Serialize)]
pub enum Constant {
    Int(i32),
    String(String),
    Keyword(KeywordConstant),
}

#[derive(Debug, PartialEq, Serialize)]
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

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ReturnType {
    Int,
    Char,
    Boolean,
    Void,
    ClassName(String),
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum VariableType {
    Array,
    Int,
    Char,
    Boolean,
    ClassName(String),
}

#[derive(Debug, Serialize)]
pub struct Variable {
    pub identifier: String,
    pub var_type: VariableType,
}

#[derive(Debug, Serialize)]
pub struct LetDetails {
    pub identifier: VariableRef,
    pub expression: Expr,
}

#[derive(Debug, Serialize)]
pub struct WhileDetails {
    pub condition: Expr,
    pub body: Vec<Statement>,
}

#[derive(Debug, Serialize)]
pub struct IfDetails {
    pub condition: Expr,
    pub if_body: Vec<Statement>,
    pub else_body: Option<Vec<Statement>>,
}

#[derive(Debug, PartialEq, Serialize)]
pub struct SubroutineCall {
    pub type_name: Option<String>,
    pub subroutine_name: String,
    pub parameters: Vec<Expr>,
}

#[derive(Debug, Serialize)]
pub enum Statement {
    Let(LetDetails),
    While(WhileDetails),
    Do(SubroutineCall),
    If(IfDetails),
    Return(Option<Expr>),
    VarDecl(Vec<Variable>),
}

#[derive(Debug, Serialize)]
pub struct Subroutine {
    pub subroutine_type: SubroutineType,
    pub identifier: String,
    pub parameters: Vec<Variable>,
    pub return_type: ReturnType,
    pub statements: Vec<Statement>,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ClassVariableVisibility {
    Field,
    Static,
}

#[derive(Debug, Serialize)]
pub struct ClassVariable {
    pub visibility: ClassVariableVisibility,
    pub var_type: VariableType,
    pub identifier: String,
}

#[derive(Debug, Serialize)]
pub struct Class {
    pub identifier: String,
    pub subroutines: Vec<Subroutine>,
    pub variables: Vec<ClassVariable>,
}
