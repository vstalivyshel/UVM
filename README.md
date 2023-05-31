
### Ukrainian Virtual Machine  
### Very much WIP

Source of inspiration: https://github.com/tsoding/bm

Goals:  
- Learn things.  
- Build a simple virtual machine.  
- Build an assembly for this VM.  
- Try to create a programming language using the assembly of my VM.

And all that great stuff will be interfaced in Ukrainian.

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

- [ ] Assembly
```
# This is not the final version

клади 60 	#  push 60 on the stack
клади 9  	# push 9 on the stack
сума     	# sum the top values of the stack
копію    	# duplicate the top value
рівн     	# push 1 if two top values are equal, otherwise, push 0

# Everything that have ':' as suffix will be treated as a label,
# which will be expanded to the instruction address that is labeled
собака: клади 1 # Instruction 0
собака2:
	клади 2     # 1
собака3:

клади 3         # 2

# After labeling instruction you can use 'крок' (jump) with the label name as an argument
крок собака3 # Jump to instruction 2
крок собака2 # Jump to 1
крок собака # Jump to 0

# Each instruction can have '?' as suffix, which indicates that
# it will be execute only if the top value of the stack is greater that zero
клади 1 	# push 1 on top of the stack
клади 2 	# push 2
рівн    	# check for equality: false, so push 0
сума?   	# this instruction will be skipped: top == 0
клади 2 	# the value doesn't need to be 1 to represent a true statement
сума?   	# this will be executed: 2 != 0

```
- [ ] Ua-En dictionary for the instructions

- [ ] TODO

