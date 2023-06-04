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
struct VM {
    stack: Array<Value, VM_STACK_CAPACITY>,
    program: Array<Instruction, PROGRAM_INST_CAPACITY>,
    inst_ptr: usize,
}

impl VM {
    fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> VMResult<()> {
        for inst_chunck in fs::read(path.as_ref())
            .map_err(Panic::ReadFileErr)?
            .chunks(INST_CHUNCK_SIZE)
        {
            self.program
                .push(Instruction::deserialize(inst_chunck.try_into().unwrap())?);
        }

        Ok(())
    }

    fn save_into_file(&self, file: Option<String>) -> VMResult<()> {
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

    fn disassemble_from_file<P: AsRef<Path>>(&mut self, path: P) -> VMResult<()> {
        self.program =
            usm::disassemble(fs::read_to_string(path.as_ref()).map_err(Panic::ReadFileErr)?)?;

        Ok(())
    }

    fn execute_instruction(&mut self) -> VMResult<()> {
        let inst = self.program.get(self.inst_ptr);

        if inst.conditional && self.stack_pop()?.into_uint()? == 0 {
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
                    // We are not allowed to push or pop Null values
                    _ => unreachable!(),
                }
            }};
        }

        use InstructionKind::*;
        let result = match inst.kind {
            Nop => Ok(()),
            Push => self.stack_push(inst.operand),
            Drop => {
                let _ = self.stack_pop()?;
                Ok(())
            }
            Dup => self.stack_push(self.stack_take(inst.operand.into_uint()?)?),
            Jump => {
                let addr = inst.operand.into_uint()?;
                if addr > self.inst_ptr {
                    return Err(Panic::InvalidOperandValue);
                }
                self.inst_ptr = addr;

                return Ok(());
            }
            NotEq | Eq => {
                let a = self.stack_take(0)?;
                let b = self.stack_take(1)?;
                self.stack_push(Value::Uint(
                    ((inst.kind == Eq) & (a == b)) as usize | (a != b) as usize,
                ))
            }
            Sum => math!(+),
            Sub => math!(-),
            Mul => math!(*),
            Div => math!(+),
        };

        self.inst_ptr += 1;
        result
    }

    fn stack_take(&self, idx: usize) -> VMResult<Value> {
        if self.stack.size == 0 {
            return Err(Panic::StackUnderflow);
        } else if idx > self.stack.size {
            return Err(Panic::InvalidOperandValue);
        }

        Ok(self.stack.get_from_end(idx))
    }

    fn stack_push(&mut self, value: Value) -> VMResult<()> {
        if let Value::Null = value {
            Err(Panic::InvalidOperandValue)
        } else if self.stack.size == VM_STACK_CAPACITY {
            Err(Panic::StackOverflow)
        } else {
            self.stack.push(value);
            Ok(())
        }
    }

    fn stack_pop(&mut self) -> VMResult<Value> {
        if self.stack.size == 0 {
            return Err(Panic::StackUnderflow);
        }

        let value = self.stack.pop();
        if value.is_null() {
            return Err(Panic::StackUnderflow);
        }

        Ok(value)
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
                for i in 0..inst_limit
                    .map(|l| if l <= state.program.size { l } else { 0 })
                    .unwrap_or(0)
                {
                    println!("{}", state.program.get(i));
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
                let res = usm::assemble(state.program.get_all());
                match output_file {
                    Some(f) => fs::write(f, res),
                    _ => io::stdout().write_all(res.as_bytes()),
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
                let limit = inst_limit.unwrap_or(0);
                while state.inst_ptr < state.program.size && limit != 0 && inst_count == limit {
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
        _ => return utils::print_usage(""),
    };

    let sub = sub.as_str();
    let config = match sub {
        "dump" => {
            let mut target_file = String::new();
            let mut inst_limit: Option<usize> = None;
            while let Some(arg) = args.next() {
                match arg.as_str() {
                    "-h" => return utils::print_usage(sub),
                    "-l" => match args.next() {
                        Some(limit) => match limit.parse::<usize>() {
                            Ok(l) => inst_limit = Some(l),
                            _ => return eprintln!("ПОМИЛКА: Встановлений неправельний ліміт"),
                        },

                        _ => return eprintln!("ПОМИЛКА: Значення для ліміту не вказано"),
                    },
                    f if Path::new(&f).is_file() => target_file = f.to_string(),
                    wrong_op if wrong_op.starts_with('-') => {
                        return eprintln!("ПОМИЛКА: Вказана помилкова опція: {wrong_op}")
                    }
                    wrong_file => {
                        return eprintln!("ПОМИЛКА: Вказано неіснуючий файл: {wrong_file}")
                    }
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
                    "-h" => return utils::print_usage(sub),
                    "-o" => output_file = args.next(),
                    f if Path::new(&f).is_file() => target_file = f.into(),
                    wrong_op if wrong_op.starts_with('-') => {
                        return eprintln!("ПОМИЛКА: Вказана помилкова опція: {wrong_op}")
                    }
                    wrong_file => {
                        return eprintln!("ПОМИЛКА: Вказано неіснуючий файл: {wrong_file}")
                    }
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
                    "-h" => return utils::print_usage(sub),
                    "-ds" => debug_stack = true,
                    "-di" => debug_inst = true,
                    "-l" => match args.next() {
                        Some(limit) => match limit.parse::<usize>() {
                            Ok(l) => inst_limit = Some(l),
                            _ => {
                                return eprintln!(
                                    "ПОМИЛКА: Встановлений неправельний ліміт: {limit}"
                                )
                            }
                        },
                        _ => return eprintln!("ПОМИЛКА: Значення для ліміту не вказано"),
                    },
                    f if Path::new(&f).is_file() => target_file = f.into(),
                    wrong_op if wrong_op.starts_with('-') => {
                        return eprintln!("ПОМИЛКА: Вказана помилкова опція: {wrong_op}")
                    }
                    wrong_file => {
                        return eprintln!("ПОМИЛКА: Вказано неіснуючий файл: {wrong_file}")
                    }
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
        "-h" => return utils::print_usage(""),
        wrong_sub if wrong_sub.starts_with('-') => {
            return eprintln!("ПОМИЛКА: Вказана помилкова підкоманда: {wrong_sub}")
        }
        wrong_file => return eprintln!("ПОМИЛКА: Вказано неіснуючий файл: {wrong_file}"),
    };

    if let Err(e) = VM::start(config) {
        eprintln!("ПОМИЛКА: {e}");
    }
}
