use crate::{
    ast::{
        BinaryOp, Class, ClassVariableVisibility, Constant, Expr, Statement, Subroutine,
        SubroutineType, UnaryOp, AST,
    },
    symbol_table::SymbolTable,
};

pub struct CompilationOutput {
    pub source_filename: String,
    pub vm_code: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum CompilationError {
    MissingVariable { var_name: String },
}

struct CompilationContext {
    symbol_table: SymbolTable,
    class_name: String,
    subroutine_name: String,
    while_count: i32,
    if_count: i32,
}

impl CompilationContext {
    pub fn new(class_name: &str) -> Self {
        Self {
            symbol_table: SymbolTable::new(),
            class_name: class_name.to_owned(),
            if_count: 0,
            while_count: 0,
            subroutine_name: "".to_owned(),
        }
    }

    pub fn set_subroutine_name(&mut self, name: &str) {
        self.subroutine_name = name.to_owned();
    }

    pub fn symbol_table(&mut self) -> &mut SymbolTable {
        &mut self.symbol_table
    }

    /// Create a label for a while loop & increment the counter.
    ///
    /// A label will look like: main.while.0
    pub fn next_while_label(&mut self) -> String {
        // main.while.0.condition
        let while_label = format!("{}.while.{}", self.subroutine_name, self.while_count);
        self.while_count += 1;
        while_label
    }

