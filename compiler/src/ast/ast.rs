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
    visibility: ClassVariableVisibility,
    var_type: VariableType,
    identifier: String,
}

impl ClassVariable {
    pub fn new(identifier: &str) -> Self {
        Self {
            identifier: identifier.to_string(),
            var_type: VariableType::Int,
            visibility: ClassVariableVisibility::Field,
        }
    }

    pub fn var_type(mut self, var_type: VariableType) -> Self {
        self.var_type = var_type;
        return self;
    }

    pub fn visibility(mut self, visibility: ClassVariableVisibility) -> Self {
        self.visibility = visibility;
        return self;
    }

    pub fn get_identifier(&self) -> &str {
        &self.identifier
    }

    pub fn get_visibility(&self) -> ClassVariableVisibility {
        self.visibility
    }

    pub fn get_var_type(&self) -> VariableType {
        self.var_type.clone()
    }
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

    pub fn add_variable(mut self, variable: ClassVariable) -> Self {
        self.variables.push(variable);
        self
    }

    pub fn subroutines(&self) -> &Vec<Subroutine> {
        &self.subroutines
    }

    pub fn variables(&self) -> &Vec<ClassVariable> {
        &self.variables
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
