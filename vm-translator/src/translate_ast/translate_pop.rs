use crate::ast::{Address, MemorySegment};

/*
 * POP local 2 implementation using the top of the stack to store
 * the address variable

// pop local 2

// D = address
@2
D=A
@LCL
D=D+M

// *(SP + 1) = D; SP--
@SP
A=M
M=D

// D = *(SP)
A=A-1
D=M

// *address = D
A=A+1
A=M
M=D

 */

pub fn translate_pop(address: &Address, file_name: &str) -> Result<Vec<String>, String> {
    let mut asm = vec![];

    match address.memory_segment {
        MemorySegment::Arguments
        | MemorySegment::Local
        | MemorySegment::This
        | MemorySegment::That => {
            asm.push(format!("@{}", address.address));
            asm.push("D=A".to_owned());
            asm.push(format!("{}", segment_to_var(address, file_name)?));
            asm.push("D=D+M".to_owned());
            asm.push("@SP".to_owned());
            asm.push("M=M-1".to_owned());
            asm.push("A=M+1".to_owned());
            asm.push("M=D".to_owned());
            asm.push("A=A-1".to_owned());
            asm.push("D=M".to_owned());
            asm.push("A=A+1".to_owned());
            asm.push("A=M".to_owned());
            asm.push("M=D".to_owned());
        }
        MemorySegment::Pointer | MemorySegment::Static | MemorySegment::Temp => {
            asm.push("@SP".to_string());
            asm.push("M=M-1".to_string());
            asm.push("A=M".to_string());
            asm.push("D=M".to_string());
            asm.push(format!("{}", segment_to_var(address, file_name)?));
            asm.push("M=D".to_string());
        }
        MemorySegment::Constant => {
            return Err(format!(
                "Invalid pop statement: pop constant {}",
                address.address
            ));
        }
    }

    Ok(asm)
}

fn segment_to_var(address: &Address, file_name: &str) -> Result<String, String> {
    match address.memory_segment {
        MemorySegment::Arguments => Ok("@ARG".to_string()),
        MemorySegment::Local => Ok("@LCL".to_string()),
        MemorySegment::This => Ok("@THIS".to_string()),
        MemorySegment::That => Ok("@THAT".to_string()),
        MemorySegment::Temp => address_to_temp(address.address),
        MemorySegment::Static => Ok(format!("@{}.{}", file_name, address.address)),
        MemorySegment::Pointer => match address.address {
            0 => Ok("@THIS".to_string()),
            1 => Ok("@THAT".to_string()),
            _ => Err(format!("Invalid pop pointer address {}", address.address)),
        },
        _ => Err("Unable to convert memory segment to address".to_string()),
    }
}

fn address_to_temp(address: u32) -> Result<String, String> {
    let register = match address {
        0 => "@R5".to_owned(),
        1 => "@R6".to_owned(),
        2 => "@R7".to_owned(),
        3 => "@R8".to_owned(),
        4 => "@R9".to_owned(),
        5 => "@R10".to_owned(),
        6 => "@R11".to_owned(),
        7 => "@R12".to_owned(),
        _ => {
            return Err(format!(
                "Address {} outside scope of temp registers",
                address
            ));
        }
    };
    Ok(register)
}

#[test]
fn test_pop_local() {
    let asm = translate_pop(
        &Address {
            memory_segment: MemorySegment::Local,
            address: 2,
        },
        "Vars",
    )
    .unwrap();

    assert_eq!(
        asm,
        vec![
            "@2", "D=A", "@LCL", "D=D+M", "@SP", "M=M-1", "A=M+1", "M=D", "A=A-1", "D=M", "A=A+1",
            "A=M", "M=D"
        ]
    )
}

#[test]
fn test_pop_argument() {
    let asm = translate_pop(
        &Address {
            memory_segment: MemorySegment::Arguments,
            address: 6,
        },
        "Vars",
    )
    .unwrap();

    assert_eq!(
        asm,
        vec![
            "@6", "D=A", "@ARG", "D=D+M", "@SP", "M=M-1", "A=M+1", "M=D", "A=A-1", "D=M", "A=A+1",
            "A=M", "M=D"
        ]
    )
}

#[test]
fn test_pop_this() {
    let asm = translate_pop(
        &Address {
            memory_segment: MemorySegment::This,
            address: 2,
        },
        "Vars",
    )
    .unwrap();

    assert_eq!(
        asm,
        vec![
            "@2", "D=A", "@THIS", "D=D+M", "@SP", "M=M-1", "A=M+1", "M=D", "A=A-1", "D=M", "A=A+1",
            "A=M", "M=D"
        ]
    )
}

#[test]
fn test_pop_that() {
    let asm = translate_pop(
        &Address {
            memory_segment: MemorySegment::That,
            address: 6,
        },
        "Vars",
    )
    .unwrap();

    assert_eq!(
        asm,
        vec![
            "@6", "D=A", "@THAT", "D=D+M", "@SP", "M=M-1", "A=M+1", "M=D", "A=A-1", "D=M", "A=A+1",
            "A=M", "M=D"
        ]
    )
}

#[test]
fn test_pop_static() {
    /*
        // Implementation of
        // pop static 6

    // SP--; D=SP
    @SP
    M=M-1
    A=M
    D=M

    // *Vars.6 = D
    @Vars.6
    M=D
         */
    let asm = translate_pop(
        &Address {
            memory_segment: MemorySegment::Static,
            address: 6,
        },
        "Vars",
    )
    .unwrap();

    assert_eq!(asm, vec!["@SP", "M=M-1", "A=M", "D=M", "@Vars.6", "M=D"])
}

#[test]
fn test_pop_temp() {
    let asm = translate_pop(
        &Address {
            memory_segment: MemorySegment::Temp,
            address: 1,
        },
        "Vars",
    )
    .unwrap();

    assert_eq!(asm, vec!["@SP", "M=M-1", "A=M", "D=M", "@R6", "M=D"])
}

#[test]
fn test_pop_pointer_0() {
    let asm = translate_pop(
        &Address {
            memory_segment: MemorySegment::Pointer,
            address: 0,
        },
        "Vars",
    )
    .unwrap();

    assert_eq!(asm, vec!["@SP", "M=M-1", "A=M", "D=M", "@THIS", "M=D"])
}

#[test]
fn test_pop_pointer_1() {
    let asm = translate_pop(
        &Address {
            memory_segment: MemorySegment::Pointer,
            address: 1,
        },
        "Vars",
    )
    .unwrap();

    assert_eq!(asm, vec!["@SP", "M=M-1", "A=M", "D=M", "@THAT", "M=D"])
}

#[test]
fn test_pop_pointer_out_of_bounds() {
    let asm = translate_pop(
        &Address {
            memory_segment: MemorySegment::Pointer,
            address: 2,
        },
        "Vars",
    );
    assert!(asm.is_err());
}

#[test]
fn test_pop_constant() {
    let asm = translate_pop(
        &Address {
            memory_segment: MemorySegment::Constant,
            address: 6,
        },
        "Vars",
    );

    assert!(asm.is_err());
}
