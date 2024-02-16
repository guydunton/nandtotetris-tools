use std::collections::HashMap;

pub fn create_symbol_table() -> HashMap<String, u16> {
    let mut symbol_table = HashMap::new();

    symbol_table.insert("R0".to_owned(), 0);
    symbol_table.insert("R1".to_owned(), 1);
    symbol_table.insert("R2".to_owned(), 2);
    symbol_table.insert("R3".to_owned(), 3);
    symbol_table.insert("R4".to_owned(), 4);
    symbol_table.insert("R5".to_owned(), 5);
    symbol_table.insert("R6".to_owned(), 6);
    symbol_table.insert("R7".to_owned(), 7);
    symbol_table.insert("R8".to_owned(), 8);
    symbol_table.insert("R9".to_owned(), 9);
    symbol_table.insert("R10".to_owned(), 10);
    symbol_table.insert("R11".to_owned(), 11);
    symbol_table.insert("R12".to_owned(), 12);
    symbol_table.insert("R13".to_owned(), 13);
    symbol_table.insert("R14".to_owned(), 14);
    symbol_table.insert("R15".to_owned(), 15);

    symbol_table.insert("SCREEN".to_owned(), 16384);
    symbol_table.insert("KBD".to_owned(), 24576);

    symbol_table.insert("SP".to_owned(), 0);
    symbol_table.insert("LCL".to_owned(), 1);
    symbol_table.insert("ARG".to_owned(), 2);
    symbol_table.insert("THIS".to_owned(), 3);
    symbol_table.insert("THAT".to_owned(), 4);

    symbol_table
}
