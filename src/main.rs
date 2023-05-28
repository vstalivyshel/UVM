mod utils;
use std::{
    fs,
    io::{self, Read, Write},
    isize,
    path::Path,
};

const VM_STACK_CAPACITY: usize = 1024;
const PROGRAM_INST_CAPACITY: usize = 1024;
const INST_CHUNCK_SIZE: usize = 9;

// TODO: test to write some instructions to file and load
// TODO: macro rule for creating enumeration along with const variants count and index array
// TODO: need a way to test the instructions

macro_rules! inst {
    ($kind:tt $operand:expr) => {
        Instruction {
            kind: $kind,
            operand: $operand,
        }
    };

    ($kind:expr) => {
        Instruction {
            kind: $kind,
            operand: 0,
        }
    };
}

#[derive(Copy, Clone, Debug)]
pub enum Panic {
    StackOverflow,
    StackUnderflow,
    IntegerOverflow,
    InvalidOperand,
    InstLimitkOverflow,
    InvalidInstruction,
    ReadFile,
    WriteToFile,
    DivByZero,
}

#[derive(Copy, Clone, Debug)]
enum InstructionKind {
    Nop,
    Drop,
    Dup,
    Push,
    DupAt,
    JumpIf,
    Jump,
    Eq,
    Sub,
    Mul,
    Div,
    Sum,
}

#[derive(Copy, Clone, Debug)]
pub struct Instruction {
    kind: InstructionKind,
    operand: isize,
}

type SerializedInst = [u8; INST_CHUNCK_SIZE];

impl Instruction {
    const NUM_INST: usize = 12;

    const INST_KIND: [InstructionKind; Self::NUM_INST] = [
        InstructionKind::Nop,
        InstructionKind::Drop,
        InstructionKind::Dup,
        InstructionKind::Push,
        InstructionKind::DupAt,
        InstructionKind::JumpIf,
        InstructionKind::Jump,
        InstructionKind::Eq,
        InstructionKind::Sub,
        InstructionKind::Mul,
        InstructionKind::Div,
        InstructionKind::Sum,
    ];

    fn nop() -> Self {
        Self {
            kind: InstructionKind::Nop,
            operand: 0,
        }
    }

    fn serialize(&self) -> [u8; INST_CHUNCK_SIZE] {
        let mut se = [0; 9];
        se[0] = self.kind as u8;
        use InstructionKind::*;
        match self.kind {
            Push | DupAt | JumpIf | Jump => {
                for (i, b) in self.operand.to_be_bytes().into_iter().enumerate() {
                    se[i + 1] = b;
                }

                se
            }
            _ => se,
        }
    }

    fn deserialize(se: [u8; INST_CHUNCK_SIZE]) -> Result<Instruction, Panic> {
        let kind = se[0];
        if kind as usize > Self::NUM_INST {
            return Err(Panic::InvalidInstruction);
        }

        let kind = Self::INST_KIND[kind as usize];

        Ok(Instruction {
            kind,
            operand: isize::from_be_bytes(se[1..INST_CHUNCK_SIZE].try_into().unwrap()),
        })
    }
}

pub struct VM {
    stack: [isize; VM_STACK_CAPACITY],
    stack_size: usize,
    program: [Instruction; PROGRAM_INST_CAPACITY],
    program_size: usize,
    inst_ptr: usize,
}

impl VM {
    fn init() -> Self {
        Self {
            stack: [0; VM_STACK_CAPACITY],
            stack_size: 0,

            program: [Instruction::nop(); PROGRAM_INST_CAPACITY],
            program_size: 0,
            inst_ptr: 0,
        }
    }

    fn program_save_to_file<P: AsRef<Path>>(&self, file: P) -> Result<(), Panic> {
        let mut buf = Vec::<SerializedInst>::new();

        for (i, inst) in self.program.iter().enumerate() {
            buf.push(inst.serialize());
        }

        if let Err(_) = fs::write(file.as_ref(), &buf.concat()) {
            return Err(Panic::WriteToFile);
        }

        Ok(())
    }

    fn program_load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Panic> {
        let mut f = match fs::OpenOptions::new().read(true).open(path.as_ref()) {
            Ok(f) => f,
            Err(_) => return Err(Panic::ReadFile),
        };

        let mut buf = Vec::<u8>::new();
        if let Err(_) = f.read(&mut buf) {
            return Err(Panic::ReadFile);
        }

        let mut program = Vec::<Instruction>::new();
        for inst in buf.chunks(9) {
            program.push(Instruction::deserialize(inst.try_into().unwrap())?);
        }

        self.program_load_from_memmory(program.as_slice())
    }

