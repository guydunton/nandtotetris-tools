#![allow(dead_code)]

use serde::Serialize;

use super::{subroutine::Subroutine, variables::VariableType};

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
    identifier: String,
    subroutines: Vec<Subroutine>,
    variables: Vec<ClassVariable>,
}

impl Class {
    pub fn new(identifier: &str) -> Self {
        Self {
            identifier: identifier.to_owned(),
            subroutines: Vec::new(),
            variables: Vec::new(),
        }
    }

    pub fn add_subroutine(mut self, subroutine: Subroutine) -> Self {
        self.subroutines.push(subroutine);
        self
    }

    pub fn add_subroutines(mut self, subroutines: Vec<Subroutine>) -> Self {
        subroutines
            .into_iter()
            .for_each(|subroutine| self.subroutines.push(subroutine));
        self
    }

    pub fn add_variables(mut self, variables: Vec<ClassVariable>) -> Self {
        variables
            .into_iter()
            .for_each(|var| self.variables.push(var));
        self
    }

    pub fn subroutines(&self) -> &Vec<Subroutine> {
        &self.subroutines
    }

    pub fn get_name(&self) -> &str {
        &self.identifier
    }
}

pub struct CompiledClass {
    pub class: Class,
    pub source_filename: String,
}

pub struct AST {
    pub classes: Vec<CompiledClass>,
}
