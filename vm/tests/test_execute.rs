use wolf_vm::{
    decode::*,
    memory::Memory,
    registers::Registers,
    machine::{Machine, ExecutionError},
    flags::{Flags, CF::*, ZF::*, SF::*, OF::*},
    io::Stdio,
    execute::Execute,
};
use wolf_asm::{
    asm::{self, layout::Reg},
};

const TEST_MEMORY: usize = 1024; // 1 kB

pub fn r(reg: u8) -> Reg {
    assert!(reg < asm::REGISTERS);
    asm::RegisterKind::Numbered(reg).into()
}

pub fn sp() -> Reg {
    asm::RegisterKind::StackPointer.into()
}

pub fn fp() -> Reg {
    asm::RegisterKind::FramePointer.into()
}

macro_rules! postconditions {
    ($vm:ident, reg $r:expr => ($rty:ty) $value:expr $(,)?) => (
        let value: $rty = $vm.registers.load($r);
        assert_eq!(value, $value, "Post condition `{}` failed", stringify!(reg $r => $value));
    );
    ($vm:ident, reg $r:expr => ($rty:ty) $value:expr, $($rem:tt)*) => (
        postconditions!($vm, reg $r => ($rty) $value);
        postconditions!($vm, $($rem)*);
    );
    ($vm:ident, flag $f:ident => $value:expr $(,)?) => (
        assert_eq!($vm.flags.$f, $value, "Post condition `{}` failed", stringify!(flag $f => $value));
    );
    ($vm:ident, flag $f:ident => $value:expr, $($rem:tt)*) => (
        postconditions!($vm, flag $f => $value);
        postconditions!($vm, $($rem)*);
    );
}

macro_rules! execute {
    (
        program: [
            $($instr:ident {
                $($field_name:ident : $field_value:expr),* $(,)?
            }),* $(,)?
        ],
        $(postconditions: [
            $($postcond:tt)*
        ],)?
        $(flags: {
            $($flag_name:ident : $flag_value:expr),* $(,)?
        },)?
    ) => {
        let mut vm = Machine {
            program_counter: 0,
            memory: Memory::new(TEST_MEMORY),
            registers: Registers::new(TEST_MEMORY),
            flags: Flags::default(),
            io: Stdio::default(),
        };

        $(
            let instr = $instr {
                $($field_name : $field_value.into()),*
            };
            instr.execute(&mut vm)?;
        )*

        $(postconditions!(vm, $($postcond)*);)?

        $(assert_eq!(vm.flags, Flags {$($flag_name : $flag_value),*}))?
    };
}

#[test]
fn add_flags() -> Result<(), ExecutionError> {
    macro_rules! add {
        (
            $a:literal + $b:literal == ($cty:ty) $c:expr,
            {$carry:ident, $zero:ident, $sign:ident, $overflow:ident$(,)?}
        ) => (
            execute! {
                program: [
                    Mov {dest: r(0), source: $a},
                    Add {dest: r(0), source: $b},
                ],
                postconditions: [
                    reg r(0) => ($cty) $c,
                ],
                flags: {
                    carry: $carry,
                    zero: $zero,
                    sign: $sign,
                    overflow: $overflow,
                },
            }
        );
    }

    add!(32u64 + -32i64 == (u64) 0, {Carry, Zero, PositiveSign, NoOverflow});
    add!(32u64 + -34i64 == (i64) -2, {NoCarry, NonZero, NegativeSign, NoOverflow});
    add!(33u64 + 27u64 == (u64) 60, {NoCarry, NonZero, PositiveSign, NoOverflow});
    // 0xffffffffffffffff == u64::MAX
    add!(0xffffffffffffffffu64 + 1u64 == (u64) 0, {Carry, Zero, PositiveSign, NoOverflow});
    // 0x7fffffffffffffff == i64::MAX
    add!(0x7fffffffffffffffi64 + 1u64 == (i64) i64::MIN, {NoCarry, NonZero, NegativeSign, Overflow});

    Ok(())
}

#[test]
fn sub_flags() -> Result<(), ExecutionError> {
    macro_rules! sub {
        (
            $a:literal - $b:literal == ($cty:ty) $c:expr,
            {$carry:ident, $zero:ident, $sign:ident, $overflow:ident$(,)?}
        ) => (
            execute! {
                program: [
                    Mov {dest: r(0), source: $a},
                    Sub {dest: r(0), source: $b},
                ],
                postconditions: [
                    reg r(0) => ($cty) $c,
                ],
                flags: {
                    carry: $carry,
                    zero: $zero,
                    sign: $sign,
                    overflow: $overflow,
                },
            }
        );
    }

    sub!(32u64 - 32u64 == (u64) 0, {NoCarry, Zero, PositiveSign, NoOverflow});
    sub!(32u64 - 34u64 == (i64) -2, {Carry, NonZero, NegativeSign, NoOverflow});
    sub!(33u64 - -27i64 == (u64) 60, {Carry, NonZero, PositiveSign, NoOverflow});
    // 0xffffffffffffffff == u64::MAX
    sub!(0xffffffffffffffffu64 - -1i64 == (u64) 0, {NoCarry, Zero, PositiveSign, NoOverflow});
    // 0x7fffffffffffffff == i64::MAX
    sub!(0x7fffffffffffffffi64 - -1i64 == (i64) i64::MIN, {Carry, NonZero, NegativeSign, Overflow});

    Ok(())
}

