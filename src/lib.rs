//! CodeGenLib is a rust libary to generate x86-64Bit machine code (like llvm)
//!
//! <h4>Example</h4>
//!
//! ```
//! use CodeGenLib::{Builder, ArtifactError}
//!
//! pub fn main() -> Result<(), ArtifactError> {
//!    let mut builder = Builder::new();
//!
//!    builder.add_function("call")
//!     .call("callme")
//!     .ret_int(1);
//!    
//!    builder.build("test.o")?;
//!    
//!    Ok(())
//! }
//! ```
//!
//! The examples would make a elf file with a function named call wich just calls
//! a function named callme and then returns a 1  

#![allow(non_snake_case)]

pub mod arch;
pub mod error;
pub mod ir;
#[cfg(feature = "jit")]
pub mod jit;
pub mod opt;
pub mod x86;

pub use ir::resolve;
pub use x86::builder::Builder;
pub use x86::*;

pub mod IR {
    pub use crate::x86::asm::AsmInstructionEnum::*;
    pub use crate::x86::asm::REGISTER::*;
}

pub use opt::Optimize;

//#[cfg(feature = "jit")]
//pub use jit::typed::JitRuntime as Jit;

/// BinaryFormat re-exported
pub use formatic::BinFormat;
