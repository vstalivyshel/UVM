## Ukrainian Virtual Machine  
### Very much WIP
Like JVM, but in Ukrainian.  

Source of inspiration: https://github.com/tsoding/bm

### Usage:
UVM bytecode "oject" file contains a serialized USM (Ukrainian assembly) instructions that can be executed using the `emu` subcommand.  
Also, you can execute a USM file without translating it into bytecode using the `-usm` flag or just run this subcommand on a file with the extension `.usm`.  

- emu - run the instructions from the provided file.
```
./uvm emu [OPT] <FILE>

[OPT]
    -usm - translate the USM instructions from the file <FILE> and execute them
    -l <NUM> - set a limit on executed instructions
    -ds - dump all changes to the stack while executing the instructions
    -di - dump list of each executed instruction
```

- dusm - translate the USM (assembly) from the file into bytecode.
```
./uvm dusm [OPT] <FILE>

[OPT]
    -o <OUTPUT FILE> - write translated into bytecode instructions into the <OUTPUT FILE>
```


- usm - translate the bytecode of instructions from the file into the USM
```
./uvm usm [OPT] <FILE>

[OPT]
    -o <OUTPUT FILE> - write translated USM instructions into the <OUTPUT FILE> (default is stdout)
```


- dump - read the instructions from the file without execution and dump them into stdout

```
./uvm dump [OPT] <FILE>
[OPT]
    -l <NUM> - set a limit on dumped instructions
    -usm - translate the USM instructions from the file <FILE> before dumping
```

### Examples (assembly)
- Basics
```

клади 60 	;; push 60 on the stack
клади 9  	;; push 9 on the stack
сума     	;; sum the top values of the stack
копію 0   	;; duplicate the top value (stack indexed from zero)
рівн     	;; push 1 if two top values are equal, otherwise, push 0

;; Everything that has ':' as a suffix will be treated as a label,
;; which will be expanded to the instruction address that is labeled
собака: клади 1 ;; Instruction 0
собака2:
	клади 2     ;; 1
собака3:

клади 3         ;; 2

;; After labeling an instruction, you can use 'крок' (jump) with the label name as an argument
крок собака3 	;; Jump to instruction 2
крок собака2 	;; Jump to 1
крок собака  	;; Jump to 0

;; You can separate instruction blocks using labels, 'вертай' (return) instruction and 'клич' (call) instruction
крок головний_блок      ;; jump to the 'main instruction block'

головний_блок:          ;; main instruction block
	клич клади_число_42 ;; 'клич' (call) instruction will push the address of the next
                        ;; instruction on the stack and will jump to the specified label

	кінчай              ;; the 'stop' instruction that will terminate the program
	кинь                ;; the 'drop' will not be executed
	
клади_число_42:         ;; define an entry point to the set of instructions
	клади 21            ;; push 21
	клади 2             ;; push 2
	множ                ;; multiply
	міняй 1             ;; at this point, the stack will have 42 at the top (0 idx) and after is the address of the 'кінчай' instruction (1 idx)
	                    ;; so we need to swap them to pop the instruction address and jump to it, leaving 42 untoched 

	вертай              ;; 'return' will pop the top value and jump to the instruction with this address

;; Each instruction can have '?' as suffix, which indicates that
;; it will be executed only if the top value of the stack is greater than zero
;; '?' operator will pop the top value, so it might be useful to 'копію 0' duplicate
;; that value or to use one of the comparison instructions: 'рівн' (equale) or 'нерівн' (not equale), which will push 1 or 0 on the stack.
клади 1 	;; push 1 on top of the stack
клади 2 	;; push 2
рівн    	;; check for equality: false, so push 0
сума?   	;; this instruction will be skipped: top == 0
клади 2 	;; the value doesn't need to be 1 to represent a true statement
сума?   	;; this will drop the top value and execute itself

```
- Types and casting
```
;; The type of operands can be specified with the fallowing syntax:
клади 1_дроб   ;; float 
клади 2_ціл    ;; unsigned integer
клади 3_зціл   ;; signed integer
клади 4.       ;; float
клади 5.0      ;; float
клади 6        ;; signed integer
клади -7       ;; signed integer

;; Wrong annotation will be treated as a label:
клади -1_ціл   ;; unsigned integer with a sign
клади 2.0_ціл  ;; unsigned integer with a floating point?
клади 3._зціл  ;; signed integer with a floating point

;; Math instructions will use the type of the top value
клади 10_дроб  ;; push 10 float
клади 5_ціл    ;; push 5 uint
різн           ;; 10 float - 5 uint = 5 uint

;; Use this example, if you want to change the type of value for further operations:
клади 10       ;; push 10 int
клади 5        ;; push 5 int

клади 0_дроб   ;; push 0 float
сума           ;;  5 int + 0 float = 5 float

різн           ;; 10 int - 5 float = 5 float

;; To change the sign of a signed integer or float:
клади 5

копію 0        ;; copy top
клади 2        ;; push 2
множ           ;; multiply
різн           ;; substruct: 5 - 5 * 2

;; Or:
клади 5

клади 0        ;; push 0
міняй 1        ;; swap 0 and 5
різн           ;; substruct: 0 - 5

```
- For loop:
```
;; The loop will iterate until it reaches 10

клади 10
клади 0             ;; starting point
киця:
    клади 1         ;; this instruction is labeled as 'киця' and this is our step instruction
    сума            ;; pop 1 and 0 and push their sum
    нерівн          ;; 'Not Equale' instruction will push 1
    крок? киця      ;; check if top > 0 and decide whether to jump to a label or not

    кінчай          ;; terminate the program

```

- For loop #2
```
;; This one will iterate until it reaches 0

клади 10
клади 10            ;; starting point
клади 0             ;; a little bit of a hack to have a 'кинь' (drop) instruction in the beginning of the loop

абоба:
    кинь
    клади -1        ;; step
    сума
    клади 0         ;; push 0 to compare with our initial value, then we will drop this one on the next iteration
    нерівн          ;; not equal to zero at this point, so push 1
    крок? абоба     ;; pop and check if larger than 0 and then jump

    кінчай          ;; terminate the program 
```

