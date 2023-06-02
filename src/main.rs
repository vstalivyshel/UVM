mod instruction;
mod utils;
use crate::instruction::{Instruction, InstructionKind, SerializedInst, Value, INST_CHUNCK_SIZE};
use std::{
    fs,
    io::{self, Write},
    isize,
    path::Path,
};
use utils::Array;

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

type VMResult<T> = Result<T, Panic>;

pub struct VM {
    stack: Array<Value, VM_STACK_CAPACITY>,
    program: Array<Instruction, PROGRAM_INST_CAPACITY>,
    inst_ptr: usize,
}

impl VM {
    fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Panic> {
        let buf = match fs::read(path.as_ref()) {
            Ok(i) => i,
            Err(io_err) => return Err(Panic::ReadFileErr(io_err)),
        };

        for inst_chunck in buf.chunks(INST_CHUNCK_SIZE) {
            let inst = Instruction::deserialize(inst_chunck.try_into().unwrap())?;
            self.program.push(inst);
        }

        Ok(())
    }

    pub fn save_into_file(&self, file: Option<String>) -> Result<(), Panic> {
        let mut buf = Array::<SerializedInst, PROGRAM_INST_CAPACITY>::new();

        for inst in self.program.items.iter() {
            buf.push(inst.serialize()?);
        }

        let ser_prog = buf.items.concat();

        match file {
            Some(f) => fs::write(f, ser_prog.as_slice()),
            _ => io::stdout().lock().write_all(ser_prog.as_slice()),
        }
        .map_err(Panic::WriteToFileErr)
    }

    pub fn disassemble_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Panic> {
        let program = match fs::read_to_string(path.as_ref()) {
            Ok(p) => p,
            Err(io_err) => return Err(Panic::ReadFileErr(io_err)),
        };

        self.program = instruction::disassemble(program)?;

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
            match self.stack_pop()? {
                Value::Int(i) if i > 0 => {}
                _ => {
                    self.inst_ptr += 1;
                    return Ok(());
                }
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

    fn start(config: Configuration) -> VMResult<()> {
        let mut state = Self {
            stack: Array::new(),
            program: Array::new(),
            inst_ptr: 0,
        };

        use Configuration::*;
        match config {
            Dump {
                target_file,
                inst_limit,
            } => {
                state.load_from_file(target_file)?;
                let limit = match inst_limit {
                    Some(l) if l <= state.program.size => l,
                    _ => state.program.size,
                };

                for i in 0..limit {
                    println!("{}", state.program.items[i]);
                }
            }
            Disassemble {
                target_file,
                output_file,
            } => {
                state.disassemble_from_file(target_file)?;
                state.save_into_file(output_file)?;
            }
            Assemble {
                target_file,
                output_file,
            } => {}
            Run {
                target_file,
                inst_limit,
                debug_inst,
                debug_stack,
            } => {
                state.load_from_file(target_file)?;
                let mut inst_count = 0;
                let limit = match inst_limit {
                    Some(l) => l,
                    _ => PROGRAM_INST_CAPACITY,
                };

                while state.inst_ptr < state.program.size && inst_count != limit {
                    if debug_inst {
                        println!(
                            "+ ІНСТ {ptr} : {inst}",
                            ptr = state.inst_ptr,
                            inst = state.program.get(state.inst_ptr),
                        );
                    }

                    state.execute_instruction()?;
                    inst_count += 1;

                    if debug_stack {
                        println!(
                            "СТЕК [{size}] : {v}",
                            size = state.stack.size,
                            v = if state.stack.size < 1 {
                                state.stack.get(state.stack.size)
                            } else {
                                state.stack.get(state.stack.size - 1)
                            }
                        );
                    }
                }
            }
        }

        Ok(())
    }
}

enum Configuration {
    Dump {
        target_file: String,
        inst_limit: Option<usize>,
    },
    Run {
        target_file: String,
        inst_limit: Option<usize>,
        debug_inst: bool,
        debug_stack: bool,
    },
    Assemble {
        target_file: String,
        output_file: Option<String>,
    },
    Disassemble {
        target_file: String,
        output_file: Option<String>,
    },
}

fn main() {
    let mut args = std::env::args().skip(1);
    let sub = match args.next() {
        Some(s) => s,
        _ => return,
    };

    if args.any(|a| a.contains("-h")) {
        return utils::print_usage();
    }

    let config = match sub.as_str() {
        "dump" => {
            let mut target_file = String::new();
            let mut inst_limit: Option<usize> = None;
            while let Some(arg) = args.next() {
                match arg.as_str() {
                    "-l" => match args.next() {
                        Some(limit) => match limit.parse::<usize>() {
                            Ok(l) => inst_limit = Some(l),
                            _ => return eprintln!("[!] Встановлений неправельний ліміт"),
                        },

                        _ => return eprintln!("[!] Значення для ліміту не вказано"),
                    },
                    f if Path::new(&f).is_file() => target_file = f.into(),
                    arg => return eprintln!("[!] Невідома опція для підкоманди \"{sub}\": {arg}"),
                }
            }

            Configuration::Dump {
                target_file,
                inst_limit,
            }
        }
        opt @ "usm" | opt @ "dusm" => {
            let mut target_file = String::new();
            let mut output_file: Option<String> = None;
            while let Some(arg) = args.next() {
                match arg.as_str() {
                    "-o" => output_file = args.next(),
                    f if Path::new(&f).is_file() => target_file = f.into(),
                    arg => return eprintln!("[!] Невідома опція для підкоманди \"{sub}\": {arg}"),
                }
            }

            if opt == "usm" {
                Configuration::Assemble {
                    target_file,
                    output_file,
                }
            } else {
                Configuration::Disassemble {
                    target_file,
                    output_file,
                }
            }
        }

        _ => {
            let mut target_file = String::new();
            let mut inst_limit: Option<usize> = None;
            let mut debug_inst = false;
            let mut debug_stack = false;

            while let Some(a) = args.next() {
                match a.as_str() {
                    "-ds" => debug_stack = true,
                    "-di" => debug_inst = true,
                    "-l" => match args.next() {
                        Some(limit) => match limit.parse::<usize>() {
                            Ok(l) => inst_limit = Some(l),
                            _ => return eprintln!("[!] Встановлений неправельний ліміт"),
                        },
                        _ => return eprintln!("[!] Значення для ліміту не вказано"),
                    },
                    f if Path::new(&f).is_file() => target_file = f.into(),
                    arg => return eprintln!("[!] Невідома опція для підкоманди \"{sub}\": {arg}"),
                }
            }

            Configuration::Run {
                target_file,
                inst_limit,
                debug_inst,
                debug_stack,
            }
        }
    };

    if let Err(e) = VM::start(config) {
        eprintln!("[!] {e}");
    }
}
