use std::collections::HashMap;

use crate::parser::{Address, Command, Dest, Operation, Stmt};

fn convert_a_statement(address: Address, symbol_table: &HashMap<String, u16>) -> u16 {
    const MASK: u16 = 0b01111111_11111111;
    match address {
        Address::Value(val) => val & MASK,
        Address::Symbol(symbol) => {
            let symbol_value = symbol_table.get(&symbol);
            match symbol_value {
                Some(value) => *value & MASK,
                None => panic!("Unable to find symbol in table {}", symbol),
            }
        }
    }
}

fn convert_operation(operation: Operation) -> u16 {
    match operation {
        Operation::Zero => 0b0101010,
        Operation::One => 0b0111111,
        Operation::MinusOne => 0b0111010,
        Operation::D => 0b0001100,
        Operation::A => 0b0110000,
        Operation::M => 0b1110000,
        Operation::NotD => 0b0001101,
        Operation::NotA => 0b0110001,
        Operation::NotM => 0b1110001,
        Operation::MinusD => 0b0001111,
        Operation::MinusA => 0b0110011,
        Operation::MinusM => 0b1110011,
        Operation::DPlus1 => 0b0011111,
        Operation::APlus1 => 0b0110111,
        Operation::MPlus1 => 0b1110111,
        Operation::DMinus1 => 0b0001110,
        Operation::AMinus1 => 0b0110010,
        Operation::MMinus1 => 0b1110010,
        Operation::DPlusA => 0b0000010,
        Operation::DPlusM => 0b1000010,
        Operation::DMinusA => 0b0010011,
        Operation::DMinusM => 0b1010011,
        Operation::AMinusD => 0b0000111,
        Operation::MMinusD => 0b1000111,
        Operation::DAndA => 0b0000000,
        Operation::DAndM => 0b1000000,
        Operation::DOrA => 0b0010101,
        Operation::DOrM => 0b1010101,
    }
}

fn convert_c_statement(command: Command) -> u16 {
    0b1110_0000_0000_0000 as u16
        | (convert_operation(command.operation) << 6)
        | ((command.dest.unwrap_or(Dest::NULL) as u16) << 3)
        | command.jump.unwrap_or(crate::parser::Jump::NULL) as u16
}

pub fn interpret_ast(statements: &[Stmt], symbol_table: &HashMap<String, u16>) -> Vec<u16> {
    let vals: Vec<u16> = statements
        .iter()
        .map(|s| match s {
            Stmt::A(a_statement) => convert_a_statement(a_statement.clone(), symbol_table),
            Stmt::C(c_statement) => convert_c_statement(c_statement.clone()),
            _ => panic!("Unable to convert label"),
        })
        .collect();

    vals
}

#[test]
fn test_interpret_ast() {
    let symbol_table = crate::symbol_table::create_symbol_table();

    assert_eq!(
        interpret_ast(&vec![Stmt::A(Address::Value(u16::MAX))], &symbol_table),
        vec![0b01111111_11111111]
    );

    assert_eq!(
        interpret_ast(
            &vec![Stmt::A(Address::Symbol("SCREEN".to_string()))],
            &symbol_table
        ),
        vec![0b01000000_00000000]
    );

    assert_eq!(
        interpret_ast(
            &vec![Stmt::C(Command {
                dest: Some(Dest::M),
                operation: Operation::Zero,
                jump: None
            })],
            &symbol_table
        ),
        vec![0b11101010_10001000]
    );
}
