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
* System calls
* Flags: Carry, Zero, Overflow, etc.

## File layout

* file extension: `.ax` (assembly language extended)
* section 1 (optional): `.static` (case-insensitive) on its own line
  * contains static data declarations
  * the data is laid out exactly as specified, in the order specified, with no
    additional padding inserted between items of different sizes
* section 2 (required): `.code` (case-insensitive) on its own line
  * contains source code (instructions)
  * executes from top to bottom

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
.static

# no label = data at the start of the section that cannot be explicitly
# referred to in assembly code

  .b1 100  # initialize 1 byte to the value 100

ARR1:  # a labelled region of data whose location is named `ARR1`
  .zero 10  # initialize 10 bytes to zero
```

## Instruction Syntax

Used in the `.code` section.

* comment
  * `#` character to the end of the line
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
  * 64 general purpose registers: `$0`, `$1`, `$2`, etc. (up to `$63`)
  * stack pointer: `$sp` - 64-bit top address of the stack (next available slot)
  * frame pointer: `$fp` - 64-bit base address of the stack (base pointer)

Example:

```asm
.code

main:
  add $1, $2   # $1 = $1 + $2
  sub $1, 5    # $1 = $1 + 5
  ret
```

## Calling Convention

* return address is stored in register `$?` (TODO)

## Example Programs

This implements a hello world program: (filename: `hello.ax`)

```asm
TODO
```

This implements the `cat` command: (filename: `cat.ax`)

```asm
TODO
```

## Instruction Reference

Instruction names are case-insensitive.

### Key

* `dest` - destination: register or address
* `source` - operand: immediate, register or address

### Arithmetic

* `add dest, source` - add `source` and `dest` and put the result in `dest`
* `sub dest, source` - subtract `source` and `dest` and put the result in `dest`

### Control Flow

* `jz`
* `ret`
