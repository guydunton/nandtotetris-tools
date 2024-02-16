use crate::ast::{Address, MemorySegment};

/*

// Example implementation of:
// push local 3

// D=LCL[3]
@3
D=A
@LCL
A=D+M
D=M

// *SP = D
@SP
A=M
M=D

@SP
M=M+1

 */

pub fn translate_push(address: &Address, file_name: &str) -> Result<Vec<String>, String> {
    let mut result = vec![];

    // Setup the fetch of the value into D
    translate_address_fetch(address, file_name, &mut result)?;

    // Set the stack value
    result.push("@SP".to_owned());
    result.push("A=M".to_owned());
    result.push("M=D".to_owned());

    // Increment the stack
    result.push("@SP".to_owned());
    result.push("M=M+1".to_owned());

    Ok(result)
}

fn translate_address_fetch(
    address: &Address,
    file_name: &str,
    asm: &mut Vec<String>,
) -> Result<(), String> {
    match address.memory_segment {
        MemorySegment::Constant => {
            asm.push(format!("@{}", address.address));
            asm.push("D=A".to_owned());
        }
        MemorySegment::Local => {
            asm.push(format!("@{}", address.address));
            asm.push("D=A".to_owned());
            asm.push("@LCL".to_owned());
            asm.push("A=D+M".to_owned());
            asm.push("D=M".to_owned());
        }
        MemorySegment::Arguments => {
            asm.push(format!("@{}", address.address));
            asm.push("D=A".to_owned());
            asm.push("@ARG".to_owned());
            asm.push("A=D+M".to_owned());
            asm.push("D=M".to_owned());
        }
        MemorySegment::This => {
            asm.push(format!("@{}", address.address));
            asm.push("D=A".to_owned());
            asm.push("@THIS".to_owned());
            asm.push("A=D+M".to_owned());
            asm.push("D=M".to_owned());
        }
        MemorySegment::That => {
            asm.push(format!("@{}", address.address));
            asm.push("D=A".to_owned());
            asm.push("@THAT".to_owned());
            asm.push("A=D+M".to_owned());
            asm.push("D=M".to_owned());
        }
        MemorySegment::Static => {
            asm.push(format!("@{}.{}", file_name, address.address));
            asm.push("D=M".to_owned());
        }
        MemorySegment::Temp => {
            asm.push(format!("@{}", address_to_temp(address.address)?));
            asm.push("D=M".to_owned());
        }
        MemorySegment::Pointer => {
            match address.address {
                0 => asm.push("@THIS".to_owned()),
                1 => asm.push("@THAT".to_owned()),
                _ => return Err(format!("Out of range pointer address {}", address.address)),
            }
            asm.push("D=M".to_owned());
        }
    };

    Ok(())
}

fn address_to_temp(address: u32) -> Result<String, String> {
    let register = match address {
        0 => "R5".to_owned(),
        1 => "R6".to_owned(),
        2 => "R7".to_owned(),
        3 => "R8".to_owned(),
        4 => "R9".to_owned(),
        5 => "R10".to_owned(),
        6 => "R11".to_owned(),
        7 => "R12".to_owned(),
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
fn test_push_constant() {
    let asm = translate_push(
        &Address {
            memory_segment: MemorySegment::Constant,
            address: 5,
        },
        "Vars",
    )
    .unwrap();
    assert_eq!(
        asm,
        vec!["@5", "D=A", "@SP", "A=M", "M=D", "@SP", "M=M+1"]
            .into_iter()
            .map(|line| line.to_owned())
            .collect::<Vec<String>>()
    );
}

#[test]
fn test_push_local() {
    let asm = translate_push(
        &Address {
            memory_segment: MemorySegment::Local,
            address: 3,
        },
        "Vars",
    )
    .unwrap();
    assert_eq!(
        asm,
        vec!["@3", "D=A", "@LCL", "A=D+M", "D=M", "@SP", "A=M", "M=D", "@SP", "M=M+1",]
    );
}

#[test]
fn test_push_arg() {
    let asm = translate_push(
        &Address {
            memory_segment: MemorySegment::Arguments,
            address: 5,
        },
        "Vars",
    )
    .unwrap();

    assert_eq!(
        asm,
        vec!["@5", "D=A", "@ARG", "A=D+M", "D=M", "@SP", "A=M", "M=D", "@SP", "M=M+1",]
    );
}

#[test]
fn test_push_this() {
    let asm = translate_push(
        &Address {
            memory_segment: MemorySegment::This,
            address: 2,
        },
        "Vars",
    )
    .unwrap();

    assert_eq!(
        asm,
        vec!["@2", "D=A", "@THIS", "A=D+M", "D=M", "@SP", "A=M", "M=D", "@SP", "M=M+1",]
    );
}

#[test]
fn test_push_that() {
    let asm = translate_push(
        &Address {
            memory_segment: MemorySegment::That,
            address: 4,
        },
        "Vars",
    )
    .unwrap();

    assert_eq!(
        asm,
        vec!["@4", "D=A", "@THAT", "A=D+M", "D=M", "@SP", "A=M", "M=D", "@SP", "M=M+1",]
    );
}

#[test]
fn test_push_static() {
    let asm = translate_push(
        &Address {
            memory_segment: MemorySegment::Static,
            address: 4,
        },
        "Vars",
    )
    .unwrap();

    assert_eq!(
        asm,
        vec!["@Vars.4", "D=M", "@SP", "A=M", "M=D", "@SP", "M=M+1",]
    );
}

#[test]
fn test_push_temp() {
    let asm = translate_push(
        &Address {
            memory_segment: MemorySegment::Temp,
            address: 1,
        },
        "Vars",
    )
    .unwrap();

    assert_eq!(
        asm,
        vec!["@R6", "D=M", "@SP", "A=M", "M=D", "@SP", "M=M+1",]
    );
}

#[test]
fn test_push_temp_fails() {
    let asm = translate_push(
        &Address {
            memory_segment: MemorySegment::Temp,
            address: 9,
        },
        "Vars",
    );
    assert!(asm.is_err());
}

#[test]
fn test_push_pointer_0() {
    let asm = translate_push(
        &Address {
            memory_segment: MemorySegment::Pointer,
            address: 0,
        },
        "Vars",
    )
    .unwrap();

    assert_eq!(
        asm,
        vec!["@THIS", "D=M", "@SP", "A=M", "M=D", "@SP", "M=M+1",]
    );
}

#[test]
fn test_push_pointer_1() {
    let asm = translate_push(
        &Address {
            memory_segment: MemorySegment::Pointer,
            address: 1,
        },
        "Vars",
    )
    .unwrap();

    assert_eq!(
        asm,
        vec!["@THAT", "D=M", "@SP", "A=M", "M=D", "@SP", "M=M+1",]
    );
}
