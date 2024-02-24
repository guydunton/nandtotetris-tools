#[derive(Debug, PartialEq)]
pub struct Stmt {
    pub operation: Operation,
    pub text: String,
}

#[derive(Debug, PartialEq)]
pub enum Operation {
    Pop(Address),
    Push(Address),
    Label(String),
    Function(Function),
    Return,
    ConditionalJump(String),
    Jump(String),
    Add,
    Sub,
    Neg,
    Eq,
    Gt,
    Lt,
    And,
    Or,
    Not,
    // TODO: Implement support for call
}

#[derive(Debug, PartialEq)]
pub struct Address {
    pub memory_segment: MemorySegment,
    pub address: u32,
}

#[derive(Debug, PartialEq)]
pub struct Function {
    pub name: String,
    pub num_locals: u32,
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
