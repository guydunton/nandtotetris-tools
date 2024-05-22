use crate::{
    ast::{BinaryOp, Class, Constant, Expr, Statement, Subroutine, UnaryOp, VariableRef, AST},
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

fn compile_class(class: &Class) -> Result<Vec<String>, CompilationError> {
    let mut output = Vec::new();

    let mut context = CompilationContext::new(class.get_name());

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
    // create the symbol table for the subroutine
    for parameter in subroutine.get_parameters() {
        context.symbol_table().add_argument(
            parameter.get_identifier(),
            &format!("{:?}", parameter.get_type()),
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
            // Put the expression into the stack first
            compile_expression(output, details.get_expression(), context)?;

            if details.identifier.get_index().is_some() {
                todo!();
            }

            // Add the push to the correct variable
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

            output.push(format!("pop {} {}", scope, variable.index()));
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

            output.push(format!(
                "call {} {}",
                call.name_as_string(),
                call.get_parameters().len()
            ));

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
        Expr::Constant(Constant::String(_)) => todo!(),
        Expr::Constant(Constant::Keyword(keyword)) => match keyword {
            crate::ast::KeywordConstant::True => {
                output.push("push constant 1".to_owned());
                output.push("neg".to_owned());
            }
            crate::ast::KeywordConstant::False => output.push("push constant 0".to_owned()),
            crate::ast::KeywordConstant::Null => output.push("push constant 0".to_owned()),
            crate::ast::KeywordConstant::This => todo!(),
        },
        Expr::VarRef(var) => {
            if var.get_index().is_some() {
                todo!();
            }

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

            output.push(format!("push {} {}", scope, variable.index()));
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
            for parameter in call.get_parameters() {
                compile_expression(output, parameter, context)?;
            }

            output.push(format!(
                "call {} {}",
                call.name_as_string(),
                call.get_parameters().len()
            ));
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
                symbol_table.add_local(var.get_identifier(), &format!("{:?}", var.get_type()));
            }
        }
    }
}

#[test]
fn test_compile_function() {
    let class = Class::new("Main").add_subroutine(
        Subroutine::new("main")
            .add_statement(
                Statement::do_statement()
                    .set_type("Output")
                    .name("printInt")
                    .add_parameter(Expr::int(3))
                    .as_statement(),
            )
            .add_statement(Statement::return_void()),
    );

    let result = compile_class(&class).unwrap();

    let expected: Vec<String> = r#"
        function Main.main 0
        push constant 3
        call Output.printInt 1
        pop temp 0
        push constant 0
        return
    "#
    .trim()
    .split('\n')
    .map(|s| s.trim().to_owned())
    .collect();

    assert_eq!(result, expected);
}

#[test]
fn test_compile_simple_expression() {
    use crate::ast::BinaryOp;

    // 1 + 2
    let expression = Expr::binary_op(Expr::int(1), BinaryOp::Plus, Expr::int(2));

    let class = Class::new("Main").add_subroutine(
        Subroutine::new("main")
            .add_statement(
                Statement::do_statement()
                    .set_type("Output")
                    .name("printInt")
                    .add_parameter(expression)
                    .as_statement(),
            )
            .add_statement(Statement::return_void()),
    );

    let result = compile_class(&class).unwrap();

    let expected: Vec<String> = r#"
        push constant 1
        push constant 2
        add
    "#
    .trim()
    .split('\n')
    .map(|s| s.trim().to_owned())
    .collect();

    assert!(contains_commands(&result, &expected));
}

#[test]
fn test_compile_complex_expression() {
    use crate::ast::BinaryOp;

    // 1 + (2 * 3)
    let expression = Expr::binary_op(
        Expr::int(1),
        BinaryOp::Plus,
        Expr::brackets(Expr::binary_op(Expr::int(2), BinaryOp::Mult, Expr::int(3))),
    );

    let class = Class::new("Main").add_subroutine(
        Subroutine::new("main")
            .add_statement(
                Statement::do_statement()
                    .set_type("Output")
                    .name("printInt")
                    .add_parameter(expression)
                    .as_statement(),
            )
            .add_statement(Statement::return_void()),
    );

    let result = compile_class(&class).unwrap();

    let expected: Vec<String> = r#"
        push constant 1
        push constant 2
        push constant 3
        call Math.multiply 2
        add
    "#
    .trim()
    .split('\n')
    .map(|s| s.trim().to_owned())
    .collect();

    assert!(contains_commands(&result, &expected));
}

#[test]
fn compile_var_statement() {
    let class = Class::new("Main").add_subroutine(
        Subroutine::new("main")
            .add_statement(
                Statement::var()
                    .add_var(crate::ast::Variable::new(
                        "value",
                        crate::ast::VariableType::Int,
                    ))
                    .as_statement(),
            )
            .add_statement(Statement::return_void()),
    );

    let result = compile_class(&class).unwrap();
    let expected: Vec<String> = r#"
        function Main.main 1
        push constant 0
        return
    "#
    .trim()
    .split('\n')
    .map(|s| s.trim().to_owned())
    .collect();

    assert_eq!(result, expected);
}

#[test]
fn compile_let() {
    use crate::ast::{VariableRef, VariableType};
    let class = Class::new("Main").add_subroutine(
        Subroutine::new("main")
            .add_statement(
                Statement::var()
                    .add_var(crate::ast::Variable::new("value", VariableType::Int))
                    .as_statement(),
            )
            .add_statement(
                Statement::let_statement()
                    .id(VariableRef::new("value"))
                    .value(Expr::int(3))
                    .as_statement(),
            )
            .add_statement(Statement::return_void()),
    );

    let result = compile_class(&class).unwrap();
    let expected: Vec<String> = r#"
        function Main.main 1
        push constant 3
        pop local 0
        push constant 0
        return
    "#
    .trim()
    .split('\n')
    .map(|s| s.trim().to_owned())
    .collect();

    assert_eq!(result, expected);
}

#[test]
fn compile_var_used_in_do_statement() {
    use crate::ast::{Variable, VariableRef, VariableType};

    let class = Class::new("Main").add_subroutine(
        Subroutine::new("main")
            .add_statement(
                Statement::var()
                    .add_var(Variable::new("value", VariableType::Int))
                    .as_statement(),
            )
            .add_statement(
                Statement::let_statement()
                    .id(VariableRef::new("value"))
                    .value(Expr::int(3))
                    .as_statement(),
            )
            .add_statement(
                Statement::do_statement()
                    .set_type("Output")
                    .name("printInt")
                    .add_parameter(Expr::var(VariableRef::new("value")))
                    .as_statement(),
            )
            .add_statement(Statement::return_void()),
    );

    let result = compile_class(&class).unwrap();

    let expected: Vec<String> = r#"
        push constant 3
        pop local 0
        push local 0
        call Output.printInt 1
    "#
    .trim()
    .split('\n')
    .map(|s| s.trim().to_owned())
    .collect();

    assert!(contains_commands(&result, &expected));
}

#[test]
fn compile_unary_operation_test() {
    let class = Class::new("Main").add_subroutine(
        Subroutine::new("main")
            .add_statement(
                Statement::do_statement()
                    .set_type("Output")
                    .name("printInt")
                    .add_parameter(Expr::unary_op(UnaryOp::Minus, Expr::int(3)))
                    .as_statement(),
            )
            .add_statement(Statement::return_void()),
    );

    let result = compile_class(&class).unwrap();

    let expected: Vec<String> = r#"
        push constant 3
        neg
        call Output.printInt 1
    "#
    .trim()
    .split('\n')
    .map(|s| s.trim().to_owned())
    .collect();

    assert!(contains_commands(&result, &expected));
}

#[test]
fn compile_multiple_functions() {
    use crate::ast::{Variable, VariableRef, VariableType};

    let var_and_set: Vec<Statement> = vec![
        Statement::var()
            .add_var(Variable::new("value", VariableType::Int))
            .as_statement(),
        Statement::let_statement()
            .id(VariableRef::new("value"))
            .value(Expr::int(3))
            .as_statement(),
    ];

    let class = Class::new("Main")
        .add_subroutine(
            Subroutine::new("main")
                .add_statements(var_and_set.clone())
                .add_statement(
                    Statement::do_statement()
                        .set_type("Main")
                        .name("second")
                        .add_parameter(Expr::var(VariableRef::new("value")))
                        .as_statement(),
                )
                .add_statement(Statement::return_void()),
        )
        .add_subroutine(
            Subroutine::new("second")
                .add_statements(var_and_set)
                .add_statement(Statement::return_expr(Expr::var(VariableRef::new("value")))),
        );

    let result = compile_class(&class).unwrap();

    let expected: Vec<String> = r#"
        function Main.main 1
        push constant 3
        pop local 0
        push local 0
        call Main.second 1
        pop temp 0
        push constant 0
        return
        function Main.second 1
        push constant 3
        pop local 0
        push local 0
        return
    "#
    .trim()
    .split('\n')
    .map(|s| s.trim().to_owned())
    .collect();

    assert_eq!(result, expected);
}

#[test]
fn compile_function_with_args() {
    use crate::ast::{Variable, VariableRef, VariableType};

    let class = Class::new("Main").add_subroutine(
        Subroutine::new("main")
            .return_type(crate::ast::ReturnType::Int)
            .add_parameter(Variable::new("first", VariableType::Int))
            .add_parameter(Variable::new("second", VariableType::Int))
            .add_statement(Statement::return_expr(Expr::var(VariableRef::new(
                "second",
            )))),
    );

    let result = compile_class(&class).unwrap();

    let expected: Vec<String> = r#"
        function Main.main 0
        push argument 1
        return
    "#
    .trim()
    .split('\n')
    .map(|s| s.trim().to_owned())
    .collect();

    assert_eq!(result, expected);
}

#[test]
fn compile_while_loop() {
    /*
    while (true) {
        Output.printInt(2);
    }
    return;
     */
    let class = Class::new("Main").add_subroutine(
        Subroutine::new("main")
            .add_statement(
                Statement::while_loop()
                    .condition(Expr::true_c())
                    .add_statement(
                        Statement::do_statement()
                            .set_type("Output")
                            .name("printInt")
                            .add_parameter(Expr::int(2))
                            .as_statement(),
                    )
                    .as_statement(),
            )
            .add_statement(Statement::return_void()),
    );

    let result = compile_class(&class).unwrap();

    let expected: Vec<String> = r#"
            function Main.main 0
                label main.while.0.condition
                    push constant 1
                    neg
                if-goto main.while.0.while_body
                    goto main.while.0.while_end
                label main.while.0.while_body
                    push constant 2
                    call Output.printInt 1
                    pop temp 0
                    goto main.while.0.condition
                label main.while.0.while_end
            push constant 0
            return
        "#
    .trim()
    .split('\n')
    .map(|s| s.trim().to_owned())
    .collect();

    assert_eq!(result, expected);
}

#[test]
fn compile_if_statement() {
    /*
    if (true) {
        Output.printInt(2);
    } else {
        Output.printInt(3);
    }
    return;
     */
    let class = Class::new("Main").add_subroutine(
        Subroutine::new("main")
            .add_statement(
                Statement::if_statement()
                    .condition(Expr::true_c())
                    .add_if_statement(
                        Statement::do_statement()
                            .set_type("Output")
                            .name("printInt")
                            .add_parameter(Expr::int(2))
                            .as_statement(),
                    )
                    .add_else_statement(
                        Statement::do_statement()
                            .set_type("Output")
                            .name("printInt")
                            .add_parameter(Expr::int(3))
                            .as_statement(),
                    )
                    .as_statement(),
            )
            .add_statement(Statement::return_void()),
    );

    let result = compile_class(&class).unwrap();

    let expected: Vec<String> = r#"
            function Main.main 0
                push constant 1
                neg
                if-goto main.if.0.if_body
                    push constant 3
                    call Output.printInt 1
                    pop temp 0
                    goto main.if.0.if_end
                label main.if.0.if_body
                    push constant 2
                    call Output.printInt 1
                    pop temp 0
                label main.if.0.if_end
            push constant 0
            return
        "#
    .trim()
    .split('\n')
    .map(|s| s.trim().to_owned())
    .collect();

    assert_eq!(result, expected);
}

#[test]
fn compile_let_with_call() {
    use crate::ast::{Variable, VariableType};

    // Test `let mask = Main.nextMask(mask);`
    let class = Class::new("Main").add_subroutine(
        Subroutine::new("main")
            .return_type(crate::ast::ReturnType::Void)
            .add_statement(
                Statement::var()
                    .add_var(Variable::new("mask", VariableType::Int))
                    .as_statement(),
            )
            .add_statement(
                Statement::let_statement()
                    .id(VariableRef::new("mask"))
                    .value(
                        Expr::call()
                            .set_type("Main")
                            .name("nextMask")
                            .add_parameter(Expr::var(VariableRef::new("mask")))
                            .as_expr(),
                    )
                    .as_statement(),
            )
            .add_statement(Statement::return_void()),
    );

    let result = compile_class(&class).unwrap();

    let expected: Vec<String> = r#"
        function Main.main 1
        push local 0
        call Main.nextMask 1
        pop local 0
        push constant 0
        return
    "#
    .trim()
    .split('\n')
    .map(|s| s.trim().to_owned())
    .collect();

    assert_eq!(result, expected);
}

#[allow(dead_code)]
fn contains_commands(result: &Vec<String>, expected: &Vec<String>) -> bool {
    result
        .windows(expected.len())
        .position(|window| window == expected)
        .is_some()
}
