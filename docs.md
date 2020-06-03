# Machine Language

The assembly language you can compile and run on the machine.

Taking inspiration from RISC-V, the machine language instruction set is very
limited. It can be extended by adding various modules in the form of additional
hardware or extra CPUs.

## Considerations

* Register machine or stack language? Or something else?
* Turing complete?
* Limited memory
* Limited registers (if register machine)
* System calls or memory mapped IO
* Flags: Carry, Zero, Overflow, etc.
* Sign extension

## File layout

* file extension: `.ax` (assembly language extended)
* `section .static` (case-insensitive) on its own line
  * contains static data declarations
  * the data is laid out exactly as specified, in the order specified, with no
    additional padding inserted between items of different sizes
* `section .code` (case-insensitive) on its own line
  * contains source code (instructions)
  * executes from top to bottom
* The sections are ordered: `.static`, `.code`

## Static Data Declaration Syntax

Used in the `.static` section.

* label
  * an ASCII alphabetic character followed by any number of alphanumeric characters
  * e.g. `abc`, `L1`, `x2`
  * use `label:` to designate the address of a given section of the executable
* string literal
  * single or double quoted ASCII characters, e.g. `'a'`, `"123abc\n"`
  * supports string escapes like `\n`, `\t`, `\x{FF}`, `\b{00011000}`
* `.b1`, `.b2`, `.b4`, `.b8`
  * declare and initialize 1, 2, 4, or 8 bytes to a given value
  * e.g. `.b1 3` initializes a byte to the value 3
  * negative values are initialized as two's complement values
* `.zero`
  * fills a given number of bytes with zero
  * e.g. `.zero 100` initializes 100 bytes to zero
* `.uninit`
  * declares the given number of bytes but does not initialize them
  * e.g. `.uninit 30` declares 30 uninitialized bytes
* `.bytes`
  * declares and initializes bytes to the ASCII values of each character in the
    given string literal
  * e.g. `.bytes 'hello'` initializes **5** bytes to 104, 101, 108, 108, and
    111 respectively
  * note that this does **not** add a null terminator at the end of the string
    (use `.zero 1` or `.b1 0` after `.bytes` if you need that)

Example:

```asm
section .static

# no label = data at the start of the section that cannot be explicitly
# referred to in assembly code

  .b1 100  # initialize 1 byte to the value 100

ARR1:  # a labelled region of data whose location is named `ARR1`
  .zero 10  # initialize 10 bytes to zero
```

## Instruction Syntax

Used in the `.code` section.

* comment
  * `#` or `;` character to the end of the line
* immediate
  * decimal number: `0`, `1`, `2`, `3`, `1_000_000`, etc.
  * two's complement number: `-1`, `-2`, `-3`, `0`, `1`, `2`, etc.
  * hexadecimal number: `0x123`
  * binary number: `0b0100_1000`
* label
  * an ASCII alphabetic character followed by any number of alphanumeric characters
  * e.g. `abc`, `L1`, `x2`
  * use `label:` to designate the address of a given section of the executable
* register
  * 64 general purpose registers (64-bit): `$0`, `$1`, `$2`, etc. (up to `$63`)
  * stack pointer: `$sp` - 64-bit top address of the stack (next available slot)
  * frame pointer: `$fp` - 64-bit base address of the stack (base pointer)
* data directives
  * any of the directives valid in the `.static` section may also be used in the
    `.code` section

Example:

```asm
section .code

main:
  add $1, $2   # $1 = $1 + $2
  sub $1, 5    # $1 = $1 + 5
  ret
```

## Calling Convention

* return address is stored in register `$?` (TODO)
* pop calls should be in the opposite order to push calls

## Memory Mapped IO

Before syscalls become available, IO is done through memory-mapped IO.

