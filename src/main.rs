#[cfg(test)]
mod test;
mod utils;
use std::{fs, isize, path::Path};

const VM_STACK_CAPACITY: usize = 1024;
const PROGRAM_INST_CAPACITY: usize = 1024;
const INST_CHUNCK_SIZE: usize = 10;

// TODO: Have more descriptive errors

#[derive(Copy, Clone, Debug)]
pub enum Panic {
    StackOverflow,
    StackUnderflow,
    IntegerOverflow,
    InvalidOperand,
    InstLimitkOverflow,
    InvalidInstruction,
    ReadFileErr,
    WriteToFileErr,
    DivByZero,
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum InstructionKind {
    Nop = 0,
    Drop = 1,
    Dup = 2,
    Push = 3,
    DupAt = 4,
    JumpIf = 5,
    Jump = 6,
    Eq = 7,
    Sub = 8,
    Mul = 9,
    Div = 10,
    Sum = 11,
}

#[derive(Copy, Clone, Debug)]
pub struct Instruction {
    pub kind: InstructionKind,
    pub operand: Option<isize>,
}

pub type SerializedInst = [u8; INST_CHUNCK_SIZE];

impl Instruction {
    pub fn serialize(&self) -> Result<SerializedInst, Panic> {
        let mut se = [0; INST_CHUNCK_SIZE];
        se[0] = self.kind as u8;
        use InstructionKind::*;
        match self.kind {
            Push | DupAt | JumpIf | Jump => {
                // Tenth byte tells if there is supposed to be an operand
                se[INST_CHUNCK_SIZE - 1] = 1;
                for (i, b) in self
                    .operand
                    .ok_or(Panic::InvalidOperand)?
                    .to_be_bytes()
                    .into_iter()
                    .enumerate()
                {
                    se[i + 1] = b;
                }
            }
            _ => {}
        }

        Ok(se)
    }

    fn deserialize(se: SerializedInst) -> Result<Instruction, Panic> {
        use InstructionKind::*;
        let kind = match se[0] {
            0 => Nop,
            1 => Drop,
            2 => Dup,
            3 => Push,
            4 => DupAt,
            5 => JumpIf,
            6 => Jump,
            7 => Eq,
            8 => Sub,
            9 => Mul,
            10 => Div,
            11 => Sum,
            _ => {
                return Err(Panic::InvalidInstruction);
            }
        };

        let operand = if se[INST_CHUNCK_SIZE - 1] != 0 {
            Some(isize::from_be_bytes(
                se[1..INST_CHUNCK_SIZE - 1].try_into().unwrap(),
            ))
        } else {
            None
        };

        Ok(Instruction { kind, operand })
    }
}

pub struct VM {
    stack: [isize; VM_STACK_CAPACITY],
    stack_size: usize,
    program: [Instruction; PROGRAM_INST_CAPACITY],
    program_size: usize,
    inst_ptr: usize,
    inst_limit: Option<usize>,
    debug: (bool, bool),
}

impl VM {
    fn init() -> Self {
        Self {
            stack: [0; VM_STACK_CAPACITY],
            stack_size: 0,
            program: [Instruction {
                kind: InstructionKind::Nop,
                operand: None,
            }; PROGRAM_INST_CAPACITY],
            program_size: 0,
            inst_ptr: 0,
            inst_limit: None,
            debug: (false, false),
        }
    }

    fn load_from_memmory(&mut self, program: &[Instruction]) -> Result<(), Panic> {
        let len = program.len();
        if len > PROGRAM_INST_CAPACITY {
            return Err(Panic::InstLimitkOverflow);
        }

        self.program_size = len;
        for i in 0..len {
            self.program[i] = program[i];
        }

        Ok(())
    }

    fn deserialize_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Panic> {
        let buf = match fs::read(path.as_ref()) {
            Ok(i) => i,
            Err(_) => return Err(Panic::ReadFileErr),
        };

        let mut size = 0;
        for (i, inst) in buf.chunks(INST_CHUNCK_SIZE).enumerate() {
            // TODO: maybe handle this unwrap
            self.program[i] = Instruction::deserialize(inst.try_into().unwrap())?;
            size += 1;
        }
        self.program_size = size;

        Ok(())
    }

