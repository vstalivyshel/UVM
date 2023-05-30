
### Ukrainian Virtual Machine  
### Very much WIP

Source of inspiration: https://github.com/tsoding/bm

Goals:  
- Learn things.  
- Build a simple virtual machine.  
- Build an assembly for this VM.  
- Try to create a programming language using the assembly of my VM.

And all that great stuff will be interfaced using Ukrainian.

Progress:

- [x] Basic stack implementation:  
```  
Basic instructions: Push, Drop, Dup, DupAt  
Binary instructions: Sub, Sum, Mul, Div, Eq  
Flow instructions: Jump, JumpIf  
```

- [x] (De)serealization of the instructions, which allow UVM to read/write byte code from/to a file.  
- [ ] CLI:  
```
-b - indicates that the loaded file contains binary instructions
-l <N> - limit number of executed instructions to <N>
-di - dump each executed instruction to the stdout  
-ds - dump stack values after each executed instruction  
```

- [ ] Assembler and disassembler for USM (human-readable format for writing the instructions)
```
# This is not the final version

клади 60 #  push 60 on the stack
клади 9 # push 9 on the stack
сума # sum the top values of the stack
копію # duplicate the top value
рівн # push 1 if two top values are equal, otherwise, push 0
```
- [ ] Ua-En dictionary for the instructions

- [ ] TODO



