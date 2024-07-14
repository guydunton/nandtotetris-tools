#![allow(dead_code)]
use serde::Serialize;

use super::{statement::Statement, variables::Variable};

#[derive(Debug, Clone, Copy, Serialize, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SubroutineType {
    #[default]
    Function,
    Constructor,
    Method,
}

#[derive(Debug, Serialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ReturnType {
    Int,
    Char,
    Boolean,
    #[default]
    Void,
    ClassName(String),
}

#[derive(Debug, Serialize, Default)]
pub struct Subroutine {
    subroutine_type: SubroutineType,
    identifier: String,
    parameters: Vec<Variable>,
    return_type: ReturnType,
    statements: Vec<Statement>,
}

impl Subroutine {
    pub fn new(identifier: &str) -> Self {
        Self {
            identifier: identifier.to_owned(),
            ..Default::default()
        }
    }

    pub fn add_statement(mut self, statement: Statement) -> Self {
        self.statements.push(statement);
        self
    }

    pub fn add_statements(mut self, statements: Vec<Statement>) -> Self {
        statements
            .into_iter()
            .for_each(|statement| self.statements.push(statement));
        self
    }

    pub fn return_type(mut self, return_type: ReturnType) -> Self {
        self.return_type = return_type;
        self
    }

    pub fn add_parameter(mut self, parameter: Variable) -> Self {
        self.parameters.push(parameter);
        self
    }

    pub fn add_parameters(mut self, parameters: Vec<Variable>) -> Self {
        parameters
            .into_iter()
            .for_each(|parameter| self.parameters.push(parameter));
        self
    }

    pub fn subroutine_type(mut self, subroutine_type: SubroutineType) -> Self {
        self.subroutine_type = subroutine_type;
        self
    }

    pub fn get_subroutine_type(&self) -> SubroutineType {
        self.subroutine_type
    }

    pub fn get_name(&self) -> &String {
        &self.identifier
    }

    pub fn get_statements(&self) -> &Vec<Statement> {
        &self.statements
    }

    pub fn get_parameters(&self) -> &Vec<Variable> {
        &self.parameters
    }
}
