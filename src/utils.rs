use crate::{Instruction, Panic, VM };
use std::{error, fmt};

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Instruction::*;
        match self {
            Nop => write!(f, "<Немає_Інструкції>"),
            Push(v) => write!(f, "Стек_Доповнення({v})"),
            Drop => write!(f, "Звільнення_Верхнього_Значення"),
            Dup => write!(f, "Копія_Верхнього_Значення"),
            DupAt(addr) => write!(f, "Копія_За_Адр({addr})"),
            Jump(addr) => write!(f, "Крок_До_Адр({addr})"),
            JumpIf(addr) => write!(f, "Умова_Крок_До_Адр({addr})"),
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
            InvalidOperand => write!(f, "Невірний_Операнд"),
            InstLimitkOverflow => write!(f, "Перевищено_Ліміт_Інструкцій"),
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
            addr =  if self.stack_size == 0 {
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
