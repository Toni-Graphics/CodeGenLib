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

pub mod x86;
pub mod opt;
pub mod error;
pub mod arch;
#[cfg(feature = "jit")]
pub mod jit;

pub use x86::*;
pub use x86::function::Function as Function;
pub use x86::builder::Builder as Builder;
pub use x86::mem::AdressManager as AdressManager;
pub use opt::optimize::Optimize as OptimizeTrait;
pub use error::Result as Result;
pub use error::CodeGenLibError as CodeGenLibError;
pub use arch::def::Arch as Arch;
pub use arch::ext::all as AllExtensions;

#[cfg(feature = "jit")]
pub use jit::typed::JitRuntime as Jit;

/// ArtifactError exportet from the faerie crate
pub use faerie::ArtifactError as ArtifactError;