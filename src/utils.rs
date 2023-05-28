use crate::{Instruction, InstructionKind, Panic, VM};
use std::{error, fmt};

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use InstructionKind::*;
        match self.kind {
            Push | DupAt | JumpIf | Jump => write!(f, "{}({})", self.kind, self.operand),
            _ => write!(f, "{}", self.kind),
        }
    }
}

impl fmt::Display for InstructionKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use InstructionKind::*;
        match self {
            Nop => write!(f, "<Немає_Інструкції>"),
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
            ReadFile => write!(f, "Неможливо_Прочитати_Файл"),
            WriteToFile => write!(f, "Помилка_Запусу_До_Файлу"),
            DivByZero => write!(f, "Ділення_На_Нуль"),
        }
    }
}

impl error::Error for Panic {}

impl fmt::Display for VM {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "СТЕК [{capa}]\n    АДР: {addr} ЗНАЧ: {value}  ",
            capa = self.stack_size,
            addr = if self.stack_size == 0 {
                0
            } else {
                self.stack_size - 1
            },
            value = if self.stack_size != 0 {
                self.stack[self.stack_size - 1]
            } else {
                self.stack[self.stack_size]
            },
        )
    }
}
