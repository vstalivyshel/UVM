use crate::{Instruction, InstructionKind, Panic };
use std::{error, fmt};

#[macro_export]
macro_rules! inst {
    ($kind:tt $operand:expr) => {
        Instruction {
            kind: $kind,
            operand: Some($operand),
        }
    };

    ($kind:expr) => {
        Instruction {
            kind: $kind,
            operand: None,
        }
    };
}

#[macro_export]
macro_rules! prog {
    ($($inst:tt $($operand:expr)?),*$(,)?) => {
        [$(inst!($inst $($operand)?),)*]
    };
}


impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use InstructionKind::*;
        match self.kind {
            Push | DupAt | JumpIf | Jump => write!(f, "{}({})", self.kind, self.operand.unwrap()),
            _ => write!(f, "{}", self.kind),
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
            JumpIf => write!(f, "Умова_Крок_До_Адр"),
            Eq => write!(f, "Рівність"),
            Sum => write!(f, "Сумма"),
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
            StackOverflow => write!(f, "Переповнений_Стек"),
            StackUnderflow => write!(f, "Незаповненість_Стека"),
            IntegerOverflow => write!(f, "Перевищено_Ліміт_Цілого_Числа"),
            InvalidOperand => write!(f, "Невірний_Операнд"),
            InstLimitkOverflow => write!(f, "Перевищено_Ліміт_Інструкцій"),
            InvalidInstruction => write!(f, "Нелегальна_Інструкція"),
            ReadFileErr => write!(f, "Неможливо_Прочитати_Файл"),
            WriteToFileErr => write!(f, "Помилка_Запусу_До_Файлу"),
            DivByZero => write!(f, "Ділення_На_Нуль"),
        }
    }
}

impl error::Error for Panic {}
