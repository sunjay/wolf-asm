# Wolf Assembly Language

The Wolf Assembly Language is designed for an non-existent machine and is
loosely based on conventions from x86, ARM, and MIPS assembly.

## File Layout

* file extension: `.wa` (for "Wolf Assembly")
* `section .code` (case-insensitive) on its own line
  * contains source code (instructions)
  * executes from top to bottom
* `section .static` (case-insensitive) on its own line
  * contains static data declarations
  * the data is laid out exactly as specified, in the order specified, with no
    additional padding inserted between items of different sizes
* The sections are ordered: `.code`, `.static`

## Assembler Directives

Available for use in any section.

* `.include "path/to/file.wa"` - equivalent to copying/pasting the contents of
  the specified file directly at the location of the `.include` statement.
  Relative paths are resolved relative to the directory of the file in which the
  `.include` directive is parsed. That is, if `a/b/c.wa` contains an `.include`
  directive, that directive path will be resolved relative to `a/b`.
* `.const NAME immediate` - declares a named constant that can be used in place
  of an immediate value. The immediate value will be substituted as-is for each
  instance of the name found throughout the file. The name may only be used in
  positions where an immediate would be valid.
  * Scope: The constant name will be available throughout the entire file and
    all included files, regardless of where it is declared. Multiple
    declarations of a constant name can exist as long as they have the same
    immediate value. It is a warning to redefine a constant name with a
    different immediate value.
  * Uniqueness: The constant name must be distinct from all labels declared
    anywhere in the program or in any included files.

## Static Data Declaration Syntax

Used in the `.static` section.

* label
  * an ASCII alphabetic character followed by any number of alphanumeric
    characters or underscores
  * e.g. `abc`, `L1`, `x2`
  * use `label:` to designate the address of a given section of the executable
  * labels must be unique throughout the entire program (a program may be one or
    more files joined by `.include`)
* string literal
  * single or double quoted ASCII characters, e.g. `'a'`, `"123abc\n"`
  * supports string escapes like `\n`, `\t`, `\x{FF}`, `\b{00011000}`
* `.b1`, `.b2`, `.b4`, `.b8`
  * declare and initialize 1, 2, 4, or 8 bytes to a given value
  * e.g. `.b1 3` initializes a byte to the value 3
  * negative values are initialized as two's complement values
  * the value must be an immediate value and not a label/constant name
* `.zero`
  * fills a given number of bytes with zero
  * e.g. `.zero 100` initializes 100 bytes to zero
  * the value must be an immediate value and not a label/constant name
* `.uninit`
  * declares the given number of bytes but does not initialize them
  * e.g. `.uninit 30` declares 30 uninitialized bytes
  * the value must be an immediate value and not a label/constant name
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
  * hexadecimal number: `0x1f3`
  * binary number: `0b0100_1000`
  * Underscores in literals are ignored, however the `0x` or `0b` prefix must
    not contain any `_` characters
* label
  * an ASCII alphabetic character followed by any number of alphanumeric characters
  * e.g. `abc`, `L1`, `x2`
  * use `label:` to designate the address of a given section of the executable
* register
  * 64 general purpose registers (64-bit): `$0`, `$1`, `$2`, etc. (up to `$63`)
  * `$62` and `$63` are reserved for the frame pointer and the stack pointer
    respectively
  * stack pointer: `$sp` or `$63` - 64-bit top address of the stack
    (end of next available slot)
  * frame pointer: `$fp` or `$62` - 64-bit base address of the stack
    (base pointer to the end of the current stack frame)
* register + offset
  * specified by the syntax `offset(register)`,
  * the offset is a signed, 16-bit immediate
  * e.g. `-8($sp)`, `0($1)`, `12($3)`
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

## Flags

A status register contains the current state of the processor.

