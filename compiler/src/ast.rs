#[derive(Debug)]
pub enum SubroutineType {
    Constructor,
}

#[derive(Debug)]
pub enum ReturnType {
    Int,
    Char,
    Boolean,
    ClassName(String),
}

#[derive(Debug)]
pub enum VariableType {
    Array,
    Int,
    Char,
    Boolean,
}

#[derive(Debug)]
pub struct Variable {
    pub identifier: String,
    pub var_type: VariableType,
}

#[derive(Debug)]
pub enum Statement {
    Let,
    While,
    Do,
    Return,
}

#[derive(Debug)]
pub struct Subroutine {
    pub subroutine_type: SubroutineType,
    pub identifier: String,
    pub return_type: ReturnType,
    pub var_declarations: Vec<Variable>,
    pub statements: Vec<Statement>,
}

#[derive(Debug)]
pub struct Class {
    pub identifier: String,
    pub subroutines: Vec<Subroutine>,
}