    fn program_load_from_memmory(&mut self, program: &[Instruction]) -> Result<(), Panic> {
        let program_size = program.len();
        self.program_size = program_size;

        if program_size > PROGRAM_INST_CAPACITY {
            return Err(Panic::InstLimitkOverflow);
        }

        for (i, inst) in program.iter().enumerate() {
            self.program[i] = *inst;
        }

        Ok(())
    }

    fn program_execute(&mut self) -> Result<(), Panic> {
        while self.inst_ptr < self.program_size {
            self.instruction_execute()?;
            self.inst_ptr += 1;
        }

        Ok(())
    }

    fn instruction_execute(&mut self) -> Result<(), Panic> {
        fn push_from<F>(state: &mut VM, f: F) -> Result<(), Panic>
        where
            F: Fn(isize, isize) -> Result<isize, Panic>,
        {
            let (a, b) = (state.stack_pop()?, state.stack_pop()?);
            state.stack_push(f(a, b)?)
        }

        let inst = self.program[self.inst_ptr];
        use InstructionKind::*;
        match inst.kind {
            Nop => Ok(()),
            Push => self.stack_push(inst.operand),
            Drop => {
                self.stack_size -= 1;
                self.stack[self.stack_size] = 0;
                Ok(())
            }
            DupAt => {
                let addr = inst.operand + 1;
                if addr < 0 || addr as usize > self.inst_ptr {
                    return Err(Panic::InvalidOperand);
                }

                self.stack_push(self.stack[addr as usize])
            }
            Dup => {
                let target = self.stack_pop()?;
                self.stack_push(target)?;
                self.stack_push(target)
            }
            JumpIf => {
                if self.stack_size < 1 {
                    return Err(Panic::StackUnderflow);
                }

                if self.stack[self.stack_size] != 0 {
                    self.inst_ptr = inst.operand as usize;
                }

                Ok(())
            }
            Jump => {
                let addr = inst.operand;
                if addr < 0 || addr as usize > self.program_size {
                    return Err(Panic::InvalidOperand);
                }

                self.inst_ptr = addr as usize;

                Ok(())
            }
            Eq => {
                if self.stack_size < 2 {
                    return Err(Panic::StackUnderflow);
                }

                let a = self.stack[self.stack_size];
                let b = self.stack[self.stack_size - 1];

                self.stack_push(if a == b { 1 } else { 0 })
            }
            Sum => push_from(self, |a, b| Ok(b + a)),
            Sub => push_from(self, |a, b| Ok(b - a)),
            Mul => push_from(self, |a, b| Ok(b * a)),
            Div => push_from(self, |a, b| {
                if a == 0 {
                    Err(Panic::DivByZero)
                } else {
                    Ok(b / a)
                }
            }),
            _ => Err(Panic::InvalidInstruction),
        }
    }

    fn stack_push(&mut self, value: isize) -> Result<(), Panic> {
        if value > isize::MAX || value < isize::MIN {
            Err(Panic::IntegerOverflow)
        } else if self.stack_size == VM_STACK_CAPACITY {
            Err(Panic::StackOverflow)
        } else {
            self.stack[self.stack_size] = value;
            self.stack_size += 1;
            Ok(())
        }
    }

    fn stack_pop(&mut self) -> Result<isize, Panic> {
        if self.stack_size == 0 {
            Err(Panic::StackUnderflow)
        } else {
            self.stack_size -= 1;
            let value = self.stack[self.stack_size];
            self.stack[self.stack_size] = 0;
            Ok(value)
        }
    }

    fn stack_dump(&self, from: usize, to: usize) {
        println!("СТЕК [{}]", self.stack_size);
        for i in from..to {
            println!("    АДР: {i} ЗНАЧ: {}", self.stack[i]);
        }
    }

    fn program_dump(&self, from: usize, to: usize) {
        for i in from..to {
            println!("ІНСТ({i}): {}", self.program[i]);
        }
    }
}

fn main() {
    use InstructionKind::*;

    // let program = [
    //     inst!(Push 1),
    //     inst!(Dup),
    // ];

    let mut state = VM::init();
    state.program_load_from_file("./test").unwrap();
    // if let Err(vm_err) = state.program_load_from_memmory(&program) {
    //     eprintln!("[!] ПАНІКА: {vm_err}");
    // }

    state.program_save_to_file("./test").unwrap();

    if let Err(vm_err) = state.program_execute() {
        eprintln!("[!] ПАНІКА: {vm_err}");
    }

    state.program_dump(0, state.stack_size);
    state.stack_dump(0, state.program_size);
}
