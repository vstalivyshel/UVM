mod usm;
mod utils;
use crate::usm::{Instruction, InstructionKind, SerializedInst, Value, INST_CHUNCK_SIZE};
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

#[derive(Debug)]
pub struct VM {
    stack: Array<Value, VM_STACK_CAPACITY>,
    program: Array<Instruction, PROGRAM_INST_CAPACITY>,
    inst_ptr: usize,
}

impl VM {
    fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Panic> {
        for inst_chunck in fs::read(path.as_ref())
            .map_err(Panic::ReadFileErr)?
            .chunks(INST_CHUNCK_SIZE)
        {
            let inst = Instruction::deserialize(inst_chunck.try_into().unwrap())?;
            self.program.push(inst);
        }

        Ok(())
    }

    pub fn save_into_file(&self, file: Option<String>) -> Result<(), Panic> {
        let mut buf = Array::<SerializedInst, PROGRAM_INST_CAPACITY>::new();

        for inst in self.program.get_all().iter() {
            buf.push(inst.serialize()?);
        }

        let ser_prog = buf.get_all().concat();
        match file {
            Some(f) => fs::write(f, ser_prog.as_slice()),
            _ => io::stdout().lock().write_all(ser_prog.as_slice()),
        }
        .map_err(Panic::WriteToFileErr)
    }

    pub fn disassemble_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Panic> {
        self.program =
            usm::disassemble(fs::read_to_string(path.as_ref()).map_err(Panic::ReadFileErr)?)?;

        Ok(())
    }

    fn execute_instruction(&mut self) -> Result<(), Panic> {
        let inst = self.program.get(self.inst_ptr);

        if inst.conditional && self.stack_pop()?.into_uint().is_some_and(|v| v == 0) {
            self.inst_ptr += 1;
            return Ok(());
        }

        macro_rules! pop_math_push {
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
                if addr > self.inst_ptr {
                    return Err(Panic::InvalidOperandValue);
                }
                self.inst_ptr = addr;

                return Ok(());
            }
            NotEq | Eq => {
                if self.stack.size < 2 {
                    return Err(Panic::StackUnderflow);
                }
                let a = self.stack.get_last();
                let b = self.stack.get_from_end(1);
                self.stack_push(Value::Uint(if inst.kind == Eq {
                    a.is_eq_to(b) as usize
                } else {
                    !a.is_eq_to(b) as usize
                }))
            }
            Sum => pop_math_push!(+),
            Sub => pop_math_push!(-),
            Mul => pop_math_push!(*),
            Div => pop_math_push!(+),
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
                let res = usm::assemble(&state.program.get_all());
                if let Some(f) = output_file {
                    fs::write(f, res)
                } else {
                    io::stdout().write_all(res.as_bytes())
                }
                .map_err(Panic::WriteToFileErr)?;
            }
            Run {
                target_file,
                from_usm,
                inst_limit,
                debug_inst,
                debug_stack,
            } => {
                if from_usm {
                    state.disassemble_from_file(target_file)?;
                } else {
                    state.load_from_file(target_file)?;
                };

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
                            v = state.stack.get_last()
                        );
                    }
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug)]
enum Configuration {
    Dump {
        target_file: String,
        inst_limit: Option<usize>,
    },
    Run {
        target_file: String,
        from_usm: bool,
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
        _ => return utils::print_usage_ua(""),
    };

	let sub = sub.as_str();
    let config = match sub {
        "dump" => {
            let mut target_file = String::new();
            let mut inst_limit: Option<usize> = None;
            while let Some(arg) = args.next() {
                match arg.as_str() {
                    "-h" => return utils::print_usage_ua(sub),
                    "-l" => match args.next() {
                        Some(limit) => match limit.parse::<usize>() {
                            Ok(l) => inst_limit = Some(l),
                            _ => return eprintln!("ПОМИЛКА: Встановлений неправельний ліміт"),
                        },

                        _ => return eprintln!("ПОМИЛКА: Значення для ліміту не вказано"),
                    },
                    f if Path::new(&f).is_file() => target_file = f.to_string(),
                    arg => return eprintln!("ПОМИЛКА: Невідома опція для підкоманди \"{sub}\": {arg}"),
                }
            }

            Configuration::Dump {
                target_file,
                inst_limit,
            }
        }
        "usm" | "dusm" => {
            let mut target_file = String::new();
            let mut output_file: Option<String> = None;
            while let Some(arg) = args.next() {
                match arg.as_str() {
                    "-h" => return utils::print_usage_ua(sub),
                    "-o" => output_file = args.next(),
                    f if Path::new(&f).is_file() => target_file = f.into(),
                    arg => return eprintln!("ПОМИЛКА: Невідома опція для підкоманди \"{sub}\": {arg}"),
                }
            }

            if sub == "usm" {
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

        "emu" => {
            let mut target_file = String::new();
            let mut inst_limit: Option<usize> = None;
            let mut debug_inst = false;
            let mut debug_stack = false;
            let mut from_usm = false;

            while let Some(a) = args.next() {
                match a.as_str() {
                    "-usm" => from_usm = true,
                    "-h" => return utils::print_usage_ua(sub),
                    "-ds" => debug_stack = true,
                    "-di" => debug_inst = true,
                    "-l" => match args.next() {
                        Some(limit) => match limit.parse::<usize>() {
                            Ok(l) => inst_limit = Some(l),
                            _ => return eprintln!("ПОМИЛКА: Встановлений неправельний ліміт"),
                        },
                        _ => return eprintln!("ПОМИЛКА: Значення для ліміту не вказано"),
                    },
                    f if Path::new(&f).is_file() => target_file = f.into(),
                    arg => return eprintln!("ПОМИЛКА: Невідома опція для підкоманди \"{sub}\": {arg}"),
                }
            }

            Configuration::Run {
                target_file,
                from_usm,
                inst_limit,
                debug_inst,
                debug_stack,
            }
        }
        "-h" => return utils::print_usage_ua(""),
        no_sub => return eprintln!("ПОМИЛКА: Вказана невірна підкоманда: {no_sub}"),
    };

    if let Err(e) = VM::start(config) {
        eprintln!("ПОМИЛКА: {e}");
    }
}