    /// Create a label for a if statement & increment the counter.
    ///
    /// A label will look like: main.if.0
    pub fn next_if_label(&mut self) -> String {
        let if_label = format!("{}.if.{}", self.subroutine_name, self.if_count);
        self.if_count += 1;
        if_label
    }
}

pub fn translate_ast(ast: &AST) -> Result<Vec<CompilationOutput>, CompilationError> {
    let mut output = Vec::with_capacity(ast.classes.len());

    for compiled_class in &ast.classes {
        let vm_code = compile_class(&compiled_class.class)?;
        output.push(CompilationOutput {
            source_filename: compiled_class.source_filename.clone(),
            vm_code,
        })
    }

    Ok(output)
}

pub fn compile_class(class: &Class) -> Result<Vec<String>, CompilationError> {
    let mut output = Vec::new();

    let mut context = CompilationContext::new(class.get_name());

    // Find all the local variables
    for variable in class.variables() {
        match variable.get_visibility() {
            ClassVariableVisibility::Field => {
                context.symbol_table().add_field(
                    &variable.get_identifier(),
                    &variable.get_var_type().to_string(),
                );
            }
            ClassVariableVisibility::Static => {
                context.symbol_table().add_static(
                    &variable.get_identifier(),
                    &variable.get_var_type().to_string(),
                );
            }
        }
    }

    for subroutine in class.subroutines() {
        context.symbol_table().create_scope();
        context.set_subroutine_name(subroutine.get_name());
        compile_subroutines(&mut output, subroutine, &mut context)?;
        context.symbol_table().pop_scope();
    }

    Ok(output)
}

fn compile_subroutines(
    output: &mut Vec<String>,
    subroutine: &Subroutine,
    context: &mut CompilationContext,
) -> Result<(), CompilationError> {
    if subroutine.get_subroutine_type() == SubroutineType::Method {
        let class_name = context.class_name.clone();
        context.symbol_table().add_argument("this", &class_name);
    }

    // create the symbol table for the subroutine
    for parameter in subroutine.get_parameters() {
        context.symbol_table().add_argument(
            parameter.get_identifier(),
            &format!("{}", parameter.get_type().to_string()),
        );
    }

    // Find all the var declarations
    for s in subroutine.get_statements() {
        find_var_decl_in_statement_tree(s, context.symbol_table());
    }

    let num_args = context.symbol_table().count_locals();

    output.push(format!(
        "function {}.{} {}",
        context.class_name,
        subroutine.get_name(),
        num_args
    ));

    match subroutine.get_subroutine_type() {
        SubroutineType::Constructor => {
            // Count the number of class fields
            output.push(format!(
                "push constant {}",
                context.symbol_table().count_fields()
            ));
            output.push("call Memory.alloc 1".to_owned());
            output.push("pop pointer 0".to_owned());
        }
        SubroutineType::Method => {
            output.push("push argument 0".to_owned());
            output.push("pop pointer 0".to_owned());
        }
        _ => {}
    }

    for statement in subroutine.get_statements() {
        compile_statement(output, statement, context)?;
    }

    Ok(())
}

fn compile_statement(
    output: &mut Vec<String>,
    statement: &Statement,
    context: &mut CompilationContext,
) -> Result<(), CompilationError> {
    match statement {
        Statement::Let(details) => {
            // Find the correct variable
            let variable = context
                .symbol_table()
                .find_variable(details.identifier.get_name())
                .ok_or(CompilationError::MissingVariable {
                    var_name: details.identifier.get_name().to_owned(),
                })?;

            let scope = match variable.scope() {
                crate::symbol_table::Scope::Field => "this",
                crate::symbol_table::Scope::Static => "static",
                crate::symbol_table::Scope::Argument => "argument",
                crate::symbol_table::Scope::Local => "local",
            };

            let variable_index = variable.index();

            // Prepare to store in an Array if appropriate
            if let Some(index) = details.identifier.get_index() {
                output.push(format!("push {} {}", scope, variable_index));
                compile_expression(output, index, context)?;
                output.push("add".to_owned());
            }

            // Put the expression into the stack
            compile_expression(output, details.get_expression(), context)?;

            // If an array we need to store the expression result to setup the array access
            if details.identifier.get_index().is_some() {
                output.push("pop temp 0".to_owned());
                output.push("pop pointer 1".to_owned());
                output.push("push temp 0".to_owned());
                output.push("pop that 0".to_owned());
            } else {
                output.push(format!("pop {} {}", scope, variable_index));
            }
        }
        Statement::While(details) => {
            // Create a name for the while for labels
            let while_label = context.next_while_label();

            // Label condition
            output.push(format!("label {}.condition", while_label));

            // Condition
            compile_expression(output, details.get_condition(), context)?;

            // if-goto while_body
            output.push(format!("if-goto {}.while_body", while_label));

            // goto while_end
            output.push(format!("goto {}.while_end", while_label));

            // label while_body
            output.push(format!("label {}.while_body", while_label));

            // statements
            for s in &details.body {
                compile_statement(output, s, context)?;
            }

            // goto condition
            output.push(format!("goto {}.condition", while_label));

            // label while_end
            output.push(format!("label {}.while_end", while_label));
        }
        Statement::Do(call) => {
            for parameter in call.get_parameters() {
                compile_expression(output, parameter, context)?;
            }

            let mut param_count = call.get_parameters().len();
            let mut call_text = call.name_as_string();

            // Check if the subroutine call is a method call or a function call
            // main.draw() <- if main is variable then this is method call otherwise it's a function call
            // draw() <- must be method call
            match call.get_target() {
                Some(target_name) => match context.symbol_table().find_variable(&target_name) {
                    Some(variable) => {
                        output.push(format!(
                            "push {} {}",
                            variable.scope().as_segment(),
                            variable.index()
                        ));

                        param_count += 1;
                        call_text = format!("{}.{}", variable.var_type(), call.get_name());
                    }
                    None => {}
                },
                None => {
                    output.push("push pointer 0".to_owned());
                    param_count += 1;
                    call_text = format!("{}.{}", context.class_name, call.get_name());
                }
            };

            output.push(format!("call {} {}", call_text, param_count,));

            // We aren't doing anything with the response so pop it
            output.push("pop temp 0".to_owned());
        }
        Statement::If(details) => {
            // Get a label for the if statement
            let if_label = context.next_if_label();

            // push constant 1
            // neg
            compile_expression(output, details.get_condition(), context)?;

            // if-goto main.if.0.if_body
            output.push(format!("if-goto {}.if_body", if_label));

            if let Some(else_body) = details.get_else_body() {
                for s in else_body {
                    compile_statement(output, s, context)?;
                }
            }

            //     goto main.if.0.if_end
            output.push(format!("goto {}.if_end", if_label));

            // label main.if.0.if_body
            output.push(format!("label {}.if_body", if_label));

            for s in details.get_if_body() {
                compile_statement(output, s, context)?;
            }

            // label main.if.0.if_end
            output.push(format!("label {}.if_end", if_label));
        }
        Statement::Return(return_statement) => {
            if let Some(expr) = return_statement {
                compile_expression(output, expr, context)?;
                output.push("return".to_owned());
            } else {
                output.push("push constant 0".to_owned());
                output.push("return".to_owned());
            }
        }
        Statement::VarDecl(_) => {}
    }

    Ok(())
}

fn compile_expression(
    output: &mut Vec<String>,
    expr: &Expr,
    context: &mut CompilationContext,
) -> Result<(), CompilationError> {
    match expr {
        Expr::Constant(Constant::Int(num_val)) => output.push(format!("push constant {}", num_val)),
        Expr::Constant(Constant::String(text)) => {
            output.push(format!("push constant {}", text.len()));
            output.push("call String.new 1".to_owned());
            for char in text.chars() {
                output.push(format!("push constant {}", char as u8));
                output.push("call String.appendChar 2".to_owned());
            }
        }
        Expr::Constant(Constant::Keyword(keyword)) => match keyword {
            crate::ast::KeywordConstant::True => {
                output.push("push constant 1".to_owned());
                output.push("neg".to_owned());
            }
            crate::ast::KeywordConstant::False => output.push("push constant 0".to_owned()),
            crate::ast::KeywordConstant::Null => output.push("push constant 0".to_owned()),
            crate::ast::KeywordConstant::This => output.push("push pointer 0".to_owned()),
        },
        Expr::VarRef(var) => {
            let variable = context.symbol_table().find_variable(var.get_name()).ok_or(
                CompilationError::MissingVariable {
                    var_name: var.get_name().to_owned(),
                },
            )?;

            let scope = match variable.scope() {
                crate::symbol_table::Scope::Field => "this",
                crate::symbol_table::Scope::Static => "static",
                crate::symbol_table::Scope::Argument => "argument",
                crate::symbol_table::Scope::Local => "local",
            };

            let variable_index = variable.index();

            if let Some(index) = var.get_index() {
                output.push(format!("push {} {}", scope, variable_index));
                compile_expression(output, index, context)?;
                output.push("add".to_owned());
                output.push("pop pointer 1".to_owned());
                output.push("push that 0".to_owned());
            } else {
                output.push(format!("push {} {}", scope, variable_index));
            }
        }
        Expr::UnaryExpr(op, expr) => {
            compile_expression(output, expr, context)?;
            let operator = match op {
                UnaryOp::Minus => "neg",
                UnaryOp::Not => "not",
            };
            output.push(format!("{}", operator));
        }
        Expr::BinaryExpr { lhs, op, rhs } => {
            compile_expression(output, lhs, context)?;
            compile_expression(output, rhs, context)?;
            match op {
                BinaryOp::Plus => output.push("add".to_owned()),
                BinaryOp::Minus => output.push("sub".to_owned()),
                BinaryOp::Mult => output.push("call Math.multiply 2".to_owned()),
                BinaryOp::Div => output.push("call Math.divide 2".to_owned()),
                BinaryOp::And => output.push("and".to_owned()),
                BinaryOp::Or => output.push("or".to_owned()),
                BinaryOp::Lt => output.push("lt".to_owned()),
                BinaryOp::Gt => output.push("gt".to_owned()),
                BinaryOp::Eq => output.push("eq".to_owned()),
            }
        }
        Expr::BracketedExpr(expr) => compile_expression(output, expr, context)?,
        Expr::Call(call) => {
            let mut param_count = call.get_parameters().len();
            let mut call_text = call.name_as_string();

            // If the call is a method then we need to push this
            match call.get_target() {
                Some(target_name) => match context.symbol_table().find_variable(&target_name) {
                    Some(variable) => {
                        output.push(format!(
                            "push {} {}",
                            variable.scope().as_segment(),
                            variable.index()
                        ));
                        param_count += 1;
                        call_text = format!("{}.{}", variable.var_type(), call.get_name());
                    }
                    None => {}
                },
                None => {
                    output.push("push pointer 0".to_owned());
                    param_count += 1;
                    call_text = format!("{}.{}", context.class_name, call.get_name());
                }
            };

            for parameter in call.get_parameters() {
                compile_expression(output, parameter, context)?;
            }

            output.push(format!("call {} {}", call_text, param_count));
        }
    }

    Ok(())
}

fn find_var_decl_in_statement_tree(statement: &Statement, symbol_table: &mut SymbolTable) {
    match statement {
        Statement::Let(_) => {}
        Statement::While(details) => {
            for body_statement in &details.body {
                find_var_decl_in_statement_tree(&body_statement, symbol_table);
            }
        }
        Statement::Do(_) => {}
        Statement::If(if_details) => {
            for s in &if_details.if_body {
                find_var_decl_in_statement_tree(s, symbol_table);
            }
            if let Some(else_body) = &if_details.else_body {
                for s in else_body {
                    find_var_decl_in_statement_tree(s, symbol_table);
                }
            }
        }
        Statement::Return(_) => {}
        Statement::VarDecl(var_details) => {
            for var in var_details.get_variables() {
                symbol_table.add_local(
                    var.get_identifier(),
                    &format!("{}", var.get_type().to_string()),
                );
            }
        }
    }
}
