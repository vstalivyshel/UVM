use crate::{Array, Panic, PROGRAM_INST_CAPACITY};

pub const INST_CHUNCK_SIZE: usize = 10;
pub type SerializedInst = [u8; INST_CHUNCK_SIZE];

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub enum Value {
    Float(f64),
    Int(isize),
    Uint(usize),
    #[default]
    Null,
}

impl Value {
    pub fn into_float(self) -> Result<f64, Panic> {
        use Value::*;
        match self {
            Float(v) => Ok(v),
            Int(v) => Ok(v as f64),
            Uint(v) => Ok(v as f64),
            Null => Err(Panic::InvalidOperandValue),
        }
    }

    pub fn into_int(self) -> Result<isize, Panic> {
        use Value::*;
        match self {
            Float(v) => Ok(v as isize),
            Int(v) => Ok(v),
            Uint(v) => Ok(v as isize),
            Null => Err(Panic::InvalidOperandValue),
        }
    }
    pub fn into_uint(self) -> Result<usize, Panic> {
        use Value::*;
        match self {
            Float(v) => Ok(v as usize),
            Int(v) => Ok(v as usize),
            Uint(v) => Ok(v),
            Null => Err(Panic::InvalidOperandValue),
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
}

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
    NotEq = 10,
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
            "нерівн" => NotEq,
            inst => return Err(Panic::InvalidInstruction(inst.to_string())),
        })
    }
    fn try_from_idx(idx: u8) -> Result<Self, Panic> {
        use InstructionKind::*;
        let res = match idx {
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
            10 => NotEq,
            _ => return Err(Panic::InvalidBinaryInstruction),
        };

        Ok(res)
    }
    fn has_operand(&self) -> bool {
        use InstructionKind::*;
        match self {
            Nop => false,
            Push => true,
            Dup => true,
            Drop => false,
            Eq => false,
            Jump => true,
            Sum => false,
            Sub => false,
            Mul => false,
            Div => false,
            NotEq => false,
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
    // 		1 - kind of instruction
    // 		2 - information about instruction and it's operand
    // 			1/0 -conditional/not
    // 			i < 10 - operand is Value::Null
    // 			i >= 10 - operand is i64
    // 			i >= 100 - operand is u64
    // 			i >= 200 - operand is f64
    //
    // 		3..=10 - bytes representation of the value

    pub fn serialize(&self) -> Result<SerializedInst, Panic> {
        let mut se = [0; INST_CHUNCK_SIZE];
        se[0] = self.kind as u8;

        if self.conditional {
            se[1] += 1;
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
                se[1] += 10;
                se[2..].copy_from_slice(i.to_le_bytes().as_slice());
            }
            Null => {}
        }

        Ok(se)
    }

    pub fn deserialize(se: SerializedInst) -> Result<Instruction, Panic> {
        #[derive(Debug)]
        enum TypeOfValue {
            Float,
            Uint,
            Int,
            Null,
        }

        use TypeOfValue::*;

        let kind = InstructionKind::try_from_idx(se[0])?;
        let inst_opts = se[1];
        let (n, type_value) = match inst_opts {
            200.. => (200, Float),
            100.. => (100, Uint),
            10.. => (10, Int),
            _ => (0, Null),
        };

        let conditional = inst_opts % n != 0;
        let operand_chunck = &se[2..INST_CHUNCK_SIZE];
        let chunck = operand_chunck.try_into().unwrap();
        let operand = match type_value {
            Null if kind.has_operand() => return Err(Panic::InvalidOperandValue),
            Float => Value::Float(f64::from_le_bytes(chunck)),
            Int => Value::Int(isize::from_le_bytes(chunck)),
            Uint => Value::Uint(usize::from_le_bytes(chunck)),
            Null => Value::Null,
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
    macro_rules! try_parse {
        ($val:ident as $t:ty) => {
            $val.parse::<$t>().map_err(|_| Panic::InvalidOperandValue)?
        };
    }

    let mut program = Array::<Instruction, PROGRAM_INST_CAPACITY>::new();
    let mut lables_table = Array::<(usize, &str), PROGRAM_INST_CAPACITY>::new();
    let mut inst_addr = 0;
    let mut token_strem = source
        .lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .map(|line| line.split_once('#').map(|(l, _)| l).unwrap_or(line))
        .flat_map(|line| line.split_whitespace());

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
        let mut operand = Value::Null;
        if with_operand {
            let op = token_strem.next().ok_or(Panic::InvalidOperandValue)?;
            operand = match op.split_once('_') {
                Some((val, suf)) => match suf.trim() {
                    "дроб" => Value::Float(try_parse!(val as f64)),
                    "ціл" => Value::Uint(try_parse!(val as usize)),
                    "зціл" => Value::Int(try_parse!(val as isize)),
                    _ => Value::Null,
                },
                _ => match lables_table
                    .items
                    .iter()
                    .find(|(_, label)| label.contains(op))
                {
                    Some((addr, _)) => Value::Uint(*addr),
                    _ => Value::Int(try_parse!(op as isize)),
                },
            }
        }

        program.push(Instruction {
            kind,
            operand,
            conditional,
        });
        inst_addr += 1;
    }

    Ok(program)
}
