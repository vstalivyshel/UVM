use crate::{Array, Panic, PROGRAM_INST_CAPACITY};

pub const INST_CHUNCK_SIZE: usize = 10;
pub type SerializedInst = [u8; INST_CHUNCK_SIZE];

// TODO: stack: float, float and dupe will produce int for some reason: fix it

// All values on stack stored as f64 to save as much
// information as possible without unneeded overhead.
#[derive(Copy, Clone, Debug, Default)]
pub enum Value {
    Float(f64),
    Int(isize),
    Uint(usize),
    #[default]
    Null,
}

impl Value {
    pub fn into_float(self) -> Option<f64> {
        use Value::*;
        match self {
            Float(v) => Some(v),
            Int(v) => Some(v as f64),
            Uint(v) => Some(v as f64),
            Null => None,
        }
    }

    pub fn into_int(self) -> Option<isize> {
        use Value::*;
        match self {
            Float(v) => Some(v as isize),
            Int(v) => Some(v),
            Uint(v) => Some(v as isize),
            Null => None,
        }
    }
    pub fn into_uint(self) -> Option<usize> {
        use Value::*;
        match self {
            Float(v) => Some(v as usize),
            Int(v) => Some(v as usize),
            Uint(v) => Some(v),
            Null => None,
        }
    }

    pub fn is_null(&self) -> bool {
        if let Value::Null = self {
            return true;
        }

        false
    }

    pub fn into_type_of(self, other: Value) -> Self {
        use Value::*;
        match other {
            Float(_) => Float(self.into_float().unwrap_or_default()),
            Int(_) => Int(self.into_int().unwrap_or_default()),
            Uint(_) => Uint(self.into_uint().unwrap_or_default()),
            Null => Null,
        }
    }

    pub fn is_eq_to(&self, other: Value) -> bool {
        use Value::*;
        match self {
            Float(v) => other.into_float().is_some_and(|other_v| *v == other_v),
            Int(v) => other.into_int().is_some_and(|other_v| *v == other_v),
            Uint(v) => other.into_uint().is_some_and(|other_v| *v == other_v),
            Null => other.is_null(),
        }
    }
}

// Type of the value(s) will be stored in 'type' register, which
// used to tell how do we wont to represent this value(s)
// for next instructions.
// 'type' can be changed using 'типу' (type) instruction
// with the following arguments:
// 		'зціле' - int
// 		'ціле' - uint
// 		'дроб' - float

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Default)]
pub enum InstructionKind {
    #[default]
    Nop = 0,
    Push = 1,
    Dup = 2,
    Drop = 3,
    Eq = 4,
    Jump = 5,
    Sum = 6,
    Sub = 7,
    Mul = 8,
    Div = 9,
}

impl InstructionKind {
    fn try_from<T: AsRef<str>>(src: T) -> Result<Self, Panic> {
        use InstructionKind::*;
        Ok(match src.as_ref() {
            "неоп" => Nop,
            "кинь" => Drop,
            "копію" => Dup,
            "клади" => Push,
            "крок" => Jump,
            "рівн" => Eq,
            "різн" => Sub,
            "множ" => Mul,
            "діли" => Div,
            "сума" => Sum,
            inst => return Err(Panic::InvalidInstruction(inst.to_string())),
        })
    }

    fn has_operand(&self) -> bool {
        use InstructionKind::*;
        match self {
            Nop => false,
            Push => true,
            Dup => false,
            Drop => false,
            Eq => false,
            Jump => true,
            Sum => false,
            Sub => false,
            Mul => false,
            Div => false,
        }
    }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct Instruction {
    pub kind: InstructionKind,
    pub operand: Value,
    pub conditional: bool,
}

impl Instruction {
    // Serialized instruction contains 10 bytes:
    // 		1 - enum's variant of InstructionKind
    // 		2 - gives information about instruction:
    // 			   - with_operand:
    // 			   		1 - supposed to have an operand,
    // 			   		0 - not
    // 			   - conditional:
    // 			   		10 - conditional,
    // 			   		00 - not
    // 			   - type:
    // 			   		f64 - 200,
    // 			   		u64 - 100,
    // 			   		i64 - 000
    //
    // 		3..=10 - bytes representation of the value or
    // 				'N' char if no operand was supplied