#[test]
fn cmp_flags() -> Result<(), ExecutionError> {
    macro_rules! cmp {
        (
            ($aty:ty) $a:literal < $b:literal,
            {$carry:ident, $zero:ident, $sign:ident, $overflow:ident$(,)?}
        ) => (
            execute! {
                program: [
                    Mov {dest: r(0), source: $a},
                    Cmp {source1: r(0), source2: $b},
                ],
                postconditions: [
                    // register shouldn't change
                    reg r(0) => ($aty) $a,
                ],
                flags: {
                    carry: $carry,
                    zero: $zero,
                    sign: $sign,
                    overflow: $overflow,
                },
            }
        );
    }

    cmp!((u64) 32u64 < 32u64, {NoCarry, Zero, PositiveSign, NoOverflow});
    cmp!((u64) 32u64 < 34u64, {Carry, NonZero, NegativeSign, NoOverflow});
    cmp!((u64) 33u64 < -27i64, {Carry, NonZero, PositiveSign, NoOverflow});
    // 0xffffffffffffffff == u64::MAX
    cmp!((u64) 0xffffffffffffffffu64 < -1i64, {NoCarry, Zero, PositiveSign, NoOverflow});
    // 0x7fffffffffffffff == i64::MAX
    cmp!((i64) 0x7fffffffffffffffi64 < -1i64, {Carry, NonZero, NegativeSign, Overflow});

    Ok(())
}

#[test]
fn mul_flags() -> Result<(), ExecutionError> {
    macro_rules! mul {
        (
            $a:literal * $b:literal == ($cty:ty) $c:expr,
            {$carry:ident, $overflow:ident$(,)?}
        ) => (
            execute! {
                program: [
                    Mov {dest: r(0), source: $a},
                    Mul {dest: r(0), source: $b},
                ],
                postconditions: [
                    reg r(0) => ($cty) $c,
                    flag carry => $carry,
                    flag overflow => $overflow,
                ],
            }
        );
    }

    mul!(32u64 * 0u64 == (u64) 0, {NoCarry, NoOverflow});
    mul!(32u64 * 1u64 == (u64) 32, {NoCarry, NoOverflow});
    mul!(32u64 * -1i64 == (i64) -32, {NoCarry, NoOverflow});

    mul!(32u64 * -34i64 == (i64) -1088, {NoCarry, NoOverflow});
    mul!(33u64 * 27u64 == (u64) 891, {NoCarry, NoOverflow});

    mul!(0x1fffffffffffffffi64 * 0x1fffffffffffffffi64 == (i64) -4611686018427387903, {Carry, Overflow});
    // 0x7fffffffffffffff == i64::MAX
    mul!(0x7fffffffffffffffi64 * 0x7fffffffffffffffi64 == (i64) 1, {Carry, Overflow});

    Ok(())
}

#[test]
fn mulu_flags() -> Result<(), ExecutionError> {
    macro_rules! mulu {
        (
            $a:literal * $b:literal == ($cty:ty) $c:expr,
            {$carry:ident, $overflow:ident$(,)?}
        ) => (
            execute! {
                program: [
                    Mov {dest: r(0), source: $a},
                    Mulu {dest: r(0), source: $b},
                ],
                postconditions: [
                    reg r(0) => ($cty) $c,
                    flag carry => $carry,
                    flag overflow => $overflow,
                ],
            }
        );
    }

    mulu!(32u64 * 0u64 == (u64) 0, {NoCarry, NoOverflow});
    mulu!(32u64 * 1u64 == (u64) 32, {NoCarry, NoOverflow});

    mulu!(33u64 * 27u64 == (u64) 891, {NoCarry, NoOverflow});

    // 0xffffffffffffffff == u64::MAX
    mulu!(0x1fffffffffffffffu64 * 0x1fffffffffffffffu64 == (u64) 0xc000000000000001, {Carry, Overflow});
    // 0xffffffffffffffff == u64::MAX
    mulu!(0xffffffffffffffffu64 * 0xffffffffffffffffu64 == (u64) 1, {Carry, Overflow});
    // 0x7fffffffffffffff == i64::MAX
    mulu!(0x7fffffffffffffffi64 * 0x7fffffffffffffffi64 == (i64) 1, {Carry, Overflow});

    Ok(())
}
