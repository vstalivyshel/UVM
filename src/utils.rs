use crate::{Instruction, InstructionKind, Panic, Value, PROGRAM_INST_CAPACITY};
use std::{error, fmt};

impl<T: Copy + Default, const N: usize> Default for Array<T, N> {
    fn default() -> Self {
        Self::new()
    }
}

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

    pub fn get_last(&self) -> T {
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

    pub fn replace(&mut self, idx: usize, item: T) {
        self.items[idx] = item;
    }
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{cond_mark}{kind}({oper})",
            kind = self.kind,
            cond_mark = if self.conditional {
                "Умовно "
            } else {
                ""
            },
            oper = match self.operand {
                Value::Int(v) => format!("{v}"),
                _ => String::new(),
            }
        )
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Value::*;
        match self {
            Int(i) => write!(f, "Число({i})"),
            Null => write!(f, "<Відсутнє Значення>"),
        }
    }
}

impl fmt::Display for InstructionKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use InstructionKind::*;
        match self {
            Nop => write!(f, "_"),
            Push => write!(f, "Стек_Доповнення"),
            Drop => write!(f, "Звільнення_Верхнього_Значення"),
            Dup => write!(f, "Копія_Верхнього_Значення"),
            DupAt => write!(f, "Копія_За_Адр"),
            Jump => write!(f, "Крок_До_Адр"),
            Eq => write!(f, "Рівність"),
            Sum => write!(f, "Сума"),
            Sub => write!(f, "Віднімання"),
            Mul => write!(f, "Множення"),
            Div => write!(f, "Ділення"),
        }
    }
}

impl fmt::Display for Panic {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Panic::*;
        match self {
            StackOverflow => write!(f, "Переповнений Стек"),
            StackUnderflow => write!(f, "Незаповненість Стека"),
            IntegerOverflow => write!(f, "Перевищено Ліміт Цілого Числа"),
            InvalidOperandValue { operand, inst } => {
                write!(
                    f,
                    "Невірний Операнд \"{operand}\" для інструкціЇ \"{inst}\""
                )
            }
            IlligalInstructionOperands { inst, val_a, val_b } => write!(
                f,
                "Неможливо виконати \"{inst}\" для значень \"{val_b}\" та \"{val_a}\""
            ),
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
