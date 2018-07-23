# Mini OS notes

## The VGA Text Buffer

VGA's address is 0xb8000.

To print a character to the screen in VGA text mode, one has to write it to the text buffer of the VGA hardware. 
The VGA text buffer is a two-dimensional array with typically 25 rows and 80 columns, which is directly 
rendered to the screen. Each array entry describes a single screen character through the following format:

|Bit(s) |Value              |
| -     | -                 | 
|0-7    |ASCII code| point  |
|8-11	|Foreground color   |
|12-14	|Background color   |
|15     |Blink              |

The VGA text buffer is accessible via memory-mapped I/O to the address 0xb8000. 
This means that reads and writes to that address don't access the RAM, 
but directly the text buffer on the VGA hardware. 
This means that we can read and write it through normal memory operations to that address.

The following colors are available:

|Number |Color      |Number + Bright Bit|Bright Color   
|-      |-          |-                  |-              
|0x0    |Black      |0x8                |Dark Gray 
|0x1	|Blue	    |0x9            	|Light Blue
|0x2	|Green	    |0xa        	    |Light Green
|0x3	|Cyan	    |0xb    	        |Light Cyan
|0x4	|Red	    |0xc        	    |Light Red
|0x5	|Magenta    |0xd        	    |Pink
|0x6	|Brown	    |0xe        	    |Yellow
|0x7	|Light Gray |0xf                |White

## Volatile

The problem is that we only write to the Buffer and never read from 
it again. The compiler doesn't know that we really access VGA buffer 
memory (instead of normal RAM) and knows nothing about the side effect 
that some characters appear on the screen. So it might decide that 
these writes are unnecessary and can be omitted. To avoid this erroneous 
optimization, we need to specify these writes as volatile. This tells 
the compiler that the write has side effects and should not be optimized away.

## Newlines

Right now, we just ignore newlines and characters that don't fit into 
the line anymore. Instead we want to move every character one line up 
(the top line gets deleted) and start at the beginning of the last line 
again. To do this, we add an implementation for the new_line method of Writer.

## Spinlocks

To get synchronized interior mutability, users of the standard library 
can use Mutex. It provides mutual exclusion by blocking threads when 
the resource is already locked. But our basic kernel does not have any 
blocking support or even a concept of threads, so we can't use it either. 
However there is a really basic kind of mutex in computer science that 
requires no operating system features: the spinlock. Instead of blocking, 
the threads simply try to lock it again and again in a tight loop and thus burn CPU time until the mutex is free again.


## The Serial Port

The naive way of doing an integration test would be to add some assertions in the code, 
launch QEMU, and manually check if a panic occured or not. This is very cumbersome and 
not practical if we have hundreds of integration tests. So we want an automated solution 
that runs all tests and fails if not all of them pass.

Such an automated test framework needs to know whether a test succeeded or failed. 
It can't look at the screen output of QEMU, so we need a different way of retrieving the test 
results on the host system. A simple way to achieve this is by using the serial port, 
an old interface standard which is no longer found in modern computers. It is easy to program 
and QEMU can redirect the bytes sent over serial to the host's standard output or a file.

The chips implementing a serial interface are called UARTs. There are lots of UART models on x86, 
but fortunately the only differences between them are some advanced features we don't need. 
The common UARTs today are all compatible to the 16550 UART, so we will use that model for our testing framework.

## Port I/O

There are two different approaches for communicating between the CPU and peripheral hardware on x86, 
memory-mapped I/O and port-mapped I/O. We already used memory-mapped I/O for accessing the VGA text buffer 
through the memory address 0xb8000. This address is not mapped to RAM, but to some memory on the GPU.

In contrast, port-mapped I/O uses a separate I/O bus for communication. Each connected 
peripheral has one or more port numbers. To communicate with such an I/O port there are 
special CPU instructions called in and out, which take a port number and a data byte 
(there are also variations of these commands that allow sending an u16 or u32).

The UART uses port-mapped I/O. Fortunately there are already several crates that provide abstractions 
for I/O ports and even UARTs, so we don't need to invoke the in and out assembly instructions manually.

