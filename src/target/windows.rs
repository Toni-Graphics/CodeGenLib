use iced_x86::Register::*;

use super::Abi;

pub trait WindowsAbi {
    /// Returns new ABI struct with Windows ABI values
    fn windows() -> Self;
}

impl WindowsAbi for Abi {
    fn windows() -> Self {
        Abi {
            reg_args: 4, 
            regs_64: vec![RCX, RDX, R8, R9], 
            regs_32: vec![ECX, EDX, R8D, R9D], 

            return_reg: RAX,

            stack_base: 8,
        }
    }
}