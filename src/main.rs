mod array;
mod instruction;
mod utils;

use array::Array;

use crate::instruction::{Instruction, InstructionKind, SerializedInst, Value, INST_CHUNCK_SIZE};
use std::{fs, io, isize, path::Path};

const VM_STACK_CAPACITY: usize = 1024;
const PROGRAM_INST_CAPACITY: usize = 1024;

#[derive(Debug)]
pub enum Panic {
    StackOverflow,
    StackUnderflow,
    IntegerOverflow,
    InvalidOperandValue {
        operand: String,
        inst: InstructionKind,
    },
    IlligalInstructionOperands {
        inst: InstructionKind,
        val_a: Value,
        val_b: Value,
    },
    InvalidInstruction(String),
    InvalidBinaryInstruction,
    InstLimitkOverflow(usize),
    ReadFileErr(io::Error),
    WriteToFileErr(io::Error),
    DivByZero,
}

pub struct VM {
    stack: Array<Value, VM_STACK_CAPACITY>,
    program: Array<Instruction, PROGRAM_INST_CAPACITY>,
    inst_ptr: usize,
    inst_limit: Option<usize>,
    debug: (bool, bool),
}

impl VM {
    fn init() -> Self {
        Self {
            stack: Array::new(),
            program: Array::new(),
            inst_ptr: 0,
            inst_limit: None,
            debug: (false, false),
        }
    }

    pub fn load_from_memmory(&mut self, program: &[Instruction]) -> Result<(), Panic> {
        let len = program.len();
        if len > PROGRAM_INST_CAPACITY {
            return Err(Panic::InstLimitkOverflow(len));
        }

        self.program.size = len;
        self.program.items[..len].copy_from_slice(&program[..len]);

        Ok(())
    }

    fn deserialize_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Panic> {
        let buf = match fs::read(path.as_ref()) {
            Ok(i) => i,
            Err(io_err) => return Err(Panic::ReadFileErr(io_err)),
        };

        for inst_chunck in buf.chunks(INST_CHUNCK_SIZE) {
            // TODO: maybe handle this unwrap
            let inst = Instruction::deserialize(inst_chunck.try_into().unwrap())?;
            self.program.push(inst);
        }

        Ok(())
    }

    pub fn serialize_into_file<P: AsRef<Path>>(&self, file: P) -> Result<(), Panic> {
        let mut buf = Vec::<SerializedInst>::new();

        for inst in self.program.items.iter() {
            buf.push(inst.serialize()?);
        }

        if let Err(io_err) = fs::write(file.as_ref(), buf.concat()) {
            return Err(Panic::WriteToFileErr(io_err));
        }

        Ok(())
    }

    pub fn disassemble_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Panic> {
        let program = match fs::read_to_string(path.as_ref()) {
            Ok(p) => p,
            Err(io_err) => return Err(Panic::ReadFileErr(io_err)),
        };

        self.program = Instruction::disassemble(program)?;

        Ok(())
    }

    fn execute(&mut self) -> Result<(), Panic> {
        let (dbg_stack, dbg_inst) = self.debug;
        let mut inst_count = 0;
        let limit = match self.inst_limit {
            Some(l) => l,
            _ => std::usize::MAX,
        };

        while self.inst_ptr < self.program.size && inst_count != limit {
            if dbg_inst {
                println!(
                    "+ ІНСТ {ptr} : {inst}",
                    ptr = self.inst_ptr,
                    inst = self.program.get(self.inst_ptr),
                );
            }

            self.execute_instruction()?;

            if dbg_stack {
                println!(
                    "СТЕК [{size}] АДР: {ptr} ЗНАЧ: {v}",
                    size = self.stack.size,
                    ptr = if self.stack.size < 1 {
                        0
                    } else {
                        self.stack.size - 1
                    },
                    v = if self.stack.size < 1 {
                        self.stack.get(self.stack.size)
                    } else {
                        self.stack.get(self.stack.size - 1)
                    }
                );
            }

            inst_count += 1;
        }

        Ok(())
    }