    pub fn serialize(&self) -> Result<SerializedInst, Panic> {
        let mut se = [0; INST_CHUNCK_SIZE];
        se[0] = self.kind as u8;

        if self.kind.has_operand() {
            se[1] += 1;
        }

        if self.conditional {
            se[1] += 10;
        }

        use Value::*;
        match self.operand {
            Float(i) => {
                se[1] += 200;
                se[2..].copy_from_slice(i.to_le_bytes().as_slice());
            }
            Uint(i) => {
                se[1] += 100;
                se[2..].copy_from_slice(i.to_le_bytes().as_slice());
            }
            Int(i) => {
                se[2..].copy_from_slice(i.to_le_bytes().as_slice());
            }
            Null => se[2] = b'N',
        }

        Ok(se)
    }

    pub fn deserialize(se: SerializedInst) -> Result<Instruction, Panic> {
        use InstructionKind::*;
        let kind = match se[0] {
            0 => Nop,
            1 => Push,
            2 => Dup,
            3 => Drop,
            4 => Eq,
            5 => Jump,
            6 => Sum,
            7 => Sub,
            8 => Mul,
            9 => Div,
            // I don't like this one
            _ => return Err(Panic::InvalidBinaryInstruction),
        };

		#[derive(Debug)]
        enum TypeValue {
            Float,
            Uint,
            Int,
        }

        let mut inst_opts = se[1];
        let type_value = if inst_opts >= 200 {
            inst_opts %= 200;
            TypeValue::Float
        } else if inst_opts >= 100 {
            inst_opts %= 100;
            TypeValue::Uint
        } else {
            TypeValue::Int
        };

        let conditional = if inst_opts >= 10 {
            inst_opts %= 10;
            true
        } else {
            false
        };

        let with_operand = inst_opts != 0;
        let operand_chunck = &se[2..INST_CHUNCK_SIZE];
        let operand = if with_operand && operand_chunck.contains(&b'N') {
            return Err(Panic::InvalidOperandValue);
        } else if !with_operand && operand_chunck.contains(&b'N') {
            Value::Null
        } else {
            let chunck = operand_chunck.try_into().unwrap();
            use TypeValue::*;
            match type_value {
                Float => Value::Float(f64::from_le_bytes(chunck)),
                Int => Value::Int(isize::from_le_bytes(chunck)),
                Uint => Value::Uint(usize::from_le_bytes(chunck)),
            }
        };

        Ok(Instruction {
            kind,
            operand,
            conditional,
        })
    }
}

pub fn assemble(source: &[Instruction]) -> String {
    source
        .iter()
        .map(|inst| {
            let mut inst = inst.to_string();
            inst.push('\n');
            inst
        })
        .collect::<String>()
}

pub fn disassemble(source: String) -> Result<Array<Instruction, PROGRAM_INST_CAPACITY>, Panic> {
    let mut token_strem = source
        .lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .map(|line| match line.split_once('#') {
            Some((l, _)) => l,
            _ => line,
        })
        .flat_map(|line| line.split_whitespace());
    let mut program = Array::<Instruction, PROGRAM_INST_CAPACITY>::new();
    let mut lables_table = Array::<(usize, &str), PROGRAM_INST_CAPACITY>::new();
    let mut inst_addr = 0;

    while let Some(token) = token_strem.next() {
        let token = token.trim();
        if token.ends_with(':') {
            lables_table.push((inst_addr, token.strip_suffix(':').unwrap()));
            continue;
        }
        let conditional = token.ends_with('?');
        let token = token.strip_suffix('?').unwrap_or(token);
        let kind = InstructionKind::try_from(token)?;
        let with_operand = kind.has_operand();
        let operand = if with_operand {
            match token_strem.next() {
                Some(op) => {
                    if let Ok(v) = op.parse::<f64>() {
                        Value::Float(v)
                    } else if let Ok(v) = op.parse::<usize>() {
                        Value::Uint(v)
                    } else if let Ok(v) = op.parse::<isize>() {
                        Value::Int(v)
                    } else if let Some((addr, _)) = lables_table
                        .items
                        .iter()
                        .find(|(_, label)| label.contains(op))
                    {
                        Value::Uint(*addr)
                    } else {
                        return Err(Panic::InvalidOperandValue);
                    }
                }

                _ => return Err(Panic::InvalidOperandValue),
            }
        } else {
            Value::Null
        };

        program.push(Instruction {
            kind,
            operand,
            conditional,
        });
        inst_addr += 1;
    }

    Ok(program)
}
