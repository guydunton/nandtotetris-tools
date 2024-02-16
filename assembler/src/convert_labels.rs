use std::collections::HashMap;

use crate::parser::Stmt;

pub fn find_labels(statements: &Vec<Stmt>, symbol_table: &mut HashMap<String, u16>) {
    let mut label_count = 0;
    for (line_number, stmt) in statements.iter().enumerate() {
        if let Stmt::Label(name) = stmt {
            symbol_table.insert(name.clone(), (line_number - label_count) as u16);
            label_count += 1;
        }
    }
}

pub fn remove_all_labels(statements: Vec<Stmt>) -> Vec<Stmt> {
    statements
        .into_iter()
        .filter(|stmt| !matches!(stmt, Stmt::Label(_)))
        .collect()
}

#[test]
fn test_find_labels() {
    let mut symbol_table = crate::symbol_table::create_symbol_table();

    let statements = vec![
        Stmt::A(crate::parser::Address::Value(21)),
        Stmt::Label("FIRST_LABEL".to_string()),
        Stmt::A(crate::parser::Address::Value(43)),
        Stmt::Label("SECOND_LABEL".to_string()),
        Stmt::A(crate::parser::Address::Value(86)),
    ];

    find_labels(&statements, &mut symbol_table);
    assert_eq!(*symbol_table.get("FIRST_LABEL").unwrap(), 1);
    assert_eq!(*symbol_table.get("SECOND_LABEL").unwrap(), 2);
}

#[test]
fn test_remove_all_labels() {
    let mut statements = vec![
        Stmt::Label("FIRST_LABEL".to_string()),
        Stmt::A(crate::parser::Address::Value(21)),
        Stmt::Label("SECOND_LABEL".to_string()),
        Stmt::A(crate::parser::Address::Value(32)),
    ];

    statements = remove_all_labels(statements);

    assert_eq!(statements.len(), 2);
    assert_eq!(statements[0], Stmt::A(crate::parser::Address::Value(21)));
    assert_eq!(statements[1], Stmt::A(crate::parser::Address::Value(32)));
}
