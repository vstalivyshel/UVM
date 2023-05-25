mod utils;

const VM_STACK_CAPACITY: usize = 1024;

#[derive(Copy, Clone, Debug)]
pub enum Panic {
    StackOverflow,
    StackUnderflow,
    IlligalInstruction {
        inst: Instruction,
        val_a: Value,
        val_b: Value,
    },
}

#[derive(Copy, Clone, Debug)]
pub enum Instruction {
    Push(Value),
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
    pub stack: [Value; VM_STACK_CAPACITY],
    pub stack_size: usize,
    pub debug: bool,
}

impl VM {
    fn init() -> Self {
        Self {
            stack: [Value::Null; VM_STACK_CAPACITY],
            stack_size: 0,
            debug: false,
        }
    }

    fn load_from_memmory(
        &mut self,
        program: &[Instruction],
    ) -> Result<(), Panic> {
        for inst in program.into_iter() {
            if self.debug {
                println!("{}", self);
                println!("ВИКОНУЮ ІНСТРУКЦІЮ {}", inst);
            }
            self.execute_instruction(*inst)?;
        }

        Ok(())
    }

    fn execute_instruction(&mut self, inst: Instruction) -> Result<(), Panic> {
        use Instruction::*;

        fn op<F>(vm: &mut VM, f: F) -> Result<(), Panic>
        where
            F: Fn(isize, isize) -> Result<isize, Panic>
        {
            match (vm.stack_pop()?, vm.stack_pop()?) {
                (Value::Number(a), Value::Number(b)) => vm.stack_push(Value::Number(f(a, b)?)),
                (val_a, val_b) => Err(Panic::IlligalInstruction {
                    inst: Sum,
                    val_a,
                    val_b,
                }),
            }
        }

        match inst {
            Push(value) => self.stack_push(value),
            Sum => op(self, |a,  b| Ok(a + b)),
            Sub => op(self, |a,  b| Ok(a - b)),
            Mul => op(self, |a,  b| Ok(a * b)),
            Div => op(self, |a,  b| Ok(a / b)),
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
    ];

    let mut vm = VM::init();
    vm.debug = true;
    let _ = vm.load_from_memmory(&program);
    println!("{}", vm);
}