* When a value is stored at address `0xffff_ffff_000c`, the lower 4-bytes
  (32-bits) are sent to standard output. The bytes are interpreted as a unicode
  scalar value. If the bytes are not valid as a unique scalar value, a
  `U+FFFD REPLACEMENT CHARACTER` (&#65533;) is outputted instead.
* Loading a 1, 2, 4, or 8 byte value from address `0xffff_ffff_0004` places the
  respective next 1, 2, 4, or 8 bytes from standard input into the destination
  register. At EOF, a value of `0` will be loaded.

### Example Programs

This implements a hello world program: (filename: `hello.ax`)

```asm
section .code

main:
  push $fp
  mov $fp, $sp

  # Loop through and write each character

  # $8 = the address of the current character
  mov $8, message
  # $9 = the address one past the last character in the string
  load8 $9, length
  add $9, message

loop:
  cmp $8, $9
  jge end

  # Load the current character
  load1 $10, $8
  # Write the current character
  store8 0xffff_ffff_000c, $10
  # Move to the next character
  add $8, 1

  # Continue the loop
  jmp loop

end:
  pop $fp
  ret

section .static

# Declare a string with the message we want to print
message:
  .bytes 'hello, world!'
length:
  .b8 13
```

This implements the `cat` command: (filename: `cat.ax`)

```asm
section .code

main:
  push $fp
  mov $fp, $sp

loop:
  # Loop through and write each received byte
  load1 $0, 0xffff_ffff_0004
  # Quit at EOF
  jz end

  # Write the character
  store1 0xffff_ffff_000c, $0

  # Continue the loop
  jmp loop

end:
  pop $fp
  ret
```

## Syscalls

To make a syscall, place the syscall number in `$0` and use the `syscall`
instruction. Some syscalls take arguments in other registers. See the
documentation below for more details.

* `open`: `$0 = 0`
  * Arguments:
    * `$1` - address of file path bytes
    * `$2` - length of file path (in bytes)
  * Returns:
    * `$0` - file descriptor
  * On Error:
    * `$0` is set to -1
  * Notes:
    * Special file descriptors: `0` is stdin, `1` is stdout, `2` is stderr
* `read`: `$0 = 1`
  * Arguments:
    * `$1` - file descriptor
    * `$2` - address of buffer to read bytes into
    * `$3` - size of buffer (number of bytes to read)
  * Returns:
    * `$0` - the number of bytes that were read
  * On Error:
    * `$0` is set to -1
* `write`: `$0 = 2`
  * Arguments:
    * `$1` - file descriptor
    * `$2` - address of buffer to write bytes from
    * `$3` - size of buffer (number of bytes to write)
  * Returns:
    * `$0` - the number of bytes that were written
  * On Error:
    * `$0` is set to -1

### Example Programs

This implements a hello world program: (filename: `hello.ax`)

```asm
section .code

main:
  push $fp
  mov $fp, $sp

  # Populate arguments for write syscall

  # File descriptor 1 is stdout
  mov $1, 1
  # Set $2 to the address of the message
  mov $2, message
  # Load the value at address `length` into $3
  load8 $3, length

  # Set syscall register to value for write syscall
  mov $0, 2
  # Run the syscall
  syscall
  # Technically, we should check $0 to see if all the bytes were written

  pop $fp
  ret

section .static

# Declare a string with the message we want to print
message:
  .bytes 'hello, world!'
length:
  .b8 13
```

This implements the `cat` command: (filename: `cat.ax`)

```asm
TODO
```

## Instruction Reference

Instruction names are case-insensitive.

### Conventions

* `dest` - destination register
* `source` - operand immediate, label, or register
* `loc` - a register, or an address (usually specified using a label)

### Arithmetic

* `add dest, source` - add `source` and `dest` and put the result in `dest`
* `sub dest, source` - subtract `source` and `dest` and put the result in `dest`
* `mul`
* `div`
* `rem` - euclid rem

### Binary

* `shl`
* `shr`
* `band`
* `bor`
* `xor`

### Comparison

* `cmp source1 source2` - signed comparison

### Memory

* `mov dest, source` - copies data between registers or assigns a value
  to a register
* `load{1,2,4,8} dest, loc` or `loadu{1,2,4,8} dest, loc` - loads a value from
  memory into a register
  * The loaded value has size: 1, 2, 4, or 8 bytes
  * If `load` is used and the value is 1, 2, or 4 bytes, the value is
    sign-extended prior to being assigned into the register
  * If `loadu` is used and the value is 1, 2, or 4 bytes, the value is
    zero-extended prior to being assigned into the register
  * Values in memory must be loaded into registers before they may be used in
    other instructions
* `store{1,2,4,8} loc, source` - stores 1, 2, 4, or 8 bytes a register's value
  into the given memory location
  * If storing 1, 2, or 4 bytes, the bytes copied from the register will be
    aligned with the least-significant bit of the register
  * That is, the lower bytes will always be copied in cases where less than 8
    bytes are requested
* `push`
* `pop`

### Control Flow

* `jmp` - unconditional jump
* `je` - jump if equal
* `jne` - jump if not equal
* `jg` - jump if greater
* `jge` - jump if greater or equal
* `jl` - jump if less
* `jle` - jump if less or equal
* `jo` - jump if overflow
* `jno` - jump if no overflow
* `jz` - jump if zero
* `jnz` - jump if not zero
* `js` - jump if signed (sign bit is set)
* `jns` - jump if not signed (sign bit is not set)
* `call loc` - pushes the value of the program counter onto the stack and then
  jumps to the given location
* `ret` - pops the value at the top of the stack and sets the program counter to it
* `nop` -
* `syscall`

### Floating Point

* TODO