## CPU Exceptions

On x86 there are about 20 different CPU exception types. The most important are:

- Page Fault: A page fault occurs on illegal memory accesses. For example, if the current instruction tries to read from an unmapped page or tries to write to a read-only page.
- Invalid Opcode: This exception occurs when the current instruction is invalid, for example when we try to use newer SSE instructions on an old CPU that does not support them.
- General Protection Fault: This is the exception with the broadest range of causes. It occurs on various kinds of access violations such as trying to executing a privileged instruction in user level code or writing reserved fields in configuration registers.
- Double Fault: When an exception occurs, the CPU tries to call the corresponding handler function. If another exception occurs while calling the exception handler, the CPU raises a double fault exception. This exception also occurs when there is no handler function registered for an exception.
- Triple Fault: If an exception occurs while the CPU tries to call the double fault handler function, it issues a fatal triple fault. We can't catch or handle a triple fault. Most processors react by resetting themselves and rebooting the operating system.

## The Interrupt Descriptor Table

In order to catch and handle exceptions, we have to set up a so-called Interrupt Descriptor Table (IDT). 
In this table we can specify a handler function for each CPU exception. The hardware uses this table directly, 
so we need to follow a predefined format. Each entry must have the following 16-byte structure:

|Type	|Name	|Description
|-      |-      |-          
|u16	|Function Pointer [0:15]	|The lower bits of the pointer to the handler function.
|u16	|GDT selector	|Selector of a code segment in the global descriptor table.
|u16	|Options	|(see below)
|u16	|Function Pointer [16:31]	|The middle bits of the pointer to the handler function.
|u32	|Function Pointer [32:63]	|The remaining bits of the pointer to the handler function.
|u32	|Reserved|


The options field has the following format:

|Bits	|Name	|Description |
|-      |-                                  |-                              |
|0-2	|Interrupt Stack Table Index	    |0: Don't switch stacks, 1-7: Switch to the n-th stack in the Interrupt Stack Table when this handler is called.
|3-7	|Reserved
|8	    |0: Interrupt Gate, 1: Trap Gate	|If this bit is 0, interrupts are disabled when this handler is called.
|9-11	|must be one
|12     |must be zero
|13‑14  |Descriptor Privilege Level (DPL)	|The minimal privilege level required for calling this handler.
|15	    |  Present

Each exception has a predefined IDT index. For example the invalid opcode exception has table index 6 and 
the page fault exception has table index 14. Thus, the hardware can automatically load the 
corresponding IDT entry for each exception. The Exception Table in the OSDev wiki shows the IDT 
indexes of all exceptions in the “Vector nr.” column.

When an exception occurs, the CPU roughly does the following:

- Push some registers on the stack, including the instruction pointer and the RFLAGS register. (We will use these values later in this post.)
- Read the corresponding entry from the Interrupt Descriptor Table (IDT). For example, the CPU reads the 14-th entry when a page fault occurs.
- Check if the entry is present. Raise a double fault if not.
- Disable hardware interrupts if the entry is an interrupt gate (bit 40 not set).
- Load the specified GDT selector into the CS segment.
- Jump to the specified handler function.

## The Interrupt Calling Convention

Calling conventions specify the details of a function call. For example, they specify where 
function parameters are placed (e.g. in registers or on the stack) and how results are returned. 
On x86_64 Linux, the following rules apply for C functions (specified in the System V ABI):

- the first six integer arguments are passed in registers rdi, rsi, rdx, rcx, r8, r9
- additional arguments are passed on the stack
- results are returned in rax and rdx

