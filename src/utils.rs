use crate::{Instruction, Panic, Value, PROGRAM_INST_CAPACITY, VM};
use std::{error, fmt};

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Instruction::*;
        match self {
            Push(v) => write!(f, "Стак-Доповнення({v})"),
            Jump(i) => write!(f, "Повтор-Інструкції({i})"),
            Eof => write!(f, "Кінцева-Інструкція"),
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
            StackOverflow => write!(f, "Стак переповнений"),
            StackUnderflow => write!(f, "Спроба дістати неіснуюче значення зі стаку"),
            EofReached => write!(f, "Досягнено кінцевої інструкції"),
            InstLimitkOverflow(inst_size) => write!(
                f,
                "Перевищено ліміт інструкцій: {inst_size} із доступних {PROGRAM_INST_CAPACITY}"
            ),
            DivByZero => write!(f, "Спроба ділення на нуль"),
            IncompatibleValue { inst, val_a, val_b } => write!(
                f,
                "Неможливо виконати інструкцію {inst} для значень {val_a} та {val_b}"
            ),
        }
    }
}

impl error::Error for Panic {}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Value::*;
        match self {
            Number(n) => write!(f, "Число({n})"),
            Null => write!(f, "<Порожньо>"),
        }
    }
}

impl fmt::Display for VM {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "[СТАК({capa})] {value}  ",
            value = if self.stack_size != 0 {
                self.stack[self.stack_size - 1]
            } else {
                self.stack[self.stack_size]
            },
            capa =self.stack_size,
        )
    }
}
