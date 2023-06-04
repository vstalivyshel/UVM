use crate::{Instruction, InstructionKind, Panic, Value, PROGRAM_INST_CAPACITY};
use std::{error, fmt};

#[macro_export]
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
            operand: Value::Null,
        }
    };
}

#[macro_export]
macro_rules! prog {
    ($($inst:tt $($operand:expr)?),*$(,)?) => {
        [$(inst!($inst $($operand)?),)*]
    };
}

pub fn print_usage_ua(sub: &str) {
	let general =  "./uvm [ПІДКОМАНДА] [ОПЦ] <ФАЙЛ>

[ПІДКОМАНДА]
    emu - виконати інструкції UVM з <ФАЙЛУ>
    usm - перекласти <ФАЙЛ> з байткодом інструкцій UVM на USM (assembly)
    dusm - перекласти <ФАЙЛ> формату USM (assembly) на байткод з інструкціями UVM
    dump - прочитати <ФАЙЛ> без виконання інструкцій та показати лист цих інструкцій

[ОПЦ]
    -h - показати це повідомлення";

    let emu = "./uvm emu [ОПЦ] <ФАЙЛ>

[ОПЦ]
    -usm - перекласти <ФАЙЛ> формату USM (assembly) на байткод інструкцій UVM та виконати їх
    -l <ЧИС> - встановити ліміт на кількість виконуваних інструкцій
    -ds - показати всі зміни стеку на протязі виконня програми
    -di - показати лист виконаних інструкцій
    -h - показати це повідомлення";

    let dusm = "./uvm dusm [ОПЦ] <ФАЙЛ>

[ОПЦ]
    -o <ВИХІДНИЙ ФАЙЛ> - записати байткод інструкцій до <ВИХІДНОГО ФАЙЛУ>
    -h - показати це повідомлення";

    let usm = "./uvm usm [ОПЦ] <ФАЙЛ>

[ОПЦ]
    -o <ВИХІДНИЙ ФАЙЛ> - записати перекладені на USM (assembly) інструкціЇ до <ВИХІДНОГО ФАЙЛУ>
    -h - показати це повідомлення";

    let dump = "./uvm usm [ОПЦ] <ФАЙЛ>

[ОПЦ]
    -l <ЧИС> - встановити ліміт на кількість показаних інструкцій
    -h - показати це повідомлення";

    eprintln!("{}", match sub {
        "emu" => emu,
        "dusm" => dusm,
        "usm" => usm,
        "dump" => dump,
        _ => general,
    });
}

pub fn print_usage_en(sub: &str) {
	let general =  "./uvm [SUBCMD] [OPT] <FILE>

[SUBCMD]
    emu - run the instructions from the <FILE>
    usm - translate the bytecode of instructions from the <FILE> into the USM (assembly)
    dusm - translate the USM (assembly) from the file <FILE> into bytecode
    dump - read the instructions from the <FILE> without execution and dump them into stdout

[OPT]
    -h - show this message";

    let emu = "./uvm emu [OPT] <FILE>

[OPT]
    -usm - перекласти <FILE> формату USM (assembly) на байткод інструкцій UVM та виконати їх
    -usm - translate the USM instructions from the file <FILE> and execute them
    -l <NUM> - set a limit of executed instructions
    -ds - dump all changes to the stack while executing the instructions
    -di - dump list of each executed instruction
    -h - show this message";

    let dusm = "./uvm dusm [OPT] <FILE>

[OPT]
    -o <OUTPUT FILE> - write translated into bytecode instructions into the <OUTPUT FILE>
    -h - show this message";

    let usm = "./uvm usm [OPT] <FILE>

[OPT]
    -o <OUTPUT FILE> - write translated into USM instructions into the <OUTPUT FILE>
    -h - show this message";

    let dump = "./uvm usm [OPT] <FILE>

[OPT]
    -l <NUM> - set a limit of dumped instructions
    -h - show this message";

    eprintln!("{}", match sub {
        "emu" => emu,
        "dusm" => dusm,
        "usm" => usm,
        "dump" => dump,
        _ => general,
    });
}

impl<T: Copy + Default, const N: usize> Default for Array<T, N> {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct Array<T, const N: usize> {
    pub items: [T; N],
    pub size: usize,
}

impl<T: Copy + Default, const N: usize> Array<T, N> {
    pub fn new() -> Self {
        Self {
            items: [T::default(); N],
            size: 0,
        }
    }

    pub fn get_from_end(&self, idx: usize) -> T {
        self.items[self.size - (idx + 1)]
    }

    pub fn get_last(&self) -> T {
        self.get_from_end(0)
    }

    pub fn push(&mut self, item: T) {
        self.items[self.size] = item;
        self.size += 1;
    }

    pub fn get(&self, idx: usize) -> T {
        self.items[idx]
    }

    pub fn pop(&mut self) -> T {
        self.size -= 1;
        self.items[self.size]
    }

    pub fn _replace(&mut self, idx: usize, item: T) {
        self.items[idx] = item;
    }

    pub fn get_all(&self) -> &[T] {
        &self.items[..self.size]
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Float(v) => write!(f, "{v}"),
            Value::Uint(v) => write!(f, "{v}"),
            Value::Int(v) => write!(f, "{v}"),
            Value::Null => write!(f, ""),
        }
    }
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{kind}{cond} {oper}",
            kind = self.kind,
            cond = if self.conditional { "?" } else { "" },
            oper = self.operand,
        )
    }
}

impl fmt::Display for InstructionKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use InstructionKind::*;
        match self {
            Nop => write!(f, "неоп"),
            Drop => write!(f, "кинь"),
            Dup => write!(f, "копію"),
            Push => write!(f, "клади"),
            Jump => write!(f, "крок"),
            Eq => write!(f, "рівн"),
            Sub => write!(f, "різн"),
            Mul => write!(f, "множ"),
            Div => write!(f, "діли"),
            Sum => write!(f, "сума"),
            NotEq => write!(f, "нерівн"),
        }
    }
}

impl fmt::Display for Panic {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Panic::*;
        match self {
            StackOverflow => write!(f, "Переповнений Стек"),
            StackUnderflow => write!(f, "Незаповненість Стека"),
            ValueOverflow => write!(f, "Перевищено Ліміт Цілого Числа"),
            ValueUnderflow => write!(f, ""),
            InvalidOperandValue => {
                write!(f, "Невірний Операнд")
            }
            IlligalInstructionOperands => write!(f, "Неможливо Виконати Інструкцію"),
            InstLimitkOverflow(size) => write!(
                f,
                "Перевищено Ліміт Інструкцій: {size} з доступних {PROGRAM_INST_CAPACITY}"
            ),
            InvalidBinaryInstruction => write!(f, "Нелегальна Інструкція"),
            InvalidInstruction(inst) => write!(f, "Нелегальна Інструкція: {inst}"),
            ReadFileErr(err) => write!(f, "Неможливо Прочитати Файл: {err}"),
            WriteToFileErr(err) => write!(f, "Помилка Запусу До Файлу: {err}"),
            DivByZero => write!(f, "Ділення На Нуль"),
        }
    }
}

impl error::Error for Panic {}