Note that Rust does not follow the C ABI (in fact, there isn't even a Rust ABI yet), 
so these rules apply only to functions declared as extern "C" fn.

## Preserved and Scratch Registers

The calling convention divides the registers in two parts: preserved and scratch registers.

The values of preserved registers must remain unchanged across function calls. So a called function 
(the “callee”) is only allowed to overwrite these registers if it restores their original values before returning. 
Therefore these registers are called “callee-saved”. A common pattern is to save these registers to the stack 
at the function's beginning and restore them just before returning.

In contrast, a called function is allowed to overwrite scratch registers without restrictions. 
If the caller wants to preserve the value of a scratch register across a function call, it needs to 
backup and restore it before the function call (e.g. by pushing it to the stack). So the scratch 
registers are caller-saved.

On x86_64, the C calling convention specifies the following preserved and scratch registers:

|preserved registers	            |scratch registers                              |
|-                                  |-                                              |      
|rbp, rbx, rsp, r12, r13, r14, r15  |rax, rcx, rdx, rsi, rdi, r8, r9, r10, r11      |
|callee-saved	                    |caller-saved                                   |

The compiler knows these rules, so it generates the code accordingly.

Since we don't know when an exception occurs, we can't backup any registers before. 
This means that we can't use a calling convention that relies on caller-saved registers for 
exception handlers. Instead, we need a calling convention means that preserves all registers. 
The x86-interrupt calling convention is such a calling convention, so it guarantees that 
all register values are restored to their original values on function return.

## The Exception Stack Frame

For exception and interrupt handlers, however, pushing a return address would not suffice, 
since interrupt handlers often run in a different context (stack pointer, CPU flags, etc.). 
Instead, the CPU performs the following steps when an interrupt occurs:

## Behind the Scenes

## BreakPoint Exception

The breakpoint exception is commonly used in debuggers: When the user sets a breakpoint, 
the debugger overwrites the corresponding instruction with the int3 instruction so that 
the CPU throws the breakpoint exception when it reaches that line. When the user wants to 
continue the program, the debugger replaces the int3 instruction with the original instruction 
again and continues the program. 

## Double Fault

In simplified terms, a double fault is a special exception that occurs when the CPU fails to 
invoke an exception handler. For example, it occurs when a page fault is triggered but there is 
no page fault handler registered in the Interrupt Descriptor Table (IDT). So it's kind of similar 
to catch-all blocks in programming languages with exceptions, e.g. catch(...) in C++ or 
catch(Exception e) in Java or C#.


## Causes of Double Faults

|First Exception    |   Second Exception|
|-                  |-                  |
|Divide-by-zero, Invalid TSS, Segment Not Present, Stack-Segment Fault, General Protection Fault    | Invalid TSS, Segment Not Present, Stack-Segment Fault, General Protection Fault
|Page Fault	        |Page Fault, Invalid TSS, Segment Not Present, Stack-Segment Fault, General Protection Fault


## Switching Stacks

The x86_64 architecture is able to switch to a predefined, known-good stack when an exception occurs. 
This switch happens at hardware level, so it can be performed before the CPU pushes the exception stack frame.
The switching mechanism is implemented as an Interrupt Stack Table (IST). 
The IST is a table of 7 pointers to known-good stacks.

## The IST and TSS

The Interrupt Stack Table (IST) is part of an old legacy structure called Task State Segment (TSS). 
The TSS used to hold various information (e.g. processor register state) about a task in 32-bit mode 
and was for example used for hardware context switching. However, hardware context switching is no longer 
supported in 64-bit mode and the format of the TSS changed completely.

On x86_64, the TSS no longer holds any task specific information at all. Instead, 
it holds two stack tables (the IST is one of them). The only common field between the 32-bit 
and 64-bit TSS is the pointer to the I/O port permissions bitmap.

he 64-bit TSS has the following format:

|Field                  |Type
|-                      |-|
|(reserved)	            |u32
|Privilege Stack Table	|[u64; 3]
|(reserved)	            |u64
|Interrupt Stack Table	|[u64; 7]
|(reserved)	            |u64
|(reserved)	            |u16
|I/O Map Base Address	|u16


## The Global Descriptor Table

The Global Descriptor Table (GDT) is a relict that was used for memory segmentation before paging became the de facto standard. 
It is still needed in 64-bit mode for various things such as kernel/user mode configuration or TSS loading.

The GDT is a structure that contains the segments of the program. 
It was used on older architectures to isolate programs from each other, before paging became the standard.
While segmentation is no longer supported in 64-bit mode, the GDT still exists. 
It is mostly used for two things: Switching between kernel space and user space, and loading a TSS structure.

