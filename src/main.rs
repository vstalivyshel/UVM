mod utils;

const VM_STACK_CAPACITY: usize = 1024;
const PROGRAM_INST_CAPACITY: usize = 1024;

#[derive(Copy, Clone, Debug)]
pub enum Panic {
    StackOverflow,
    StackUnderflow,
    InstLimitkOverflow(usize),
    EofReached,
    DivByZero,
    IncompatibleValue {
        inst: Instruction,
        val_a: Value,
        val_b: Value,
    },
}

#[derive(Copy, Clone, Debug)]
pub enum Instruction {
    Push(Value),
    Jump(usize),
    Eof,
    Sub,
    Mul,
    Div,
    Sum,
}

#[derive(Copy, Clone, Debug)]
pub enum Value {
    Number(isize),
    Null,
}

pub struct VM {
    stack: [Value; VM_STACK_CAPACITY],
    stack_size: usize,
    program: [Instruction; PROGRAM_INST_CAPACITY],
    inst_len: usize,
    inst_ptr: usize,
    debug: Dbg,
}

pub struct Dbg {
    stack: bool,
    inst: bool,
}

impl Dbg {
    fn full() -> Self {
        Self { stack: true, inst: true }
    }

    fn off() -> Self {
        Self { stack: false, inst: false }
    }
}


impl VM {
    fn init() -> Self {
        Self {
            stack: [Value::Null; VM_STACK_CAPACITY],
            program: [Instruction::Eof; PROGRAM_INST_CAPACITY],
            stack_size: 0,
            inst_len: 0,
            inst_ptr: 0,
            debug: Dbg::off(),
        }
    }

    fn load_from_memmory(&mut self, program: &[Instruction]) -> Result<(), Panic> {
		let prog_len = program.len();

        if prog_len > PROGRAM_INST_CAPACITY {
            return Err(Panic::InstLimitkOverflow(prog_len));
        }

        self.inst_len = prog_len;
        self.inst_ptr = prog_len;

        for (i, inst) in program.iter().enumerate() {
            self.program[i] = *inst;
        }

        Ok(())
    }

    fn execute(&mut self) -> Result<(), Panic> {
        while self.inst_ptr != 0 {
            self.execute_instruction()?;
            self.inst_ptr -= 1;
        }

        if self.debug.stack {
            println!( "{}", self );
        }

        Ok(())
    }

    fn execute_instruction(&mut self) -> Result<(), Panic> {
        use Instruction::*;
        fn op<F>(vm: &mut VM, f: F) -> Result<(), Panic>
        where
            F: Fn(isize, isize) -> Result<isize, Panic>,
        {
            match (vm.stack_pop()?, vm.stack_pop()?) {
                (Value::Number(a), Value::Number(b)) => vm.stack_push(Value::Number(f(a, b)?)),
                (val_a, val_b) => Err(Panic::IncompatibleValue {
                    inst: Sum,
                    val_a,
                    val_b,
                }),
            }
        }

        if self.debug.stack {
            println!( "{}", self );
        }

        let current_inst = self.program[self.inst_len - self.inst_ptr];

        if self.debug.inst {
            println!( "[ІНСТ({ip})] {current_inst}", ip = self.inst_ptr - 1);
        }

        match  current_inst {
            Eof => Err(Panic::EofReached),
            Push(value) => self.stack_push(value),
            Jump(addr) => {
                if addr > self.inst_len {
                    return Err(Panic::StackUnderflow);
                }
                self.inst_ptr += addr;
                self.execute_instruction()
            }
            Sum => op(self, |a, b| Ok(b + a)),
            Sub => op(self, |a, b| Ok(b - a)),
            Mul => op(self, |a, b| Ok(b * a)),
            Div => op(self, |a, b| {
                if a == 0 {
                    Err(Panic::DivByZero)
                } else {
                    Ok(b / a)
                }
            }),
        }
    }

    fn stack_push(&mut self, value: Value) -> Result<(), Panic> {
        if self.stack_size == VM_STACK_CAPACITY {
            Err(Panic::StackOverflow)
        } else {
            self.stack[self.stack_size] = value;
            self.stack_size += 1;
            Ok(())
        }
    }

    fn stack_pop(&mut self) -> Result<Value, Panic> {
        if self.stack_size == 0 {
            Err(Panic::StackUnderflow)
        } else {
            self.stack_size -= 1;
            let value = self.stack[self.stack_size];
            self.stack[self.stack_size] = Value::Null;
            Ok(value)
        }
    }
}

fn main() {
    use Instruction::*;
    use Value::*;

    let program = [
        Push(Number(1)),
        Push(Number(2)),
        Sum,
        Push(Number(60)),
        Push(Number(9)),
        Sum,
        Push(Number(-1)),
        Sub,
        Push(Number(-1)),
        Mul,
        Div,
        Push(Number(46)),
        Sum,
        Push(Number(10)),
        Push(Number(1)),
        Sum,
        // Current instruction is 0
        Jump(1),
    ];

    let mut state = VM::init();

    state.debug = Dbg::full();

    if let Err(vm_err) = state.load_from_memmory(&program) {
        eprintln!("[!] ПАНІКА: {vm_err}");
    }

    if let Err(vm_err) = state.execute() {
        match vm_err {
            Panic::EofReached => {}
            err => eprintln!("[!] ПАНІКА: {err}"),
        }
    }
}
