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
    fn try_parse<T: AsRef<str>>(token: T) -> Result<Self, ()> {
        let token = token.as_ref().trim();
        Ok(if token.contains('.') {
            Value::Float(token.parse::<f64>().map_err(|_| ())?)
        } else if let Some((val, suf)) = token.rsplit_once('_') {
            match suf {
                "дроб" => Value::Float(val.parse::<f64>().map_err(|_| ())?),
                "зціл" => Value::Int(val.parse::<isize>().map_err(|_| ())?),
                "ціл" => Value::Uint(val.parse::<usize>().map_err(|_| ())?),
                _ => return Err(()),
            }
        } else if let Ok(val) = token.parse::<isize>() {
            Value::Int(val)
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
            Float(v) => v as usize,
            Int(v) => v as usize,
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
            _ => panic!(),
        }
    }

    fn has_operand(&self) -> bool {
        use InstructionKind::*;
        matches!(self, Push | Dup | Jump)
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
        100.. => (100, Value::Int(isize::from_le_bytes(chunck))),
        10.. => (10, Value::Uint(usize::from_le_bytes(chunck))),
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

#[derive(PartialEq)]
enum Token {
    Value(Value),
    Inst(InstructionKind, bool),
    LabelExpand(String),
}

fn parse(source: String) -> (Vec<Token>, Vec<(String, usize)>) {
    let mut tokens = Vec::<Token>::new();
    let mut labels = Vec::<(String, usize)>::new();
    let mut inst_count = 0;
    let lines = source
        .lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .map(|line| line.split_once('#').map(|(l, _)| l).unwrap_or(line));

    for line in lines {
        for word in line.split_whitespace() {
            let word = word.trim();

            if let Some(label) = word.strip_suffix(':') {
                labels.push((label.into(), inst_count));
                continue;
            }

            tokens.push(if let Some(inst) = word.strip_suffix('?') {
                if let Ok(kind) = InstructionKind::try_parse(inst) {
                    Token::Inst(kind, true)
                } else {
                    Token::LabelExpand(word.into())
                }
            } else if let Ok(val) = Value::try_parse(word) {
                Token::Value(val)
            } else if let Ok(kind) = InstructionKind::try_parse(word) {
                inst_count += 1;
                Token::Inst(kind, false)
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
            Token::Inst(kind, conditional) => {
                program.push(Instruction {
                    kind,
                    conditional,
                    operand: crate::Value::Null,
                });
            }
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
                                "неіснуючий лейбл \"{name}\" для інструкції \"{kind}\"",
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
