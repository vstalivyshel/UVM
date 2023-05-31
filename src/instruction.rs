use crate::{array::Array, Panic, PROGRAM_INST_CAPACITY};

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

pub const INST_CHUNCK_SIZE: usize = 10;
pub type SerializedInst = [u8; INST_CHUNCK_SIZE];

#[derive(Copy, Clone, Debug, Default)]
pub struct Instruction {
    pub kind: InstructionKind,
    pub operand: Value,
    pub conditional: bool,
}

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
            _ => {
                return Err(Panic::InvalidBinaryInstruction);
            }
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

    pub fn disassemble(
        token_strem: String,
    ) -> Result<Array<Instruction, PROGRAM_INST_CAPACITY>, Panic> {
        let mut stream = token_strem.split_whitespace();
        let mut program = Array::<Instruction, PROGRAM_INST_CAPACITY>::new();
        let mut lables_table = Array::<(usize, &str), PROGRAM_INST_CAPACITY>::new();
        let mut inst_addr = 0;

        while let Some(token) = stream.next() {
            if token.ends_with(':') {
                lables_table.push((inst_addr, token.strip_suffix(':').unwrap()));
                continue;
            }
            let conditional = token.ends_with('?');
            let token = token.strip_suffix('?').unwrap_or(token);
            use InstructionKind::*;
            let (kind, with_operand) = match token {
                "неоп" => (Nop, false),
                "кинь" => (Drop, false),
                "копію" => (Dup, false),
                "клади" => (Push, true),
                "копію_у" => (DupAt, true),
                "крок" => (Jump, true),
                "рівн" => (Eq, false),
                "різн" => (Sub, false),
                "множ" => (Mul, false),
                "діли" => (Div, false),
                "сума" => (Sum, false),
                inst => return Err(Panic::InvalidInstruction(inst.to_string())),
            };

            let operand = if with_operand {
                match stream.next() {
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

            let inst = Instruction {
                kind,
                operand,
                conditional,
            };

            program.push(inst);
            inst_addr += 1;
        }

        Ok(program)
    }

    pub fn nop() -> Self {
        Self {
            kind: InstructionKind::Nop,
            operand: Value::Null,
            conditional: false,
        }
    }
}
