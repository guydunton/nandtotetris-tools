use super::{translate_pop::translate_pop, translate_push::translate_push};
use crate::ast::{Function, Operation, Stmt};

pub fn translate_ast(ast: Vec<Stmt>, file_name: &str) -> Result<String, String> {
    let mut output = vec![];
    let mut eq_counter = 0;
    let mut gt_counter = 0;
    let mut lt_counter = 0;
    let mut return_counter = 0;
    for stmt in ast {
        let mut asm_lines = match stmt.operation {
            Operation::Push(address) => translate_push(&address, file_name)?,
            Operation::Pop(address) => translate_pop(&address, file_name)?,
            Operation::Add => translate_add(),
            Operation::Sub => translate_sub(),
            Operation::Neg => translate_neg(),
            Operation::Eq => translate_eq(&mut eq_counter),
            Operation::Gt => translate_gt(&mut gt_counter),
            Operation::Lt => translate_lt(&mut lt_counter),
            Operation::And => translate_and(),
            Operation::Or => translate_or(),
            Operation::Not => translate_not(),
            Operation::Label(label) => translate_label(&label),
            Operation::ConditionalJump(label) => translate_if_goto(&label),
            Operation::Jump(label) => translate_goto(&label),
            Operation::Function(function) => translate_function(&function),
            Operation::Return => translate_return(&mut return_counter),
        };
        output.push(format!("// {}", stmt.text));
        output.append(&mut asm_lines);
    }

    Ok(output.join("\n"))
}

fn translate_add() -> Vec<String> {
    let mut asm = Vec::new();

    asm.push("@SP".to_string());
    asm.push("AM=M-1".to_string());
    asm.push("D=M".to_string());
    asm.push("A=A-1".to_string());
    asm.push("M=D+M".to_string());

    asm
}
fn translate_sub() -> Vec<String> {
    let mut asm = Vec::new();

    asm.push("@SP".to_string());
    asm.push("AM=M-1".to_string());
    asm.push("D=M".to_string());
    asm.push("A=A-1".to_string());
    asm.push("M=M-D".to_string());

    asm
}
fn translate_neg() -> Vec<String> {
    let mut asm = Vec::new();

    asm.push("@SP".to_owned());
    asm.push("A=M-1".to_owned());
    asm.push("M=-M".to_owned());

    asm
}

fn translate_eq(eq_counter: &mut i32) -> Vec<String> {
    let mut asm = Vec::new();

    asm.push("@SP".to_owned());
    asm.push("AM=M-1".to_owned());
    asm.push("D=M".to_owned());
    asm.push("A=A-1".to_owned());
    asm.push("MD=D-M".to_owned());
    asm.push("M=M-1".to_owned());
    asm.push(format!("@EQ_END_{}", *eq_counter));
    asm.push("D;JEQ".to_owned());
    asm.push("@SP".to_owned());
    asm.push("A=M-1".to_owned());
    asm.push("M=0".to_owned());
    asm.push(format!("(EQ_END_{})", *eq_counter));

    *eq_counter += 1;
    asm
}

fn translate_gt(gt_counter: &mut i32) -> Vec<String> {
    let mut asm = Vec::new();

    asm.push("@SP".to_owned());
    asm.push("AM=M-1".to_owned());
    asm.push("D=M".to_owned());
    asm.push("A=A-1".to_owned());
    asm.push("D=M-D".to_owned());
    asm.push("M=-1".to_owned());
    asm.push(format!("@GT_END_{}", *gt_counter));
    asm.push("D;JGT".to_owned());
    asm.push("@SP".to_owned());
    asm.push("A=M-1".to_owned());
    asm.push("M=0".to_owned());
    asm.push(format!("(GT_END_{})", *gt_counter));

    *gt_counter += 1;
    asm
}

fn translate_lt(lt_counter: &mut i32) -> Vec<String> {
    let mut asm = Vec::new();

    asm.push("@SP".to_owned());
    asm.push("AM=M-1".to_owned());
    asm.push("D=M".to_owned());
    asm.push("A=A-1".to_owned());
    asm.push("D=M-D".to_owned());
    asm.push("M=-1".to_owned());
    asm.push(format!("@LT_END_{}", *lt_counter));
    asm.push("D;JLT".to_owned());
    asm.push("@SP".to_owned());
    asm.push("A=M-1".to_owned());
    asm.push("M=0".to_owned());
    asm.push(format!("(LT_END_{})", *lt_counter));

    *lt_counter += 1;
    asm
}

