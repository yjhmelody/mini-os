# Mini OS notes

## VGA Text Mode

### The VGA Text Buffer

VGA's address is 0xb8000

To print a character to the screen in VGA text mode, one has to write it to the text buffer of the VGA hardware. The VGA text buffer is a two-dimensional array with typically 25 rows and 80 columns, which is directly rendered to the screen. Each array entry describes a single screen character through the following format:

|Bit(s)	|Value|
|0-7|	ASCII code| point|
|8-11	|Foreground color|
|12-14	|Background color|
|15|	Blink|

The VGA text buffer is accessible via memory-mapped I/O to the address 0xb8000. 
This means that reads and writes to that address don't access the RAM, 
but directly the text buffer on the VGA hardware. 
This means that we can read and write it through normal memory operations to that address.

The following colors are available:

Number	Color	Number + Bright Bit	Bright Color
0x0	Black	0x8	Dark Gray
0x1	Blue	0x9	Light Blue
0x2	Green	0xa	Light Green
0x3	Cyan	0xb	Light Cyan
0x4	Red	0xc	Light Red
0x5	Magenta	0xd	Pink
0x6	Brown	0xe	Yellow
0x7	Light Gray	0xf	White

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
again. To do this, we add an implementation for the new_line method of Writer

## Spinlocks
To get synchronized interior mutability, users of the standard library 
can use Mutex. It provides mutual exclusion by blocking threads when 
the resource is already locked. But our basic kernel does not have any 
blocking support or even a concept of threads, so we can't use it either. 
However there is a really basic kind of mutex in computer science that 
requires no operating system features: the spinlock. Instead of blocking, 
the threads simply try to lock it again and again in a tight loop and thus burn CPU time until the mutex is free again.


