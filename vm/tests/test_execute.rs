use wolf_vm::{
    decode::*,
    memory::Memory,
    registers::Registers,
    machine::{Machine, ExecutionError},
    flags::{Flags, CF::*, ZF::*, SF::*, OF::*},
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
        postconditions!($vm, reg $r:expr => $value:expr);
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
    macro_rules! test_add {
        (
            $a:literal + $b:literal == ($cty:ty) $c:literal,
            {$($flag_name:ident : $flag_value:expr),* $(,)?}
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
                    $($flag_name : $flag_value),*
                },
            }
        );
    }

    test_add!(32u64 + -32i64 == (i64) 0, {
        carry: Carry,
        zero: Zero,
        sign: PositiveSign,
        overflow: NoOverflow,
    });

    Ok(())
}
