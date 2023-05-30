use crate::{inst, prog, Instruction, InstructionKind, VM};
use std::fs;

#[test]
fn load_from_memmory() {
    use InstructionKind::*;
    let program = prog!{
        Push 1,
        Push 2,
        Sum,
    };

    let expected_top = 3;
    let expected_stack_size = 1;

    let mut state = VM::init();
    state.debug = (true, true);
    let load_res = state.load_from_memmory(&program);
    assert!(load_res.is_ok());
    assert!(state.program_size == program.len());
    let execute_res = state.execute();
    assert!(execute_res.is_ok());
    assert!(state.stack_size == expected_stack_size);
    assert!(state.stack[state.stack_size - 1] == expected_top);
}

#[test]
fn serialize_and_load_from_file() {
    let se_inst = Instruction {
        kind: InstructionKind::Push,
        operand: Some(69),
    }
    .serialize();

    assert!(se_inst.is_ok());

    let write = fs::write("tests/ser_test", se_inst.unwrap());
    assert!(write.is_ok());

    let mut state = VM::init();
    let res = state.deserialize_from_file("tests/ser_test");
    assert!(res.is_ok());
    assert!(state.program_size == 1);
    assert!(state.program[state.program_size - 1].kind == InstructionKind::Push);
    assert!(state.program[state.program_size - 1].operand == Some(69));
}

#[test]
fn disassemble() {
    let prog = "
клади 2
клади 3
сума
копію
рівн";

    let file = "tests/dis_test";
    fs::write(
        file,
        prog.as_bytes(),
    )
    .expect("write to test file");
    let mut state = VM::init();
    state.disassemble_from_file(file).expect("disassemble");
    state.execute().expect("exec program");
    assert!(state.program_size == 5);
    assert!(state.stack_size == 3);
    assert!(state.stack[state.stack_size - 1] == 1);
}
