#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Address {
    Value(u16),
    Symbol(String),
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Dest {
    NULL = 0,
    M = 1,
    D = 2,
    MD = 3,
    A = 4,
    AM = 5,
    AD = 6,
    AMD = 7,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Jump {
    NULL = 0,
    JGT = 1,
    JEQ = 2,
    JGE = 3,
    JLT = 4,
    JNE = 5,
    JLE = 6,
    JMP = 7,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Operation {
    Zero,
    One,
    MinusOne,
    D,
    A,
    M,
    NotD,
    NotA,
    NotM,
    MinusD,
    MinusA,
    MinusM,
    DPlus1,
    APlus1,
    MPlus1,
    DMinus1,
    AMinus1,
    MMinus1,
    DPlusA,
    DPlusM,
    DMinusA,
    DMinusM,
    AMinusD,
    MMinusD,
    DAndA,
    DAndM,
    DOrA,
    DOrM,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Command {
    pub dest: Option<Dest>,
    pub operation: Operation,
    pub jump: Option<Jump>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Stmt {
    A(Address),
    C(Command),
    Label(String),
    Empty,
}
