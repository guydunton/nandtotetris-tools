#[derive(Debug, PartialEq)]
pub struct Stmt {
    pub operation: Operation,
    pub text: String,
}

#[derive(Debug, PartialEq)]
pub enum Operation {
    Pop(Address),
    Push(Address),
    Add,
    Sub,
    Neg,
    Eq,
    Gt,
    Lt,
    And,
    Or,
    Not,
}

#[derive(Debug, PartialEq)]
pub struct Address {
    pub memory_segment: MemorySegment,
    pub address: u32,
}

#[derive(Debug, PartialEq)]
pub enum MemorySegment {
    Constant,
    Local,
    Arguments,
    This,
    That,
    Static,
    Pointer,
    Temp,
}
