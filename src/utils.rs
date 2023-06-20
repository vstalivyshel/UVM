use crate::{Instruction, InstructionKind, Panic, Value};
use std::{error, fmt};

pub fn print_usage(sub: &str) {
    let general = "./uvm [ПІДКОМАНДА] [ОПЦ] <ФАЙЛ>

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

    eprintln!(
        "{}",
        match sub {
            "emu" => emu,
            "dusm" => dusm,
            "usm" => usm,
            "dump" => dump,
            _ => general,
        }
    );
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

    fn is_empty(&self) -> bool {
        self.size == 0
    }

    pub fn get_from_end(&self, idx: usize) -> T {
        self.items[if self.is_empty() && idx == 0 {
            self.size
        } else {
            self.size - (idx + 1)
        }]
    }

    pub fn get_from_end_mut(&mut self, idx: usize) -> &mut T {
        &mut self.items[if self.is_empty() && idx == 0 {
            self.size
        } else {
            self.size - (idx + 1)
        }]
    }

    pub fn get_last(&self) -> T {
        self.get_from_end(0)
    }

    pub fn get_last_mut(&mut self) -> &mut T {
        self.get_from_end_mut(0)
    }

    pub fn get(&self, idx: usize) -> T {
        self.items[idx]
    }

    pub fn _get_mut(&mut self, idx: usize) -> &mut T {
        &mut self.items[idx]
    }

    pub fn push(&mut self, item: T) {
        self.items[self.size] = item;
        self.size += 1;
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
            Value::Float(v) => write!(f, "{v}_дроб"),
            Value::Uint(v) => write!(f, "{v}_ціл"),
            Value::Int(v) => write!(f, "{v}_зціл"),
            Value::Null => write!(f, "_"),
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
            ParseError(e) => write!(f, "Помилка Перекладу: {e}"),
            ReadFileErr(err) => write!(f, "Неможливо Прочитати Файл: {err}"),
            WriteToFileErr(err) => write!(f, "Помилка Запусу До Файлу: {err}"),
            DivByZero => write!(f, "Ділення На Нуль"),
        }
    }
}

impl error::Error for Panic {}
