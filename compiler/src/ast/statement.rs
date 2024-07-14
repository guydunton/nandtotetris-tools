#![allow(dead_code)]

use serde::Serialize;

use super::{
    expression::Expr,
    variables::{Variable, VariableRef},
};

#[derive(Debug, Clone, Serialize)]
pub struct LetDetails {
    pub identifier: VariableRef,
    pub expression: Expr,
}

impl LetDetails {
    pub fn new() -> Self {
        Self {
            identifier: VariableRef::new(""),
            expression: Expr::int(0),
        }
    }

    pub fn id(mut self, id: VariableRef) -> Self {
        self.identifier = id;
        self
    }

    pub fn value(mut self, expr: Expr) -> Self {
        self.expression = expr;
        self
    }

    pub fn get_identifier(&self) -> &VariableRef {
        &self.identifier
    }

    pub fn get_expression(&self) -> &Expr {
        &self.expression
    }

    pub fn as_statement(self) -> Statement {
        Statement::Let(self)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct WhileDetails {
    pub condition: Expr,
    pub body: Vec<Statement>,
}

impl WhileDetails {
    pub fn new() -> Self {
        Self {
            condition: Expr::true_c(),
            body: Vec::new(),
        }
    }

    pub fn condition(mut self, condition: Expr) -> Self {
        self.condition = condition;
        self
    }

    pub fn add_statement(mut self, statement: Statement) -> Self {
        self.body.push(statement);
        self
    }

    pub fn get_condition(&self) -> &Expr {
        &self.condition
    }

    pub fn get_body(&self) -> &Vec<Statement> {
        &self.body
    }

    pub fn as_statement(self) -> Statement {
        Statement::While(self)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct IfDetails {
    pub condition: Expr,
    pub if_body: Vec<Statement>,
    pub else_body: Option<Vec<Statement>>,
}

impl IfDetails {
    pub fn new() -> Self {
        Self {
            condition: Expr::true_c(),
            if_body: Vec::new(),
            else_body: None,
        }
    }

    pub fn condition(mut self, condition: Expr) -> Self {
        self.condition = condition;
        self
    }

    pub fn add_if_statement(mut self, statement: Statement) -> Self {
        self.if_body.push(statement);
        self
    }

    pub fn add_else_statement(mut self, statement: Statement) -> Self {
        if let Some(else_body) = &mut self.else_body {
            else_body.push(statement);
        } else {
            self.else_body = Some(vec![statement]);
        }
        self
    }

    pub fn get_condition(&self) -> &Expr {
        &self.condition
    }

    pub fn get_if_body(&self) -> &Vec<Statement> {
        &self.if_body
    }

    pub fn get_else_body(&self) -> Option<&Vec<Statement>> {
        self.else_body.as_ref()
    }

    pub fn as_statement(self) -> Statement {
        Statement::If(self)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Default)]
pub struct SubroutineCall {
    target_name: Option<String>,
    subroutine_name: String,
    parameters: Vec<Expr>,
}

impl SubroutineCall {
    pub fn new() -> Self {
        SubroutineCall {
            ..Default::default()
        }
    }

    pub fn as_statement(self) -> Statement {
        Statement::Do(self)
    }

    pub fn as_expr(self) -> Expr {
        Expr::Call(self)
    }

    pub fn set_target(mut self, target_name: &str) -> Self {
        self.target_name = Some(target_name.to_owned());
        self
    }

    pub fn get_target(&self) -> &Option<String> {
        &self.target_name
    }

    pub fn get_name(&self) -> &str {
        &self.subroutine_name
    }

    pub fn name(mut self, name: &str) -> Self {
        self.subroutine_name = name.to_owned();
        self
    }

    pub fn add_parameter(mut self, expr: Expr) -> Self {
        self.parameters.push(expr);
        self
    }

    pub fn add_parameters(mut self, parameters: Vec<Expr>) -> Self {
        parameters
            .into_iter()
            .for_each(|parameter| self.parameters.push(parameter));
        self
    }

    pub fn name_as_string(&self) -> String {
        let type_portion = self
            .target_name
            .clone()
            .map(|type_name| format!("{}.", type_name));
        format!(
            "{}{}",
            type_portion.unwrap_or_default(),
            self.subroutine_name
        )
    }

    pub fn get_parameters(&self) -> &Vec<Expr> {
        &self.parameters
    }
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct VarDeclDetails {
    variables: Vec<Variable>,
}

impl VarDeclDetails {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn add_var(mut self, var: Variable) -> Self {
        self.variables.push(var);
        self
    }

    pub fn get_variables(&self) -> &Vec<Variable> {
        &self.variables
    }

    pub fn as_statement(self) -> Statement {
        Statement::VarDecl(self)
    }
}

#[derive(Debug, Clone, Serialize)]
pub enum Statement {
    Let(LetDetails),
    While(WhileDetails),
    Do(SubroutineCall),
    If(IfDetails),
    Return(Option<Expr>),
    VarDecl(VarDeclDetails),
}

impl Statement {
    pub fn do_statement() -> SubroutineCall {
        SubroutineCall::new()
    }

    pub fn var() -> VarDeclDetails {
        VarDeclDetails::new()
    }

    pub fn let_statement() -> LetDetails {
        LetDetails::new()
    }

    pub fn return_void() -> Statement {
        Statement::Return(None)
    }

    pub fn return_expr(expr: Expr) -> Statement {
        Statement::Return(Some(expr))
    }

    pub fn while_loop() -> WhileDetails {
        WhileDetails::new()
    }

    pub fn if_statement() -> IfDetails {
        IfDetails::new()
    }
}