    pub fn serialize_into_file<P: AsRef<Path>>(&self, file: P) -> Result<(), Panic> {
        let mut buf = Vec::<SerializedInst>::new();

        for inst in self.program.iter() {
            buf.push(inst.serialize()?);
        }

        if fs::write(file.as_ref(), buf.concat()).is_err() {
            return Err(Panic::WriteToFileErr);
        }

        Ok(())
    }

    pub fn disassemble_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Panic> {
        let program = if let Ok(p) = fs::read_to_string(path.as_ref()) {
            p
        } else {
            return Err(Panic::ReadFileErr);
        };

        let mut stream = program.split_whitespace();

        let mut idx = 0;
        while let Some(token) = stream.next() {
            use InstructionKind::*;
            let (kind, with_operand) = match token {
                "неоп" => (Nop, false),
                "кинь" => (Drop, false),
                "копію" => (Dup, false),
                "клади" => (Push, true),
                "копію_у" => (DupAt, true),
                "крок_рівн" => (JumpIf, true),
                "крок" => (Jump, true),
                "рівн" => (Eq, false),
                "різн" => (Sub, false),
                "множ" => (Mul, false),
                "діли" => (Div, false),
                "сума" => (Sum, false),
                _ => return Err(Panic::InvalidInstruction),
            };

            let operand = if with_operand {
                match stream.next() {
                    Some(i) => match i.parse::<isize>() {
                        Ok(i) => Some(i),
                        _ => return Err(Panic::InvalidOperand),
                    },
                    _ => return Err(Panic::InvalidOperand),
                }
            } else {
                None
            };

            self.program[idx] = Instruction { kind, operand };

            idx += 1;
        }

        self.program_size = idx;

        Ok(())
    }

    fn execute(&mut self) -> Result<(), Panic> {
        let (dbg_stack, dbg_inst) = self.debug;
        let mut inst_count = 0;
        let limit = match self.inst_limit {
            Some(l) => l,
            _ => self.program_size,
        };

        while self.inst_ptr < self.program_size && inst_count != limit {
            self.execute_instruction()?;
            if dbg_inst {
                println!(
                    "+ ІНСТ {ptr} : {inst}",
                    ptr = self.inst_ptr,
                    inst = self.program[self.inst_ptr],
                );
            }
            if dbg_stack {
                println!(
                    "СТЕК [{size}] АДР: {ptr} ЗНАЧ: {v}",
                    size = self.stack_size,
                    ptr = if self.stack_size < 1 {
                        0
                    } else {
                        self.stack_size - 1
                    },
                    v = self.stack[self.stack_size - 1]
                );
            }
            self.inst_ptr += 1;
            inst_count += 1;
        }

        Ok(())
    }

    fn execute_instruction(&mut self) -> Result<(), Panic> {
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
            Push => self.stack_push(inst.operand.ok_or(Panic::InvalidOperand)?),
            Drop => {
                self.stack_size -= 1;
                self.stack[self.stack_size] = 0;
                Ok(())
            }
            DupAt => {
                let addr = inst.operand.ok_or(Panic::InvalidOperand)?;
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
                    self.inst_ptr = inst.operand.ok_or(Panic::InvalidOperand)? as usize;
                }

                Ok(())
            }
            Jump => {
                let addr = inst.operand.ok_or(Panic::InvalidOperand)?;
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

                let a = self.stack[self.stack_size - 1];
                let b = self.stack[self.stack_size - 2];

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
        }
    }

    fn stack_push(&mut self, value: isize) -> Result<(), Panic> {
        if !(isize::MIN..=isize::MAX).contains(&value) {
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

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-h" => return print_usage(None),
            "-b" => binary = true,
            "-ds" => state.debug.0 = true,
            "-di" => state.debug.1 = true,
            "-l" => match args.next() {
                Some(limit) => match limit.parse::<usize>() {
                    Ok(l) => state.inst_limit = Some(l),
                    _ => return eprintln!("[!] Встановлений неправельний ліміт"),
                },
                _ => return eprintln!("[!] Значення для ліміту не вказано"),
            },
            f if Path::new(&f).is_file() => {
                file = Some(f.to_string());
            }
            a => return print_usage(Some(&format!("[!] Невірний аргумент: {a}"))),
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

        if let Err(e) = state.execute() {
            eprintln!("[!] Помилка виконання інструкцій: {e}");
        }
    } else {
        eprintln!();
        print_usage(Some("[!] Файл не вказано"));
    }
}
