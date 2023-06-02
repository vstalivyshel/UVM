use crate::{Array, Panic, PROGRAM_INST_CAPACITY};

pub const INST_CHUNCK_SIZE: usize = 10;
pub type SerializedInst = [u8; INST_CHUNCK_SIZE];

#[derive(Copy, Clone, Debug, PartialEq, Default)]
pub enum Value {
    Int(isize),
    #[default]
    Null,
}

impl Value {
    pub fn into_option(self) -> Option<isize> {
        match self {
            Value::Int(i) => Some(i),
            _ => None,
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
    DupAt = 7,
    Sub = 8,
    Mul = 9,
    Div = 10,
}

impl InstructionKind {
    fn try_from<T: AsRef<str>>(src: T) -> Result<Self, Panic> {
        use InstructionKind::*;
        Ok(match src.as_ref() {
            "неоп" => Nop,
            "кинь" => Drop,
            "копію" => Dup,
            "клади" => Push,
            "копію_у" => DupAt,
            "крок" => Jump,
            "рівн" => Eq,
            "різн" => Sub,
            "множ" => Mul,
            "діли" => Div,
            "сума" => Sum,
            inst => return Err(Panic::InvalidInstruction(inst.to_string())),
        })
    }

    fn as_string(&self) -> String {
        use InstructionKind::*;
        match self {
            Nop => "неоп",
            Drop => "кинь",
            Dup => "копію",
            Push => "клади",
            DupAt => "копію_у",
            Jump => "крок",
            Eq => "рівн",
            Sub => "різн",
            Mul => "множ",
            Div => "діли",
            Sum => "сума",
        }
        .into()
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
            DupAt => true,
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
    // 			   - with_operand: 1 - supposed to have an operand, 0 - not
    // 			   - conditional: 10 - conditional, 00 - not
    // 		3..=10 - bytes representation of the isize
    //

    pub fn serialize(&self) -> Result<SerializedInst, Panic> {
        let mut se = [0; INST_CHUNCK_SIZE];
        se[0] = self.kind as u8;

        if self.conditional {
            se[1] += 10;
        }

        if let Value::Int(i) = self.operand {
            for (i, b) in i.to_le_bytes().into_iter().enumerate() {
                se[i + 2] = b;
            }
        } else {
            // 'N' bytes to say that there is no operand supplied
            se[2] = b'N';
        };

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
            7 => DupAt,
            8 => Sub,
            9 => Mul,
            10 => Div,
            // I don't like this one
            _ => return Err(Panic::InvalidBinaryInstruction),
        };

        let mut inst_opts = se[1];
        let conditional = if inst_opts >= 10 {
            inst_opts %= 10;
            true
        } else {
            false
        };

        let with_operand = inst_opts != 0;
        let operand_chunck = &se[1..INST_CHUNCK_SIZE];
        let operand = if with_operand && operand_chunck.contains(&b'N') {
            return Err(Panic::InvalidOperandValue {
                operand: Value::Null.to_string(),
                inst: kind,
            });
        } else if !with_operand && operand_chunck.contains(&b'N') {
            Value::Null
        } else {
            Value::Int(isize::from_le_bytes(operand_chunck.try_into().unwrap()))
        };

        Ok(Instruction {
            kind,
            operand,
            conditional,
        })
    }
}

pub fn assemble(source: Array<Instruction, PROGRAM_INST_CAPACITY>) -> Result<String, Panic> {
    Ok(String::new())
}

pub fn disassemble(source: String) -> Result<Array<Instruction, PROGRAM_INST_CAPACITY>, Panic> {
    let mut token_strem = source
        .lines()
        .filter(|line| !line.trim_start().starts_with('#'))
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
                    if let Ok(i) = op.parse::<isize>() {
                        Value::Int(i)
                    } else if let Some((addr, _)) = lables_table
                        .items
                        .iter()
                        .find(|(_, label)| label.contains(op))
                    {
                        Value::Int(*addr as isize)
                    } else {
                        return Err(Panic::InvalidOperandValue {
                            operand: op.to_string(),
                            inst: kind,
                        });
                    }
                }

                _ => {
                    return Err(Panic::InvalidOperandValue {
                        operand: Value::Null.to_string(),
                        inst: kind,
                    })
                }
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
