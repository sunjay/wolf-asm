use thiserror::Error;

use crate::machine::Machine;
use crate::decode::*;

#[derive(Debug, Clone, Error)]
pub enum ExecuteError {
}

pub trait Execute {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError>;
}

impl Execute for Nop {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Nop {} = self;
        todo!()
    }
}
impl Execute for Add {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Add {dest, source} = self;
        todo!()
    }
}
impl Execute for Sub {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Sub {dest, source} = self;
        todo!()
    }
}
impl Execute for Mul {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Mul {dest, source} = self;
        todo!()
    }
}
impl Execute for Mull {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Mull {dest_hi, dest, source} = self;
        todo!()
    }
}
impl Execute for Mulu {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Mulu {dest, source} = self;
        todo!()
    }
}
impl Execute for Mullu {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Mullu {dest_hi, dest, source} = self;
        todo!()
    }
}
impl Execute for Div {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Div {dest, source} = self;
        todo!()
    }
}
impl Execute for Divr {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Divr {dest_rem, dest, source} = self;
        todo!()
    }
}
impl Execute for Divu {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Divu {dest, source} = self;
        todo!()
    }
}
impl Execute for Divru {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Divru {dest_rem, dest, source} = self;
        todo!()
    }
}
impl Execute for Rem {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Rem {dest, source} = self;
        todo!()
    }
}
impl Execute for Remu {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Remu {dest, source} = self;
        todo!()
    }
}
impl Execute for And {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let And {dest, source} = self;
        todo!()
    }
}
impl Execute for Or {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Or {dest, source} = self;
        todo!()
    }
}
impl Execute for Xor {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Xor {dest, source} = self;
        todo!()
    }
}
impl Execute for Test {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Test {dest, source} = self;
        todo!()
    }
}
impl Execute for Cmp {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Cmp {dest, source} = self;
        todo!()
    }
}
impl Execute for Mov {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Mov {dest, source} = self;
        todo!()
    }
}
impl Execute for Load1 {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Load1 {dest, loc} = self;
        todo!()
    }
}
impl Execute for Loadu1 {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Loadu1 {dest, loc} = self;
        todo!()
    }
}
impl Execute for Load2 {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Load2 {dest, loc} = self;
        todo!()
    }
}
impl Execute for Loadu2 {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Loadu2 {dest, loc} = self;
        todo!()
    }
}
impl Execute for Load4 {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Load4 {dest, loc} = self;
        todo!()
    }
}
impl Execute for Loadu4 {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Loadu4 {dest, loc} = self;
        todo!()
    }
}
impl Execute for Load8 {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Load8 {dest, loc} = self;
        todo!()
    }
}
impl Execute for Loadu8 {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Loadu8 {dest, loc} = self;
        todo!()
    }
}
impl Execute for Store1 {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Store1 {loc, source} = self;
        todo!()
    }
}
impl Execute for Store2 {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Store2 {loc, source} = self;
        todo!()
    }
}
impl Execute for Store4 {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Store4 {loc, source} = self;
        todo!()
    }
}
impl Execute for Store8 {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Store8 {loc, source} = self;
        todo!()
    }
}
impl Execute for Push {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Push {source} = self;
        todo!()
    }
}
impl Execute for Pop {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Pop {source} = self;
        todo!()
    }
}
impl Execute for Jmp {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Jmp {loc} = self;
        todo!()
    }
}
impl Execute for Je {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Je {loc} = self;
        todo!()
    }
}
impl Execute for Jne {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Jne {loc} = self;
        todo!()
    }
}
impl Execute for Jg {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Jg {loc} = self;
        todo!()
    }
}
impl Execute for Jge {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Jge {loc} = self;
        todo!()
    }
}
impl Execute for Ja {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Ja {loc} = self;
        todo!()
    }
}
impl Execute for Jae {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Jae {loc} = self;
        todo!()
    }
}
impl Execute for Jl {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Jl {loc} = self;
        todo!()
    }
}
impl Execute for Jle {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Jle {loc} = self;
        todo!()
    }
}
impl Execute for Jb {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Jb {loc} = self;
        todo!()
    }
}
impl Execute for Jbe {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Jbe {loc} = self;
        todo!()
    }
}
impl Execute for Jo {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Jo {loc} = self;
        todo!()
    }
}
impl Execute for Jno {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Jno {loc} = self;
        todo!()
    }
}
impl Execute for Jz {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Jz {loc} = self;
        todo!()
    }
}
impl Execute for Jnz {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Jnz {loc} = self;
        todo!()
    }
}
impl Execute for Js {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Js {loc} = self;
        todo!()
    }
}
impl Execute for Jns {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Jns {loc} = self;
        todo!()
    }
}
impl Execute for Call {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Call {loc} = self;
        todo!()
    }
}
impl Execute for Ret {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Ret {} = self;
        todo!()
    }
}