| Bit # | Mask   | Abbreviation | Description                  | Category | =1                    | =0                      |
|-------|--------|--------------|------------------------------|----------|-----------------------|-------------------------|
| 0     | 0x0001 | CF           | Carry flag                   | Status   | CY (Carry)            | NC (No Carry)           |
| 1     | 0x0002 |              | Reserved                     |          |                       |                         |
| 2     | 0x0004 |              | Reserved                     |          |                       |                         |
| 3     | 0x0008 |              | Reserved                     |          |                       |                         |
| 4     | 0x0010 |              | Reserved                     |          |                       |                         |
| 5     | 0x0020 |              | Reserved                     |          |                       |                         |
| 6     | 0x0040 | ZF           | Zero flag                    | Status   | ZR (Zero)             | NZ (Not Zero)           |
| 7     | 0x0080 | SF           | Sign flag                    | Status   | NG (Negative)         | PL (Positive)           |
| 8     | 0x0100 | TF           | Trap flag (single step)      | Control  |                       |                         |
| 9     | 0x0200 | IF           | Interrupt enable flag        | Control  | EI (Enable Interrupt) | DI (Disable Interrupt)  |
| 10    | 0x0400 |              | Reserved                     |          |                       |                         |
| 11    | 0x0800 | OF           | Overflow flag                | Status   | OV (Overflow)         | NV (Not Overflow)       |

## Calling Convention

* TODO <!--return address is stored in register `$?` (TODO) -->
* pop calls should be in the opposite order to push calls

## Memory Mapped IO

Before syscalls become available, IO is done through memory-mapped IO.

