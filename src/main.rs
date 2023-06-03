mod instruction;
mod utils;
use crate::instruction::{Instruction, InstructionKind, SerializedInst, Value, INST_CHUNCK_SIZE};
use crate::utils::DisplayValue;
use std::{
    fs,
    io::{self, Write},
    path::Path,
};
use utils::Array;

const VM_STACK_CAPACITY: usize = 1024;
const PROGRAM_INST_CAPACITY: usize = 1024;

type VMResult<T> = Result<T, Panic>;

#[derive(Debug)]
pub enum Panic {
    StackOverflow,
    StackUnderflow,
    ValueOverflow,
    ValueUnderflow,
    InvalidOperandValue,
    IlligalInstructionOperands,
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
        self.program = instruction::disassemble(
            fs::read_to_string(path.as_ref()).map_err(Panic::ReadFileErr)?,
        )?;

        Ok(())
    }

    fn execute_instruction(&mut self) -> Result<(), Panic> {
        let inst = self.program.get(self.inst_ptr);

        if inst.conditional && self.stack_pop()?.into_uint().is_some_and(|v| v == 0) {
            self.inst_ptr += 1;
            return Ok(());
        }

        macro_rules! math {
            ($op:tt) => {{
                let a = self.stack_pop()?;
                let b = self.stack_pop()?.into_type_of(a);
                use Value::*;
                match (a, b) {
                    (Int(a), Int(b)) => self.stack_push(Int(a $op b)),
                    (Uint(a), Uint(b)) => self.stack_push(Uint(a $op b)),
                    (Float(a), Float(b)) => self.stack_push(Float(a $op b)),
                    _ => Ok(()),
                }
            }};
        }

        use InstructionKind::*;
        let result = match inst.kind {
            Nop => Ok(()),
            Push => self.stack_push(inst.operand),
            Drop => {
                let _ = self.stack.pop();
                Ok(())
            }
            Dup => {
                let target = self.stack_pop()?;
                self.stack_push(target)?;
                self.stack_push(target)
            }
            Jump => {
                let addr = inst.operand.into_uint().ok_or(Panic::InvalidOperandValue)?;
                if addr > self.program.size {
                    return Err(Panic::InvalidOperandValue);
                }
                self.inst_ptr = addr;

                return Ok(());
            }
            Eq => {
                if self.stack.size < 2 {
                    return Err(Panic::StackUnderflow);
                }
                let a = self.stack.items[self.stack.size - 1];
                let b = self.stack.items[self.stack.size - 2];
                self.stack_push(Value::Uint(a.is_eq_to(b) as usize))
            }
            Sum => math!(+),
            Sub => math!(-),
            Mul => math!(*),
            Div => math!(+),
        };

        self.inst_ptr += 1;
        result
    }

    fn stack_push(&mut self, value: Value) -> Result<(), Panic> {
        if let Value::Null = value {
            Err(Panic::InvalidOperandValue)
        } else if self.stack.size == VM_STACK_CAPACITY {
            Err(Panic::StackOverflow)
        } else {
            self.stack.push(value);
            Ok(())
        }
    }

    fn stack_pop(&mut self) -> Result<Value, Panic> {
        if self.stack.size == 0 {
            Err(Panic::StackUnderflow)
        } else {
            let value = self.stack.pop();

            if value.is_null() {
                return Err(Panic::StackUnderflow);
            }

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
            } => {
                state.load_from_file(target_file)?;
                let res = instruction::assemble(&state.program);
                if let Some(f) = output_file {
                    fs::write(f, res)
                } else {
                    io::stdout().write_all(res.as_bytes())
                }
                .map_err(Panic::WriteToFileErr)?;
            }
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
                                DisplayValue(state.stack.get(state.stack.size))
                            } else {
                                DisplayValue(state.stack.get(state.stack.size - 1))
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
