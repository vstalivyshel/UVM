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

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use InstructionKind::*;
        match self.kind {
            Push | DupAt | JumpIf | Jump => write!(f, "{}({})", self.kind, self.operand),
            _ => write!(f, "{}", self.kind),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Value::*;
        match self {
            Int(i) => write!(f, "Число({i})"),
            Null => write!(f, "<Значення Відсутнє>"),
        }
    }
}

impl fmt::Display for InstructionKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use InstructionKind::*;
        match self {
            Nop => write!(f, "_"),
            Push => write!(f, "Стек Доповнення"),
            Drop => write!(f, "Звільнення Верхнього Значення"),
            Dup => write!(f, "Копія Верхнього Значення"),
            DupAt => write!(f, "Копія За Адр"),
            Jump => write!(f, "Крок До Адр"),
            JumpIf => write!(f, "Умова Крок До Адр"),
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
            StackOverflow => write!(f, "Переповнений Стек"),
            StackUnderflow => write!(f, "Незаповненість Стека"),
            IntegerOverflow => write!(f, "Перевищено Ліміт Цілого Числа"),
            InvalidOperandValue { operand, inst } => {
                write!(f, "Невірний Операнд {operand} для інструкціЇ {inst}")
            }
            IlligalInstructionOperands { inst, val_a, val_b } => write!(
                f,
                "Неможливо виконати {inst} для значень {val_b} та {val_a}"
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
