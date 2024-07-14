#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Scope {
    Field,
    Static,
    Argument,
    Local,
}

impl Scope {
    pub fn as_segment(&self) -> String {
        match self {
            Scope::Field => "this".to_owned(),
            Scope::Static => "static".to_owned(),
            Scope::Argument => "argument".to_owned(),
            Scope::Local => "local".to_owned(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SymbolTableVariable {
    name: String,
    scope: Scope,
    var_type: String,
    index: i32,
}

impl SymbolTableVariable {
    pub fn new(name: &str, var_type: &str, scope: Scope, index: i32) -> Self {
        Self {
            name: name.to_owned(),
            var_type: var_type.to_owned(),
            scope,
            index,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn scope(&self) -> Scope {
        self.scope
    }

    pub fn var_type(&self) -> &str {
        &self.var_type
    }

    pub fn index(&self) -> i32 {
        self.index
    }
}

/// Symbol table
///
/// This is a table which contains the following information:
/// - name
/// - type
/// - kind
/// - number
///
/// When a new scope is entered it can segment off the variables in that segment.
#[derive(Debug)]
pub struct SymbolTable {
    vars: Vec<SymbolTableVariable>,
    scopes: Vec<usize>,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            vars: Vec::new(),
            scopes: Vec::new(),
        }
    }

    pub fn add_field(&mut self, var_name: &str, var_type: &str) {
        self.vars.push(SymbolTableVariable::new(
            var_name,
            var_type,
            Scope::Field,
            self.find_next_index(Scope::Field),
        ));
    }

    pub fn add_static(&mut self, var_name: &str, var_type: &str) {
        self.vars.push(SymbolTableVariable::new(
            var_name,
            var_type,
            Scope::Static,
            self.find_next_index(Scope::Static),
        ));
    }

    pub fn add_argument(&mut self, var_name: &str, var_type: &str) {
        self.vars.push(SymbolTableVariable::new(
            var_name,
            var_type,
            Scope::Argument,
            self.find_next_index(Scope::Argument),
        ));
    }

    pub fn add_local(&mut self, var_name: &str, var_type: &str) {
        self.vars.push(SymbolTableVariable::new(
            var_name,
            var_type,
            Scope::Local,
            self.find_next_index(Scope::Local),
        ));
    }

    pub fn count_locals(&self) -> i32 {
        self.vars
            .iter()
            .filter(|var| var.scope() == Scope::Local)
            .count() as i32
    }

    pub fn count_fields(&self) -> i32 {
        self.vars
            .iter()
            .filter(|var| var.scope() == Scope::Field)
            .count() as i32
    }

    pub fn find_variable(&self, var_name: &str) -> Option<SymbolTableVariable> {
        self.vars
            .iter()
            .rev()
            .find(|var| var.name() == var_name)
            .map(|var| var.clone())
    }

    pub fn create_scope(&mut self) {
        self.scopes.push(self.vars.len());
    }

    pub fn pop_scope(&mut self) {
        let scope_index = self.scopes.pop();
        if let Some(index) = scope_index {
            // Remove all the variables from the end to the index
            let num_pops = self.vars.len() - index;
            for _ in 0..num_pops {
                self.vars.pop();
            }
        }
    }

    fn find_next_index(&self, scope: Scope) -> i32 {
        let scope_start_index = self.scopes.last().unwrap_or(&0usize).clone();
        let scope_end_index = self.vars.len();
        let max_index = self.vars[scope_start_index..scope_end_index]
            .iter()
            .filter(|var| var.scope() == scope)
            .max_by_key(|var| var.index())
            .map(|var| var.index());
        max_index.map(|index| index + 1).unwrap_or(0)
    }
}

#[test]
fn create_a_symbol_table() {
    let mut table = SymbolTable::new();
    table.add_field("y", "int");

    let var = table.find_variable("y").unwrap();
    assert_eq!(var.name(), "y");
    assert_eq!(var.scope(), Scope::Field);
    assert_eq!(var.var_type(), "int");
}

#[test]
fn fields_have_a_number() {
    let mut table = SymbolTable::new();
    table.add_field("x", "int");
    table.add_field("y", "int");

    let x = table.find_variable("x").unwrap();
    let y = table.find_variable("y").unwrap();

    assert_eq!(x.index(), 0);
    assert_eq!(y.index(), 1);
}

#[test]
fn static_fields_number_separately() {
    let mut table = SymbolTable::new();
    table.add_field("x", "int");
    table.add_field("y", "int");

    table.add_static("sharedField", "boolean");

    assert_eq!(table.find_variable("sharedField").unwrap().index(), 0);
}

#[test]
fn scopes_can_be_pushed_and_popped() {
    let mut table = SymbolTable::new();
    table.add_local("var", "int");

    table.create_scope();
    table.add_local("var", "int");

    let inner_scope_var = table.find_variable("var").unwrap();
    assert_eq!(inner_scope_var.index(), 0);

    table.pop_scope();
    let outer_scope_var = table.find_variable("var").unwrap();
    assert_eq!(outer_scope_var.index(), 0);
}

#[test]
fn creating_a_scope_before_vars() {
    let mut table = SymbolTable::new();
    table.create_scope();
    table.add_argument("first", "int");
    table.add_argument("second", "int");

    let second = table.find_variable("second").unwrap();
    assert_eq!(second.index(), 1);
}

#[test]
fn count_field_vars() {
    let mut table = SymbolTable::new();
    table.add_field("field1", "int");
    table.add_field("field2", "int");
    table.create_scope();
    table.add_argument("arg1", "int");

    assert_eq!(table.count_fields(), 2);
}
