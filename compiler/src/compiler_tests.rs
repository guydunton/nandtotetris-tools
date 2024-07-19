use crate::{
    ast::{
        BinaryOp, Class, ClassVariable, Expr, Statement, Subroutine, SubroutineType, UnaryOp,
        Variable, VariableRef, VariableType,
    },
    compiler::compile_class,
};

#[test]
fn test_compile_function() {
    let class = Class::new("Main").add_subroutine(
        Subroutine::new("main")
            .add_statement(
                Statement::do_statement()
                    .set_target("Output")
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
                    .set_target("Output")
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
                    .set_target("Output")
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
                    .set_target("Output")
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
                    .set_target("Output")
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
                        .set_target("Main")
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
                            .set_target("Output")
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
                            .set_target("Output")
                            .name("printInt")
                            .add_parameter(Expr::int(2))
                            .as_statement(),
                    )
                    .add_else_statement(
                        Statement::do_statement()
                            .set_target("Output")
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
                            .set_target("Main")
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

#[test]
fn compile_class_with_constructor() {
    let class = Class::new("Point")
        .add_variable(ClassVariable::new("x"))
        .add_variable(ClassVariable::new("y"))
        .add_subroutine(
            Subroutine::new("new")
                .subroutine_type(crate::ast::SubroutineType::Constructor)
                .return_type(crate::ast::ReturnType::ClassName("Point".to_string()))
                .add_parameter(Variable::new("ax", crate::ast::VariableType::Int))
                .add_parameter(Variable::new("ay", crate::ast::VariableType::Int))
                .add_statement(
                    Statement::let_statement()
                        .id(VariableRef::new("x"))
                        .value(Expr::var(VariableRef::new("ax")))
                        .as_statement(),
                )
                .add_statement(
                    Statement::let_statement()
                        .id(VariableRef::new("y"))
                        .value(Expr::var(VariableRef::new("ay")))
                        .as_statement(),
                )
                .add_statement(Statement::return_expr(Expr::this())),
        );

    let result = compile_class(&class).unwrap();

    let expected: Vec<String> = r#"
        function Point.new 0
        push constant 2
        call Memory.alloc 1
        pop pointer 0
        push argument 0
        pop this 0
        push argument 1
        pop this 1
        push pointer 0
        return
    "#
    .trim()
    .split('\n')
    .map(|s| s.trim().to_owned())
    .collect();

    assert_eq!(result, expected);
}

#[test]
fn test_method() {
    /*
    class Adder {
        field int a, b;

        constructor Adder new(int aa, int ab) {
            let a = aa;
            let b = ab;
            return this;
        }

        method int add() {
            return a + b;
        }
    }
     */
    let class = Class::new("Adder")
        .add_variable(ClassVariable::new("a"))
        .add_variable(ClassVariable::new("b"))
        .add_subroutine(
            Subroutine::new("new")
                .subroutine_type(crate::ast::SubroutineType::Constructor)
                .return_type(crate::ast::ReturnType::ClassName("Adder".to_string()))
                .add_parameter(Variable::new("aa", crate::ast::VariableType::Int))
                .add_parameter(Variable::new("ab", crate::ast::VariableType::Int))
                .add_statement(
                    Statement::let_statement()
                        .id(VariableRef::new("a"))
                        .value(Expr::var(VariableRef::new("aa")))
                        .as_statement(),
                )
                .add_statement(
                    Statement::let_statement()
                        .id(VariableRef::new("b"))
                        .value(Expr::var(VariableRef::new("ab")))
                        .as_statement(),
                )
                .add_statement(Statement::return_expr(Expr::this())),
        )
        .add_subroutine(
            Subroutine::new("add")
                .subroutine_type(SubroutineType::Method)
                .return_type(crate::ast::ReturnType::Int)
                .add_statement(Statement::return_expr(Expr::binary_op(
                    Expr::var(VariableRef::new("a")),
                    BinaryOp::Plus,
                    Expr::var(VariableRef::new("b")),
                ))),
        );

    let result = compile_class(&class).unwrap();

    let expected: Vec<String> = r#"
        function Adder.add 0
        push argument 0
        pop pointer 0
        push this 0
        push this 1
        add
        return
    "#
    .trim()
    .split('\n')
    .map(|s| s.trim().to_owned())
    .collect();

    assert!(contains_commands(&result, &expected));
}

#[test]
fn call_method_on_object() {
    /*
       class Main {
           function void main() {
               var Square square;
               let square = Square.new();
               do square.draw();
               return;
           }
       }
    */
    let class = Class::new("Main").add_subroutine(
        Subroutine::new("main")
            .return_type(crate::ast::ReturnType::Void)
            .add_statement(
                Statement::var()
                    .add_var(Variable::new(
                        "square",
                        VariableType::ClassName("Square".to_owned()),
                    ))
                    .as_statement(),
            )
            .add_statement(
                Statement::let_statement()
                    .id(VariableRef::new("square"))
                    .value(Expr::call().set_target("Square").name("new").as_expr())
                    .as_statement(),
            )
            .add_statement(
                Statement::do_statement()
                    .set_target("square")
                    .name("draw")
                    .as_statement(),
            )
            .add_statement(Statement::return_void()),
    );

    let result = compile_class(&class).unwrap();

    let expected: Vec<String> = r#"
        function Main.main 1
        call Square.new 0
        pop local 0
        push local 0
        call Square.draw 1
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
fn call_method_on_field_object() {
    /*
        class Game {
            field Ball ball;

            constructor Game new() {
                let ball = Ball.new();
                return this;
            }

            method void run() {
                do ball.run();
                return;
            }
       }
    */
    let class = Class::new("Game")
        .add_variable(
            ClassVariable::new("ball")
                .var_type(crate::ast::VariableType::ClassName("Ball".to_owned()))
                .visibility(crate::ast::ClassVariableVisibility::Field),
        )
        .add_subroutine(
            Subroutine::new("new")
                .subroutine_type(SubroutineType::Constructor)
                .return_type(crate::ast::ReturnType::ClassName("Game".to_owned()))
                .add_statement(
                    Statement::let_statement()
                        .id(VariableRef::new("ball"))
                        .value(Expr::call().set_target("Ball").name("new").as_expr())
                        .as_statement(),
                )
                .add_statement(Statement::return_expr(Expr::this())),
        )
        .add_subroutine(
            Subroutine::new("run")
                .subroutine_type(SubroutineType::Method)
                .add_statement(
                    Statement::do_statement()
                        .set_target("ball")
                        .name("run")
                        .as_statement(),
                )
                .add_statement(Statement::return_void()),
        );

    let result = compile_class(&class).unwrap();

    let expected: Vec<String> = r#"
        function Game.new 0
        push constant 1
        call Memory.alloc 1
        pop pointer 0
        call Ball.new 0
        pop this 0
        push pointer 0
        return
        function Game.run 0
        push argument 0
        pop pointer 0
        push this 0
        call Ball.run 1
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
fn test_chained_methods() {
    /*
    class Adder {

        constructor Adder new() {
            return this;
        }

        method int first() {
            return second();
        }

        method int second() {
            return 3;
        }
    }
     */
    let class = Class::new("Adder")
        .add_subroutine(
            Subroutine::new("new")
                .subroutine_type(crate::ast::SubroutineType::Constructor)
                .return_type(crate::ast::ReturnType::ClassName("Adder".to_string()))
                .add_statement(Statement::return_expr(Expr::this())),
        )
        .add_subroutine(
            Subroutine::new("first")
                .subroutine_type(SubroutineType::Method)
                .return_type(crate::ast::ReturnType::Int)
                .add_statement(Statement::return_expr(
                    Expr::call().name("second").as_expr(),
                )),
        )
        .add_subroutine(
            Subroutine::new("second")
                .subroutine_type(SubroutineType::Method)
                .return_type(crate::ast::ReturnType::Int)
                .add_statement(Statement::return_expr(Expr::int(3))),
        );

    let result = compile_class(&class).unwrap();

    let expected: Vec<String> = r#"
        function Adder.first 0
        push argument 0
        pop pointer 0
        push pointer 0
        call Adder.second 1
        return
        function Adder.second 0
        push argument 0
        pop pointer 0
        push constant 3
        return
    "#
    .trim()
    .split('\n')
    .map(|s| s.trim().to_owned())
    .collect();

    assert!(
        contains_commands(&result, &expected),
        "Result: {:?}",
        result
    );
}

// Write the do version of above
#[test]
fn test_chained_methods_using_do() {
    /*
    class Adder {
        constructor Adder new() {
            return this;
        }

        method void first() {
            do second();
            return;
        }

        method void second() {
            return;
        }
    }
     */
    let class = Class::new("Adder")
        .add_subroutine(
            Subroutine::new("new")
                .subroutine_type(crate::ast::SubroutineType::Constructor)
                .return_type(crate::ast::ReturnType::ClassName("Adder".to_string()))
                .add_statement(Statement::return_expr(Expr::this())),
        )
        .add_subroutine(
            Subroutine::new("first")
                .subroutine_type(SubroutineType::Method)
                .add_statement(Statement::do_statement().name("second").as_statement())
                .add_statement(Statement::return_void()),
        )
        .add_subroutine(
            Subroutine::new("second")
                .subroutine_type(SubroutineType::Method)
                .add_statement(Statement::return_void()),
        );

    let result = compile_class(&class).unwrap();

    let expected: Vec<String> = r#"
        function Adder.first 0
        push argument 0
        pop pointer 0
        push pointer 0
        call Adder.second 1
        pop temp 0
        push constant 0
        return
        function Adder.second 0
        push argument 0
        pop pointer 0
        push constant 0
        return
    "#
    .trim()
    .split('\n')
    .map(|s| s.trim().to_owned())
    .collect();

    assert!(
        contains_commands(&result, &expected),
        "Result: {:?}",
        result
    );
}

#[test]
fn compile_array_test() {
    /*
       class Main {
           function void main() {
               do Output.printString("abc");
           }
       }
    */
    let class = Class::new("Main").add_subroutine(
        Subroutine::new("main")
            .add_statement(
                Statement::do_statement()
                    .set_target("Output")
                    .name("printString")
                    .add_parameter(Expr::string("abc"))
                    .as_statement(),
            )
            .add_statement(Statement::return_void()),
    );

    let result = compile_class(&class).unwrap();

    let expected: Vec<String> = r#"
        function Main.main 0
        push constant 3
        call String.new 1
        push constant 97
        call String.appendChar 2
        push constant 98
        call String.appendChar 2
        push constant 99
        call String.appendChar 2
        call Output.printString 1
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
fn test_array_values() {
    /*
       class Main {
           function void main() {
               var Array a;
               let a = Array.new(5);
               let a[2] = 3;
               do Output.printInt(a[2]);
               return;
           }
       }
    */
    let class = Class::new("Main").add_subroutine(
        Subroutine::new("main")
            .add_statement(
                Statement::var()
                    .add_var(Variable::new("a", VariableType::Array))
                    .as_statement(),
            )
            .add_statement(
                Statement::let_statement()
                    .id(VariableRef::new("a"))
                    .value(
                        Expr::call()
                            .set_target("Array")
                            .name("new")
                            .add_parameter(Expr::int(5))
                            .as_expr(),
                    )
                    .as_statement(),
            )
            .add_statement(
                Statement::let_statement()
                    .id(VariableRef::new_with_index("a", Expr::int(2)))
                    .value(Expr::int(3))
                    .as_statement(),
            )
            .add_statement(
                Statement::do_statement()
                    .set_target("Output")
                    .name("printInt")
                    .add_parameter(Expr::var(VariableRef::new_with_index("a", Expr::int(2))))
                    .as_statement(),
            )
            .add_statement(Statement::return_void()),
    );

    let result = compile_class(&class).unwrap();

    let expected: Vec<String> = r#"
        function Main.main 1
        push constant 5
        call Array.new 1
        pop local 0
        push local 0
        push constant 2
        add
        push constant 3
        pop temp 0
        pop pointer 1
        push temp 0
        pop that 0
        push local 0
        push constant 2
        add
        pop pointer 1
        push that 0
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