fn translate_and() -> Vec<String> {
    let mut asm = Vec::new();

    asm.push("@SP".to_owned());
    asm.push("AM=M-1".to_owned());
    asm.push("D=M".to_owned());
    asm.push("A=A-1".to_owned());
    asm.push("M=D&M".to_owned());

    asm
}

fn translate_or() -> Vec<String> {
    let mut asm = Vec::new();

    asm.push("@SP".to_owned());
    asm.push("AM=M-1".to_owned());
    asm.push("D=M".to_owned());
    asm.push("A=A-1".to_owned());
    asm.push("M=D|M".to_owned());

    asm
}

fn translate_not() -> Vec<String> {
    let mut asm = Vec::new();

    asm.push("@SP".to_owned());
    asm.push("A=M-1".to_owned());
    asm.push("M=!M".to_owned());

    asm
}

fn translate_label(label: &str) -> Vec<String> {
    let mut asm = Vec::new();

    asm.push(format!("({})", label));

    asm
}

fn translate_if_goto(label: &str) -> Vec<String> {
    let mut asm = Vec::new();

    asm.push("@SP".to_owned());
    asm.push("AM=M-1".to_owned());
    asm.push("D=M".to_owned());
    asm.push(format!("@{}", label));
    asm.push("D;JNE".to_owned());

    asm
}

fn translate_goto(label: &str) -> Vec<String> {
    let mut asm = Vec::new();

    asm.push(format!("@{}", label));
    asm.push("0;JMP".to_owned());

    asm
}

fn translate_function(function: &Function) -> Vec<String> {
    let mut asm = Vec::new();

    asm.push(format!("({})", function.name));
    asm.push("@LCL".to_owned());
    asm.push("A=M".to_owned());

    for _ in 0..function.num_locals {
        asm.push("M=0".to_owned());
        asm.push("A=A+1".to_owned());
    }

    asm
}

fn translate_return(return_counter: &mut i32) -> Vec<String> {
    let mut asm = Vec::new();

    // *ARG = pop()
    asm.push("@SP".to_owned());
    asm.push("A=M-1".to_owned());
    asm.push("D=M".to_owned());
    asm.push("@ARG".to_owned());
    asm.push("A=M".to_owned());
    asm.push("M=D".to_owned());

    // SP = ARG + 1
    asm.push("@ARG".to_owned());
    asm.push("D=M+1".to_owned());
    asm.push("@SP".to_owned());
    asm.push("M=D".to_owned());

    // endFrame = LCL
    asm.push("@LCL".to_owned());
    asm.push("D=M".to_owned());
    asm.push("@R13".to_owned());
    asm.push("M=D".to_owned());

    // destination = THAT
    asm.push("@THAT".to_owned());
    asm.push("D=A".to_owned());
    asm.push("@R14".to_owned());
    asm.push("M=D".to_owned());

    // THAT = *(endFrame - 1)
    // THIS = *(endFrame - 2)
    // ARG = *(endFrame - 3)
    // LCL = *(endFrame - 4)
    asm.push(format!("(RETURN_DMA_START_{})", return_counter));
    asm.push("@R13".to_owned());
    asm.push("AM=M-1".to_owned());
    asm.push("D=M".to_owned());
    asm.push("@R14".to_owned());
    asm.push("A=M".to_owned());
    asm.push("M=D".to_owned());
    asm.push("@R14".to_owned());
    asm.push("M=M-1".to_owned());

    // if R14 > 0 goto RETURN_DMA_START
    asm.push("D=M".to_owned());
    asm.push(format!("@RETURN_DMA_START_{}", return_counter));
    asm.push("D;JGT".to_owned());

    // goto retAddress
    asm.push("@R13".to_owned());
    asm.push("A=M-1".to_owned());
    asm.push("0;JMP".to_owned());

    *return_counter += 1;

    asm
}
