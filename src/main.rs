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
    ReadFileErr(io::Error),
    WriteToFileErr(io::Error),
    ParseError(String),
    StackOverflow,
    StackUnderflow,
    ValueOverflow,
    DivByZero,
}

#[derive(Debug, Default)]
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
                .push(usm::deserialize(inst_chunck.try_into().unwrap()));
        }

        Ok(())
    }

    fn disassemble_from_file<P: AsRef<Path>>(&mut self, path: P) -> VMResult<()> {
        self.program =
            usm::disassemble(fs::read_to_string(path.as_ref()).map_err(Panic::ReadFileErr)?)?;

        Ok(())
    }

    fn save_into_file<P: AsRef<Path>>(&self, file: Option<P>) -> VMResult<()> {
        let mut buf = Array::<SerializedInst, PROGRAM_INST_CAPACITY>::new();
        for inst in self.program.get_all().iter() {
            buf.push(usm::serialize(*inst));
        }
        let ser_prog = buf.get_all().concat();
        match file {
            Some(f) => fs::write(f, ser_prog.as_slice()),
            _ => io::stdout().lock().write_all(ser_prog.as_slice()),
        }
        .map_err(Panic::WriteToFileErr)
    }

    fn assemble_into_file<P: AsRef<Path>>(&self, file: Option<P>) -> VMResult<()> {
        let src = usm::assemble(self.program.get_all());
        match file {
            Some(f) => fs::write(f, src.as_bytes()),
            _ => io::stdout().lock().write_all(src.as_bytes()),
        }
        .map_err(Panic::WriteToFileErr)
    }

    fn execute_instruction(&mut self) -> VMResult<()> {
        let inst = self.program.get(self.inst_ptr);

        if inst.conditional && self.stack_pop()?.into_uint() == 0 {
            self.inst_ptr += 1;
            return Ok(());
        }

        macro_rules! math {
            ($op:tt, $func_op:tt) => {{
                let a = self.stack_pop()?;
                let b = self.stack_pop()?.into_type_of(a);
                use Value::*;
                self.stack_push(match (a, b) {
                    (Int(a), Int(b)) => Value::Int(b.$func_op(a).ok_or(Panic::ValueOverflow)?),
                    (Uint(a), Uint(b)) => Value::Uint(b.$func_op(a).ok_or(Panic::ValueOverflow)?),
                    (Float(a), Float(b)) => {
                        let r = b $op a;
                        if !r.is_normal() {
                            return Err(Panic::ValueOverflow);
                        }
                        Value::Float(r)
                    }
                    // We are not allowed to push or pop Null values
                    _ => unreachable!(),
                })?
            }};
        }

        use InstructionKind::*;
        match inst.kind {
            Nop => {}
            Push => self.stack_push(inst.operand)?,
            Drop => _ = self.stack_pop()?,
            Dup => self.stack_push(self.stack_get(inst.operand.into_uint())?)?,
            Call | Jump => {
                if matches!(inst.kind, Call) {
                    self.stack_push(Value::Uint(self.inst_ptr + 1))?;
                }
                let addr = inst.operand.into_uint();
                return (addr < self.program.size)
                    .then(|| {
                        self.inst_ptr = addr;
                        Ok(())
                    })
                    .unwrap_or(Err(Panic::StackUnderflow));
            }
            NotEq | Eq => {
                let a = self.stack_get(0)?;
                let b = self.stack_get(1)?;
                self.stack_push(Value::Uint(
                    ((inst.kind == Eq) & (a == b)) as usize | (a != b) as usize,
                ))?;
            }
            Sum => math!(+ , checked_add),
            Sub => math!(- , checked_sub),
            Mul => math!(* , checked_mul),
            Div => math!(/ , checked_div),

            // TBD
            Extern => match inst.operand.into_uint() {
                0 => println!("{}", self.stack_get(0)?),
                _ => panic!(),
            },
            Return => {
                self.inst_ptr = self.stack_pop()?.into_uint();
                return Ok(());
            }
            Halt => {
                self.inst_ptr = self.program.size;
                return Ok(());
            }
            Swap => {
                if self.stack.size < 2 {
                    return Err(Panic::StackUnderflow);
                }
                let idx = inst.operand.into_uint();
                let saved_top = self.stack_get(0)?;
                let saved_target = self.stack_get(idx)?;
                let top = self.stack_get_mut(0)?;
                *top = saved_target;
                let target = self.stack_get_mut(idx)?;
                *target = saved_top;
            }
        }

        self.inst_ptr += 1;

        Ok(())
    }

    fn stack_get_mut(&mut self, idx: usize) -> VMResult<&mut Value> {
        (idx <= self.stack.size)
            .then_some(self.stack.get_from_end_mut(idx))
            .ok_or(Panic::StackUnderflow)
    }

    fn stack_get(&self, idx: usize) -> VMResult<Value> {
        (idx <= self.stack.size)
            .then_some(self.stack.get_from_end(idx))
            .ok_or(Panic::StackUnderflow)
    }

    fn stack_push(&mut self, value: Value) -> VMResult<()> {
        if let Value::Null = value {
            Err(Panic::StackUnderflow)
        } else if self.stack.size == VM_STACK_CAPACITY {
            Err(Panic::StackOverflow)
        } else {
            self.stack.push(value);
            Ok(())
        }
    }

    fn stack_pop(&mut self) -> VMResult<Value> {
        (self.stack.size > 0)
            .then_some(self.stack.pop())
            .filter(|v| !v.is_null())
            .ok_or(Panic::StackUnderflow)
    }
}

fn start(config: Configuration) -> VMResult<()> {
    let mut state = VM::default();

    use Configuration::*;
    match config {
        Dump {
            target_file,
            inst_limit,
            from_usm,
        } => {
            if from_usm || target_file.ends_with(".usm") {
                state.disassemble_from_file(target_file)?
            } else {
                state.load_from_file(target_file)?;
            }

            for i in 0..inst_limit
                .map(|l| if l <= state.program.size { l } else { 0 })
                .unwrap_or(state.program.size)
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
            state.assemble_into_file(output_file)?;
        }
        Run {
            target_file,
            from_usm,
            inst_limit,
            debug_inst,
            debug_stack,
        } => {
            if from_usm || target_file.ends_with(".usm") {
                state.disassemble_from_file(target_file)?;
            } else {
                state.load_from_file(target_file)?;
            };

            let mut inst_count = 0;
            let limit = inst_limit.unwrap_or(0);
            while state.inst_ptr < state.program.size {
                if limit != 0 && inst_count == limit {
                    break;
                }
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

#[derive(Debug)]
enum Configuration {
    Dump {
        target_file: String,
        inst_limit: Option<usize>,
        from_usm: bool,
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

    if args.len() < 1 {
        return utils::print_usage(sub);
    }

    let sub = sub.as_str();
    let config = match sub {
        "dump" => {
            let mut target_file = String::new();
            let mut inst_limit: Option<usize> = None;
            let mut from_usm = false;
            while let Some(arg) = args.next() {
                match arg.as_str() {
                    "-usm" => from_usm = true,
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
                from_usm,
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
        wrong_sub if !wrong_sub.starts_with('-') => {
            return eprintln!("ПОМИЛКА: Вказана помилкова підкоманда: {wrong_sub}")
        }
        wrong_file => return eprintln!("ПОМИЛКА: Вказано неіснуючий файл: {wrong_file}"),
    };

    if let Err(e) = start(config) {
        eprintln!("{e}");
    }
}