    fn execute_instruction(&mut self) -> Result<(), Panic> {
        fn push_from<F>(state: &mut VM, f: F) -> Result<(), Panic>
        where
            F: Fn(isize, isize) -> Result<isize, Panic>,
        {
            let (a, b) = match (state.stack_pop()?, state.stack_pop()?) {
                (Value::Int(a), Value::Int(b)) => (a, b),
                (val_a, val_b) => {
                    return Err(Panic::IlligalInstructionOperands {
                        inst: state.program.items[state.inst_ptr].kind,
                        val_a,
                        val_b,
                    })
                }
            };
            state.stack_push(Value::Int(f(a, b)?))
        }

        let inst = self.program.get(self.inst_ptr);

        if inst.conditional {
            if let Value::Int(i) = self.stack.get_last() {
                if i == 0 {
                    self.inst_ptr += 1;
                    return Ok(());
                }
            } else {
                return Err(Panic::InvalidOperandValue {
                    operand: Value::Null.to_string(),
                    inst: inst.kind,
                });
            }
        }

        use InstructionKind::*;
        let result = match inst.kind {
            Nop => Ok(()),
            Push => self.stack_push(inst.operand),
            Drop => {
                let _ = self.stack.pop();
                Ok(())
            }
            DupAt => {
                let addr = inst
                    .operand
                    .into_option()
                    .ok_or(Panic::InvalidOperandValue {
                        operand: inst.operand.to_string(),
                        inst: inst.kind,
                    })?;
                if addr < 0 || addr as usize > self.inst_ptr {
                    return Err(Panic::InvalidOperandValue {
                        operand: inst.operand.to_string(),
                        inst: inst.kind,
                    });
                }

                self.stack_push(self.stack.get(addr as usize))
            }
            Dup => {
                let target = self.stack_pop()?;
                self.stack_push(target)?;
                self.stack_push(target)
            }
            Jump => {
                let addr = inst
                    .operand
                    .into_option()
                    .ok_or(Panic::InvalidOperandValue {
                        operand: inst.operand.to_string(),
                        inst: inst.kind,
                    })?;
                if addr < 0 || addr as usize > self.program.size {
                    return Err(Panic::InvalidOperandValue {
                        operand: inst.operand.to_string(),
                        inst: inst.kind,
                    });
                }
                self.inst_ptr = addr as usize;

                return Ok(());
            }
            Eq => {
                if self.stack.size < 2 {
                    return Err(Panic::StackUnderflow);
                }

                let a = self.stack.items[self.stack.size - 1];
                let b = self.stack.items[self.stack.size - 2];

                self.stack_push(if a == b { Value::Int(1) } else { Value::Int(0) })
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
        };

        self.inst_ptr += 1;
        result
    }

    fn stack_push(&mut self, value: Value) -> Result<(), Panic> {
        let Value::Int(value) = value else {
            return Err(Panic::InvalidOperandValue { operand: value.to_string(), inst: InstructionKind::Push });
        };
        if !(isize::MIN..=isize::MAX).contains(&value) {
            Err(Panic::IntegerOverflow)
        } else if self.stack.size == VM_STACK_CAPACITY {
            Err(Panic::StackOverflow)
        } else {
            self.stack.push(Value::Int(value));
            Ok(())
        }
    }

    fn stack_pop(&mut self) -> Result<Value, Panic> {
        if self.stack.size == 0 {
            Err(Panic::StackUnderflow)
        } else {
            let value = self.stack.pop();

            Ok(value)
        }
    }

    fn dump_program(&self) {
        let limit = match self.inst_limit {
            Some(l) if l <= self.program.size => l,
            _ => self.program.size,
        };

        for i in 0..limit {
            println!("{}", self.program.items[i]);
        }
    }
}

fn print_usage(msg: Option<&str>) {
    eprintln!(
        "./uvm [ОПЦ] <ФАЙЛ>
<ФАЙЛ> - файл з байткодом інструкцій УВМ
[ОПЦ]:
    -b - вказаний файл є байткодом
    -l <ЧИС> - встановити ліміт на кількість виконуваних інструкцій
    -ds - показати всі зміни стеку на протязі виконня програми
    -di - показати лист виконаних інструкцій
    -h - показати це повідомлення

{msg}",
        msg = msg.unwrap_or(""),
    )
}

fn main() {
    let mut args = std::env::args().skip(1);
    let mut state = VM::init();
    let mut file: Option<String> = None;
    let mut binary = false;
    let mut dump_prog = false;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-h" => return print_usage(None),
            "-b" => binary = true,
            "-dp" => dump_prog = true,
            "-ds" => state.debug.0 = true,
            "-di" => state.debug.1 = true,
            "-l" => match args.next() {
                Some(limit) => match limit.parse::<usize>() {
                    Ok(l) => state.inst_limit = Some(l),
                    _ => return eprintln!("[!] Встановлений неправельний ліміт"),
                },
                _ => return eprintln!("[!] Значення для ліміту не вказано"),
            },
            f => {
                if Path::new(&f).is_file() {
                    file = Some(f.to_string());
                } else {
                    return print_usage(Some(&format!("[!] Вказано неіснуючий файл: {f}")));
                }
            }
        }
    }

    if let Some(f) = file {
        if binary {
            if let Err(e) = state.deserialize_from_file(&f) {
                eprintln!("[!] {e}");
            }
        } else if let Err(e) = state.disassemble_from_file(&f) {
            eprintln!("[!] {e}");
        }

        if dump_prog {
            state.dump_program();
        } else if let Err(e) = state.execute() {
            eprintln!("[!] Помилка виконання інструкцій: {e}");
        }
    } else {
        eprintln!();
        print_usage(Some("[!] Файл не вказано"));
    }
}