* When a value is stored at address `0xffff_000c`, the lower 4-bytes
  (32-bits) are sent to standard output. The bytes are interpreted as a unicode
  scalar value. If the bytes are not valid as a unique scalar value, a
  `U+FFFD REPLACEMENT CHARACTER` (&#65533;) is outputted instead.
* Loading a 1, 2, 4, or 8 byte value from address `0xffff_0004` places the
  respective next 1, 2, 4, or 8 bytes from standard input into the destination
  register. At EOF, a value of `0` will be loaded.

### Example Programs

This implements a hello world program: (filename: `hello.wa`)

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
  store8 0xffff_000c, $10
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

This implements the `cat` command: (filename: `cat.wa`)

```asm
section .code

main:
  push $fp
  mov $fp, $sp

loop:
  # Loop through and write each received byte
  load1 $0, 0xffff_0004
  # Quit at EOF
  jz end

  # Write the character
  store1 0xffff_000c, $0

  # Continue the loop
  jmp loop

end:
  pop $fp
  ret
```

## Instruction Encoding

Instructions are 64-bits in size. The layout of the bits is determined entirely
by the opcode in the upper 12-bits of the instruction. Instructions may be given
arguments which can either be registers ($0-$63, $sp, $fp) or immediates (values
up to 64-bits). Since a 64-bit immediate cannot fit within a 64-bit instruction,
the actual maximum size of the immediate varies per opcode. Instruction opcodes
that need to fit more immediates will have less space for each. Immediates are
always encoded into the instruction using little-endian byte order.

As mentioned, the layout of an instruction is as follows:

```
==========================================
| opcode (12-bits) | arguments (52-bits) |
==========================================
```

The layout of the `arguments` portion is determined by the opcode. The supported
layouts are listed below. Each layout uses a different opcode. Not all
instructions support all layouts.

1. `register, register`
  * upper 12-bits is divided between the registers, 6-bits each
2. `register, immediate`
  * upper 6-bits is used for `register`
  * remaining 46-bits is for `immediate`
3. `immediate, register`
  * lower 6-bits is used for `register`
  * remaining 46-bits is for `immediate`
4. `register + offset, register`
  * upper 12-bits is divided between the registers, 6-bits each
  * next 16-bits is used for `offset`
5. `register + offset, immediate`
  * upper 6-bits is used for `register`
  * next 16-bits is used for `offset`
  * remaining 30-bits is for `immediate`
6. `immediate, immediate`
  * upper 26-bits for the first `immediate`
  * lower 26-bits for the second `immediate`
7. `register, register, register`
  * upper 18-bits is divided between the registers, 6-bits each
8. `register, register, immediate`
  * upper 12-bits is divided between the registers, 6-bits each
  * remaining 40-bits is for `immediate`
9. `register`
  * upper 6-bits is used to hold the value of the register
10. `immediate`
  * all 52-bits are used to hold `immediate`
11. `register + offset`
  * upper 6-bits is used to hold the value of the register
  * next 16-bits is used for `offset`

Any unused bits in an instruction are reserved. Using those bits or relying on
them to be a particular value will lead to undefined behaviour.

A zero argument instruction uses a single opcode and has a completely
unspecified layout for the `arguments` section of the instruction. The assembler
is free to choose what (if anything) to put there. Relying on any particular bit
pattern will lead to undefined behaviour.

Using an opcode/layout that is not supported by a given instruction results in
undefined behaviour.

## Instruction Reference

Instruction names are case-insensitive.

### Conventions

* `dest` - destination register
* `source` - operand immediate, label, or register
* `loc` - a register, register + offset, or an address (label or immediate)

### Arithmetic

* `add dest, source` - add `source` and `dest` and put the result in `dest`
* `sub dest, source` - subtract `source` and `dest` and put the result in `dest`
* `mul dest, source` or `mull dest_hi, dest, source` or
  `mulu dest, source` or `mullu dest_hi, dest, source`
  * `mul` - signed multiply `dest * source`, lower 64-bits of result into `dest`
  * `mulu` - unsigned multiply `dest * source`, lower 64-bits of result into `dest`
  * `mull` - signed long multiply `dest * source`, lower 64-bits of result into `dest`,
    upper 64-bits into `dest_hi`
  * `mullu` - unsigned long multiply `dest * source`, lower 64-bits of result into `dest`,
    upper 64-bits into `dest_hi`
  * specifying the same register for `dest` and `dest_hi` yields undefined behaviour
* `div dest, source` or `divr dest_rem, dest, source` or
  `divu dest, source` or `divru dest_rem, dest, source`
  * `div` signed division `dest / source`, lower 64-bits of result into `dest`
  * `divu` unsigned division `dest / source`, lower 64-bits of result into `dest`
  * `divr` signed division `dest / source` and remainder `dest % source`,
    quotient into `dest` and remainder into `dest_rem`
  * `divru` unsigned division `dest / source` and remainder `dest % source`,
    quotient into `dest` and remainder into `dest_rem`
  * specifying the same register for `dest` and `dest_rem` yields undefined behaviour
* `rem dest, source` or `remu dest, source` - divide `dest` by `source` and put
  the remainder in `dest`
  * This instruction is equivalent to `divr` or `divru` if no `dest` argument
    was passed in

### Bitwise

TODO: https://en.wikibooks.org/wiki/X86_Assembly/Shift_and_Rotate

* `shl dest, source`
* `shr dest, source`
* `sal dest, source`
* `sar dest, source`
* `rol dest, source`
* `ror dest, source`
* `rcl dest, source`
* `rcr dest, source`
* `and dest, source` - perform bitwise AND operation on `dest` and `source` and
  store the result in `dest`
* `or dest, source` - perform bitwise OR operation on `dest` and `source` and
  store the result in `dest`
* `xor dest, source` - perform bitwise XOR operation on `dest` and `source` and
  store the result in `dest`

### Comparison

* `test source1 source2` - bitwise logical and that throws away its result but
  sets the ZF (zero), SF (sign), and PF (parity) bits
* `cmp source1 source2` - comparison performed as a (signed) subtraction that
  throws away its result but sets the ZF (zero), SF (sign), PF (parity),
  CF (carry), and OF (overflow), bits

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
  * `load8` and `loadu8` have the exact same behaviour since the value is
    already 64-bit and thus does not need to be extended to assign into the
    register
  * Values in memory must be loaded into registers before they may be used in
    other instructions
* `store{1,2,4,8} loc, source` - stores 1, 2, 4, or 8 bytes a register's value
  into the given memory location
  * If storing 1, 2, or 4 bytes, the bytes copied from the register will be
    aligned with the least-significant bit of the register
  * That is, the lower bytes will always be copied in cases where less than 8
    bytes are requested
* `push source`
* `pop dest`

### Control Flow

* `jmp loc` - unconditional jump
* `je loc` - jump if equal
* `jne loc` - jump if not equal
* `jg loc` - jump if greater (signed comparison)
* `jge loc` - jump if greater or equal (signed comparison)
* `ja loc` - jump if above (unsigned comparison)
* `jae loc` - jump if above or equal (unsigned comparison)
* `jl loc` - jump if less (signed comparison)
* `jle loc` - jump if less or equal (signed comparison)
* `jb loc` - jump if below (unsigned comparison)
* `jbe loc` - jump if below or equal (unsigned comparison)
* `jo loc` - jump if overflow
* `jno loc` - jump if no overflow
* `jz loc` - jump if zero
* `jnz loc` - jump if not zero
* `js loc` - jump if signed (sign bit is set)
* `jns loc` - jump if not signed (sign bit is not set)
* `call loc` - pushes the value of the program counter onto the stack and then
  jumps to the given location
* `ret` - pops the value at the top of the stack and sets the program counter to it
* `nop` - no-op instruction (does nothing)
* `syscall`

### Floating Point

* TODO
