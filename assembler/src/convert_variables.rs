use crate::parser::{Address, Stmt};
use std::collections::HashMap;

pub fn find_variables(statements: &Vec<Stmt>, symbol_table: &mut HashMap<String, u16>) {
    let mut counter = 16u16;

    for stmt in statements {
        if let Stmt::A(address) = stmt {
            if let Address::Symbol(symbol) = address {
                if !symbol_table.contains_key(symbol) {
                    symbol_table.insert(symbol.clone(), counter);
                    counter += 1;
                }
            }
        }
    }
}

#[test]
fn test_convert_variables() {
    let mut symbol_table = crate::symbol_table::create_symbol_table();

    let statements = vec![
        Stmt::A(Address::Symbol("i".to_string())),
        Stmt::A(Address::Symbol("i2".to_string())),
    ];

    find_variables(&statements, &mut symbol_table);

    assert_eq!(*symbol_table.get("i").unwrap(), 16);
    assert_eq!(*symbol_table.get("i2").unwrap(), 17);
}
