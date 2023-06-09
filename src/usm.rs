use crate::{Array, Panic, PROGRAM_INST_CAPACITY};

pub const INST_CHUNCK_SIZE: usize = 10;
pub type SerializedInst = [u8; INST_CHUNCK_SIZE];
const COMMENT_TOKEN: &str = ";;";

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub enum Value {
    Float(f64),
    Int(isize),
    Uint(usize),
    #[default]
    Null,
}

impl Value {
    fn try_parse<T: AsRef<str>>(token: T) -> Result<Self, ()> {
        let token = token.as_ref().trim();
        Ok(if let Some((val, suf)) = token.rsplit_once('_') {
            match suf {
                "дроб" => Value::Float(val.parse::<f64>().map_err(|_| ())?),
                "зціл" => Value::Int(val.parse::<isize>().map_err(|_| ())?),
                "ціл" => Value::Uint(val.parse::<usize>().map_err(|_| ())?),
                _ => return Err(()),
            }
        } else if let Ok(val) = token.parse::<isize>() {
            Value::Int(val)
        } else if let Ok(f) = token.parse::<f64>().map_err(|_| ()) {
            Value::Float(f)
        } else {
            return Err(());
        })
    }

    pub fn into_float(self) -> f64 {
        use Value::*;
        match self {
            Float(v) => v,
            Int(v) => v as f64,
            Uint(v) => v as f64,
            Null => panic!(),
        }
    }

    pub fn into_int(self) -> isize {
        use Value::*;
        match self {
            Float(v) => v as isize,
            Int(v) => v,
            Uint(v) => v as isize,
            Null => panic!(),
        }
    }
    pub fn into_uint(self) -> usize {
        use Value::*;
        match self {
            Float(v) => v.abs() as usize,
            Int(v) => v.unsigned_abs(),
            Uint(v) => v,
            Null => panic!(),
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
            Float(_) => Float(self.into_float()),
            Int(_) => Int(self.into_int()),
            Uint(_) => Uint(self.into_uint()),
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
    Extern = 11,
    Return = 12,
    Call = 13,
    Halt = 14,
    Swap = 15,
}

impl InstructionKind {
    fn try_parse<T: AsRef<str>>(src: T) -> Result<Self, ()> {
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
            "ззовні" => Extern,
            "вертай" => Return,
            "клич" => Call,
            "кінчай" => Halt,
            "міняй" => Swap,
            _ => return Err(()),
        })
    }

    fn try_from_idx(idx: u8) -> Self {
        use InstructionKind::*;
        match idx {
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
            11 => Extern,
            12 => Return,
            13 => Call,
            14 => Halt,
            15 => Swap,
            _ => panic!(),
        }
    }

    fn has_operand(&self) -> bool {
        use InstructionKind::*;
        matches!(self, Push | Dup | Jump | Call | Swap | Extern)
    }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct Instruction {
    pub kind: InstructionKind,
    pub operand: Value,
    pub conditional: bool,
}

pub fn deserialize(se: SerializedInst) -> Instruction {
    let kind = InstructionKind::try_from_idx(se[0]);
    let inst_opts = se[1];
    let operand_chunck = &se[2..INST_CHUNCK_SIZE];
    let chunck = operand_chunck.try_into().unwrap();
    let (n, operand) = match inst_opts {
        200.. => (200, Value::Float(f64::from_le_bytes(chunck))),
        100.. => (100, Value::Uint(usize::from_le_bytes(chunck))),
        10.. => (10, Value::Int(isize::from_le_bytes(chunck))),
        _ => (10, Value::Null),
    };

    Instruction {
        kind,
        operand,
        conditional: inst_opts % n != 0,
    }
}

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

pub fn serialize(inst: Instruction) -> SerializedInst {
    let mut se = [0; INST_CHUNCK_SIZE];
    se[0] = inst.kind as u8;

    if inst.conditional {
        se[1] += 1;
    }

    use Value::*;
    match inst.operand {
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

    se
}

enum Token {
    Value(Value),
    Inst(Instruction),
    LabelExpand(String),
}

fn parse(source: String) -> (Vec<Token>, Vec<(String, usize)>) {
    let mut tokens = Vec::<Token>::new();
    let mut labels = Vec::<(String, usize)>::new();
    let mut inst_count = 0;

    for line in source
        .lines()
        .filter(|line| !line.trim_start().starts_with(COMMENT_TOKEN))
    {
        let line = line.split_once(COMMENT_TOKEN).map(|(l, _)| l).unwrap_or(line);
        for word in line.split_whitespace() {
            let word = word.trim();

            if let Some(label) = word.strip_suffix(':') {
                labels.push((label.into(), inst_count));
                continue;
            }

            tokens.push(if let Some(inst) = word.strip_suffix('?') {
                InstructionKind::try_parse(inst)
                    .map(|kind| {
                        inst_count += 1;
                        Token::Inst(Instruction {
                            kind,
                            operand: Value::Null,
                            conditional: true,
                        })
                    })
                    .unwrap_or(Token::LabelExpand(word.into()))
            } else if let Ok(val) = Value::try_parse(word) {
                Token::Value(val)
            } else if let Ok(kind) = InstructionKind::try_parse(word) {
                inst_count += 1;
                Token::Inst(Instruction {
                    kind,
                    operand: Value::Null,
                    conditional: false,
                })
            } else {
                Token::LabelExpand(word.into())
            })
        }
    }

    (tokens, labels)
}

pub fn disassemble(src: String) -> Result<Array<Instruction, PROGRAM_INST_CAPACITY>, Panic> {
    let mut program = Array::<Instruction, PROGRAM_INST_CAPACITY>::new();
    let (src, labels_table) = parse(src);

    for token in src {
        match token {
            Token::Inst(inst) => program.push(inst),
            Token::LabelExpand(name) => {
                let last = program.get_last_mut();
                if let InstructionKind::Nop = last.kind {
                    return Err(Panic::ParseError(format!("не передбачений операнд у вигляді лейблу \"{name}\" для відсутьої інструкції")));
                }
                if last.kind.has_operand() {
                    last.operand = Value::Uint(
                        labels_table
                            .iter()
                            .find(|l| l.0.contains(name.as_str()))
                            .ok_or(Panic::ParseError(format!(
                                "спроба використати неіснуючий лейбл \"{name}\" для інструкції \"{kind}\"",
                                kind = last.kind
                            )))?
                            .1,
                    );
                } else {
                    return Err(Panic::ParseError(format!(
                        "спроба використати лейбл \"{name}\" як не передбачений операнд для інструкції \"{kind}\"",
                        kind = last.kind
                    )));
                }
            }
            Token::Value(val) => {
                let last = program.get_last_mut();
                if let InstructionKind::Nop = last.kind {
                    return Err(Panic::ParseError(format!(
                        "не передбачений операнд \"{val}\" для відсутьої інструкції"
                    )));
                }
                if last.kind.has_operand() {
                    last.operand = val;
                } else {
                    return Err(Panic::ParseError(format!(
                        "не передбачений операнд \"{val}\" для інструкції \"{kind}\"",
                        kind = last.kind
                    )));
                }
            }
        }
    }

    if let Some(e) = program
        .get_all()
        .iter()
        .find(|i| i.kind.has_operand() && i.operand.is_null())
    {
        return Err(Panic::ParseError(format!(
            "відсутнє значення для інструкції \"{kind}\"",
            kind = e.kind
        )));
    }

    Ok(program)
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
