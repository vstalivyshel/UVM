mod utils;

const VM_STACK_CAPACITY: usize = 1024;
const PROGRAM_INST_CAPACITY: usize = 1024;

#[derive(Copy, Clone, Debug)]
pub enum Panic {
    StackOverflow,
    StackUnderflow,
    InvalidOperand,
    InstLimitkOverflow,
    DivByZero,
}

#[derive(Copy, Clone, Debug)]
pub enum Instruction {
    Nop,
    Push(isize),
    Drop,
    Dup,
    DupAt(isize),
    JumpIf(isize),
    Jump(isize),
    Eq,
    Sub,
    Mul,
    Div,
    Sum,
}

pub struct VM {
    stack: [isize; VM_STACK_CAPACITY],
    stack_size: usize,
    program: [Instruction; PROGRAM_INST_CAPACITY],
    program_size: usize,
    inst_ptr: usize,
    debug: VMDebug,
}

impl VM {
    fn init() -> Self {
        Self {
            stack: [0; VM_STACK_CAPACITY],
            stack_size: 0,

            program: [Instruction::Nop; PROGRAM_INST_CAPACITY],
            program_size: 0,
            inst_ptr: 0,

            debug: VMDebug::off(),
        }
    }

    fn load_from_memmory(&mut self, program: &[Instruction]) -> Result<(), Panic> {
        let program_size = program.len();
        if program_size > PROGRAM_INST_CAPACITY {
            return Err(Panic::InstLimitkOverflow);
        }

        self.program_size = program_size;

        for (i, inst) in program.iter().enumerate() {
            self.program[i] = *inst;
        }

        Ok(())
    }

    fn execute(&mut self) -> Result<(), Panic> {
        while self.inst_ptr < self.program_size {
            self.execute_instruction()?;
            self.inst_ptr += 1;
        }

        if self.debug.stack {
            println!("{}", self);
        }

        Ok(())
    }

    fn execute_instruction(&mut self) -> Result<(), Panic> {
        if self.debug.stack {
            println!("{}", self);
        }

        let current_inst = self.program[self.inst_ptr];

        if self.debug.inst {
            println!("- ІНСТ({ip}): {current_inst}", ip = self.inst_ptr);
        }

        fn push_from<F>(state: &mut VM, f: F) -> Result<(), Panic>
        where
            F: Fn(isize, isize) -> Result<isize, Panic>,
        {
            let (a, b) = (state.stack_pop()?, state.stack_pop()?);
            state.stack_push(f(a, b)?)
        }

        use Instruction::*;
        match current_inst {
            Nop => Ok(()),
            Push(value) => self.stack_push(value),
            Drop => {
                self.stack_size -= 1;
                self.stack[self.stack_size] = 0;
                Ok(())
            }
            DupAt(addr) => {
                let addr = addr + 1;
                if addr < 0 || addr as usize > self.inst_ptr {
                    return Err(Panic::InvalidOperand);
                }

                self.stack_push(self.stack[addr as usize])
            }
            Dup => {
                let target = self.stack_pop()?;
                self.stack_push(target)?;
                self.stack_push(target)
            }
            JumpIf(addr) => {
                if self.stack_size < 1 {
                    return Err(Panic::StackUnderflow);
                }

                if self.stack[self.stack_size] != 0 {
                    self.inst_ptr = addr as usize;
                }

                Ok(())
            },
            Jump(addr) => {
                if addr < 0 || addr as usize > self.program_size {
                    return Err(Panic::InvalidOperand);
                }

                self.inst_ptr = addr as usize;

                Ok(())
            },
            Eq => {
                if self.stack_size < 2 {
                    return Err(Panic::StackUnderflow);
                }

                let a = self.stack[self.stack_size];
                let b = self.stack[self.stack_size - 1];

                self.stack_push(if a == b {
                    1
                } else {
                    0
                })
               
            },
            Sum => push_from(self, |a, b| Ok(b + a)),
            Sub => push_from(self, |a, b| Ok(b - a)),
            Mul => push_from(self, |a, b| Ok(b * a)),
            Div => push_from(self, |a, b| {
                if a == 0 {
                    Err(Panic::DivByZero)
                } else {
                    Ok(b / a)
                }
            }),
        }
    }

    fn stack_push(&mut self, value: isize) -> Result<(), Panic> {
        if self.stack_size == VM_STACK_CAPACITY {
            Err(Panic::StackOverflow)
        } else {
            self.stack[self.stack_size] = value;
            self.stack_size += 1;
            Ok(())
        }
    }

    fn stack_pop(&mut self) -> Result<isize, Panic> {
        if self.stack_size == 0 {
            Err(Panic::StackUnderflow)
        } else {
            self.stack_size -= 1;
            let value = self.stack[self.stack_size];
            self.stack[self.stack_size] = 0;
            Ok(value)
        }
    }
}

pub struct VMDebug {
    stack: bool,
    inst: bool,
}

impl VMDebug {
    fn _full() -> Self {
        Self {
            stack: true,
            inst: true,
        }
    }

    fn off() -> Self {
        Self {
            stack: false,
            inst: false,
        }
    }
}

fn main() {
    use Instruction::*;
    let program = [
        Push(60),
        Push(9),
        Push(50),
        Dup,
    ];

    let mut state = VM::init();

    state.debug.stack = true;
    state.debug.inst = false;

    if let Err(vm_err) = state.load_from_memmory(&program) {
        eprintln!("[!] ПАНІКА: {vm_err}");
    }

    if let Err(vm_err) = state.execute() {
        eprintln!("[!] ПАНІКА: {vm_err}");
    }
}
