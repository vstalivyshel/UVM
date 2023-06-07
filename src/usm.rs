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
        match self {
            Push | Dup | Jump => true,
            _ => false,
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
    pub fn deserialize(se: SerializedInst) -> Self {
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

        Self {
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

    pub fn serialize(&self) -> SerializedInst {
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

        se
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

fn tokenizer(src: String) {
    let lines = src
        .lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .map(|line| line.split_once('#').map(|(l, _)| l).unwrap_or(line));

	#[derive(Default)]
    struct InvalidInst {
        err_msg: String,
        body: String,
        operand: String,
    }

    enum TokenKind {
        Label,
        Inst { cond: bool },
        Value { suf: Option<String> },
    }

    struct Token {
        kind: TokenKind,
        body: String,
    }

    let mut tokens = Vec::<Token>::new();
    let mut current_instruction = Instruction::default();
    let mut current_invalid_inst = InvalidInst::default();
    let mut current_value = Value::default();
    let mut current = String::new();
    for (line, line_count) in lines.zip(1..) {
        for (token, token_count) in line.split_whitespace().zip(1..) {
            match InstructionKind::try_parse(token) {
                Ok(kind) => current_instruction.kind = kind,
                _ => current_invalid_inst.body =
            }
        }
    }
}

pub fn disassemble(source: String) -> Result<Array<Instruction, PROGRAM_INST_CAPACITY>, Panic> {
    fn fmt_err<T: std::fmt::Display + AsRef<str>>(
        msg: T,
        line: usize,
        token_count: usize,
        token: T,
        next_token: T,
        prev_inst: T,
    ) -> Panic {
        Panic::UsmError(format!(
            "ПОМИЛКА: на лініЇ {line}, токен {token_count}

  {prev_line}    {prev_inst}
  {line}    {token} {next_token}   <-- {msg}
                     ",
            prev_line = line - 1
        ))
    }

    struct InvalidInst {
        err_msg: String,
        body: String,
        operand: String,
    }

    struct Label {
        name: String,
        addr: usize,
    }

    let mut program = Vec::<Result<Instruction, InvalidInst>>::new();
    let mut lables_table = Vec::<Label>::new();
    let mut inst_addr = 0;
    let mut token_count = 0;
    let lines = source
        .lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .map(|line| line.split_once('#').map(|(l, _)| l).unwrap_or(line));

    for (line, line_num) in lines.zip(1..) {
        let mut tokens = line.split_whitespace();
        while let Some(token) = tokens.next() {
            token_count += 1;
            let token = token.trim();
            if let Some(label) = token.strip_suffix(':') {
                lables_table.push((inst_addr, label));
                continue;
            }
            let conditional = token.ends_with('?');
            let token = token.strip_suffix('?').unwrap_or(token);
            let kind = InstructionKind::try_parse(token).map_err(|_| {
                fmt_err(
                    "невідома інструкція",
                    line_num,
                    token_count,
                    token,
                    tokens.next().unwrap_or(""),
                    program.get_last().to_string().as_str(),
                )
            })?;
            let with_operand = kind.has_operand();
            let operand = if with_operand {
                let token = tokens.next().ok_or(fmt_err(
                    "інструкція без операнду",
                    line_num,
                    token_count,
                    token,
                    tokens.next().unwrap_or(""),
                    program.get_last().to_string().as_str(),
                ))?;
                token_count += 1;
                if let Ok(val) = Value::try_parse(token) {
                    val
                } else if let Some((val, _)) = lables_table
                    .items
                    .iter()
                    .find(|(_, label)| label.contains(token))
                {
                    Value::Uint(*val)
                } else {
                    return Err(fmt_err(
                        "нелегальний операнд",
                        line_num,
                        token_count,
                        token,
                        tokens.next().unwrap_or(""),
                        program.get_last().to_string().as_str(),
                    ));
                }
            } else {
                Value::Null
            };

            program.push(Ok(Instruction {
                kind,
                operand,
                conditional,
            }));
            inst_addr += 1;
        }
    }

    Ok(program)
}
