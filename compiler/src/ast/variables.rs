use serde::Serialize;

use super::expression::Expr;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum VariableType {
    Array,
    Int,
    Char,
    Boolean,
    ClassName(String),
}

impl ToString for VariableType {
    fn to_string(&self) -> String {
        match self {
            VariableType::Array => "Array".to_owned(),
            VariableType::Int => "Int".to_owned(),
            VariableType::Char => "Char".to_owned(),
            VariableType::Boolean => "Bool".to_owned(),
            VariableType::ClassName(name) => name.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Variable {
    identifier: String,
    var_type: VariableType,
}

impl Variable {
    pub fn new(identifier: &str, var_type: VariableType) -> Self {
        Self {
            identifier: identifier.to_owned(),
            var_type,
        }
    }

    pub fn get_identifier(&self) -> &str {
        &self.identifier
    }

    pub fn get_type(&self) -> &VariableType {
        &self.var_type
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct VariableRef {
    name: String,
    index: Option<Box<Expr>>,
}

impl VariableRef {
    pub fn new(identifier: &str) -> Self {
        Self {
            name: identifier.to_owned(),
            index: None,
        }
    }

    pub fn new_with_index(identifier: &str, index: Expr) -> Self {
        Self {
            name: identifier.to_owned(),
            index: Some(Box::new(index)),
        }
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_index(&self) -> Option<&Box<Expr>> {
        self.index.as_ref()
    }
}
