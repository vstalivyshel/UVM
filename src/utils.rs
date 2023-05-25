use crate::{Instruction, Panic, Value, VM, VM_STACK_CAPACITY};
use std::{error, fmt};

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Instruction::*;
        match self {
            Push(v) => write!(f, "Стак-Доповнення: {v}"),
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
            StackUnderflow => write!(f, "Спроба дістат неіснуюче значення зі стаку"),
            IlligalInstruction { inst, val_a, val_b } => write!(
                f,
                "Нелегальна інструкція: {inst} для значеннь {val_a} та {val_b}"
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
            Null => write!(f, "Нічого"),
        }
    }
}

impl fmt::Display for VM {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "СТАК: ВМІСТИМІСТЬ: {capa}; ПОТОЧНИЙ-РОЗМІР: {size};  ВЕРХНЄ ЗНАЧЕННЯ: \"{value}\"",
            capa = VM_STACK_CAPACITY - self.stack_size,
            size = self.stack_size,
            value = if self.stack_size != 0 {
                self.stack[self.stack_size - 1]
            } else {
                self.stack[self.stack_size]
            }
        )
    }
}
