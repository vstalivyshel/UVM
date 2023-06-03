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

pub fn print_usage() {
    eprintln!(
        "./uvm [ОПЦ] <ФАЙЛ>
<ФАЙЛ> - файл з байткодом інструкцій УВМ
[ОПЦ]:
    -usm - перевести <ФАЙЛ> формату USM (assembly) на байткод
    -dusm - перевести <ФАЙЛ> з байткодом на USM (assembly)
    -dump - показати інструкціЇ у вказаному файлі без виконяння
    -l <ЧИС> - встановити ліміт на кількість виконуваних інструкцій
    -ds - показати всі зміни стеку на протязі виконня програми
    -di - показати лист виконаних інструкцій
    -h - показати це повідомлення"
    )
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

    pub fn _get_last(&self) -> T {
        self.items[if self.size < 1 { 0 } else { self.size - 1 }]
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

#[derive(Debug)]
pub struct DisplayValue(pub Value);

impl fmt::Display for DisplayValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.0 {
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
            oper = DisplayValue(self.operand),
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
