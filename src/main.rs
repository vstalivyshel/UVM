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
#[derive(Copy, Clone, Debug)]
enum InstructionKind {
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
    kind: InstructionKind,
    operand: Option<isize>,
}

type SerializedInst = [u8; INST_CHUNCK_SIZE];

impl Instruction {
    fn serialize(&self) -> Result<SerializedInst, Panic> {
        let mut se = [0; INST_CHUNCK_SIZE];
        se[0] = self.kind as u8;
        use InstructionKind::*;
        match self.kind {
            Push | DupAt | JumpIf | Jump => {
                // Tenth byte indicates that there is supposed to be an operand,
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
    inst_limit: Option<usize>,
    program_size: usize,
    inst_ptr: usize,
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
            inst_limit: None,
            program_size: 0,
            inst_ptr: 0,
            debug: (false, false),
        }
    }

    fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Panic> {
        let buf = match fs::read(path.as_ref()) {
            Ok(i) => i,
            Err(_) => return Err(Panic::ReadFileErr),
        };

        let mut program = Vec::<Instruction>::new();
        for inst in buf.chunks(INST_CHUNCK_SIZE) {
            // TODO: maybe handle this unwrap
            program.push(Instruction::deserialize(inst.try_into().unwrap())?);
        }

        self.load_from_memmory(program.as_slice())
    }

    fn load_from_memmory(&mut self, program: &[Instruction]) -> Result<(), Panic> {
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

    fn execute(&mut self) -> Result<(), Panic> {
        let (dbg_stack, dbg_inst) = self.debug;
        let mut inst_count = 0;
        let limit = match self.inst_limit {
            Some(l) => l,
            _ => self.program_size,
        };

        while self.inst_ptr < self.program_size && inst_count != limit {
            self.instruction_execute()?;
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

fn program_save_to_file<P: AsRef<Path>>(file: P, program: &[Instruction]) -> Result<(), Panic> {
    let mut buf = Vec::<SerializedInst>::new();

    for inst in program.iter() {
        buf.push(inst.serialize().unwrap());
    }

    if fs::write(file.as_ref(), buf.concat()).is_err() {
        return Err(Panic::WriteToFileErr);
    }

    Ok(())
}

fn print_usage(msg: Option<&str>) {
    eprintln!(
        "./uvm [ОПЦ] <ФАЙЛ>
<ФАЙЛ> - файл з байткодом інструкцій УВМ
[ОПЦ]:
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

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-h" => return print_usage(None),
            "-ds" => state.debug.0 = true,
            "-di" => state.debug.1 = true,
            "-l" => match args.next() {
                Some(limit) => match limit.parse::<usize>() {
                    Ok(l) => state.inst_limit = Some(l),
                    _ => return eprintln!("[!] Встановлений неправельний ліміт"),
                },
                _ => return eprintln!("[!] Значення для ліміт не вказано"),
            },
            f if Path::new(&f).is_file() => {
                file = Some(f.to_string());
            }
            a => return print_usage(Some(&format!("[!] Невірний аргумент: {a}"))),
        }
    }

    if let Some(f) = file {
        if let Err(e) = state.load_from_file(&f) {
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
