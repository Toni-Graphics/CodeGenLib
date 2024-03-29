use super::asm::REGISTER;
use crate::arch::{AsmCall::AsmCall, ext::*};
use super::mem::AdressManager;
use super::var::{Variabel, VarDataType};

/// The Function class is a handler for the code generation of one function
#[derive(Clone)]   
pub struct Function<'a> {
    pub name: String,
    pub gen: Vec<Vec<u8>>,
    asm: AsmCall,
    pos: usize,

    pub esymbols: Vec<ExternSymbol>,
    pub vars: Vec<Variabel>,

    adrmng: &'a AdressManager,
}

impl<'a> Function<'a> {
    pub fn new(name: &'a str, adrmng: &'a mut AdressManager) -> Function<'a> {
        let mut asm = AsmCall {};
        let mut gen = vec![];
        gen.push( asm.endbr64() );
        gen.push( asm.push(REGISTER::RBP) );
        gen.push( asm.mov_reg(REGISTER::RBP, REGISTER::RSP) );

        Function {
            gen: gen.clone(),
            name: name.into(),
            asm: asm,
            pos: gen.len() - 1,
            esymbols: vec![],
            vars: vec![],
            adrmng: adrmng,
        }
    }

    /// Adds a return
    pub fn asm_ret(&mut self) -> &mut Self {
        self.gen.push( self.asm::nop() );
        self.gen.push( self.asm::pop(REGISTER::RBP) );
        self.gen.push( self.asm::ret() );

        self
    }

    /// Adds a assembly mov command
    pub fn asm_mov(&mut self, reg: REGISTER, value: u64) -> &mut Self {
        if  reg == REGISTER::RAX || reg == REGISTER::RBX || reg == REGISTER::RCX || reg == REGISTER::RDX || 
            reg == REGISTER::RBP || reg == REGISTER::RDI || reg == REGISTER::RIP || reg == REGISTER::RSI || 
            reg == REGISTER::RSP {
            self.gen.push( self.asm.mov_64(reg, value) );
        } else if reg == REGISTER::EAX || reg == REGISTER::EBX || reg == REGISTER::ECX || reg == REGISTER::EDX {   // 32bit Register
            self.gen.push( self.asm::mov_32(reg, value as u32) );
        } else if reg == REGISTER::AX || reg == REGISTER::BX || reg == REGISTER::DX {
            self.gen.push( self.asm::mov_16(reg, value as u16) );
        } else {
            self.gen.push( self.asm::mov_8(reg, value as u8) );
        }

        self
    }

    /// Adds a assembly mov from register to register command
    pub fn asm_mov_reg(&mut self, from: REGISTER, to: REGISTER) -> &mut Self {
        self.gen.push( self.asm.mov_reg(from, to) );

        self
    }

    /// Adds a assembly adc command
    pub fn asm_adc(&mut self, reg: REGISTER, value: u64) -> &mut Self {
        if  reg == REGISTER::RAX || reg == REGISTER::RBX || reg == REGISTER::RCX || reg == REGISTER::RDX || 
            reg == REGISTER::RBP || reg == REGISTER::RDI || reg == REGISTER::RIP || reg == REGISTER::RSI || 
            reg == REGISTER::RSP {
            self.gen.push( self.asm.adc_64(reg, value) );
        } else if reg == REGISTER::EAX || reg == REGISTER::EBX || reg == REGISTER::ECX || reg == REGISTER::EDX {   // 32bit Register
            self.gen.push( self.asm.adc_32(reg, value as u32) );
        } else if reg == REGISTER::AX || reg == REGISTER::BX || reg == REGISTER::DX {
            self.gen.push( self.asm.adc_16(reg, value as u16) );
        } else {
            self.gen.push( self.asm.adc_8(reg, value as u8) );
        }

        self
    }

    /// Adds a assembly adc command which adds the registers together
    pub fn asm_adc_reg(&mut self, dest: REGISTER, src: REGISTER) -> &mut Self {
        self.gen.push( self.asm.adc_reg(dest, src) );

        self
    }

    /// Returns an intenger
    pub fn ret_int(&mut self, value: u64) -> &mut Self {
        if value > 0xffffffff {
            self.asm_mov(REGISTER::RAX, value);
        } else {
            self.asm_mov(REGISTER::EAX, value);
        }
        self.asm_ret();

        self
    }

    /// Adds an call to the adress
    pub fn asm_call(&mut self, adr: u32) -> &mut Self {
        self.gen.push( self.asm.call(adr) );

        self
    }

    /// Calls a function with the name of `dest`
    pub fn call(&mut self, dest: &str) -> &mut Self {
        self.esymbols.push(
            ExternSymbol {
                start: self.name.clone(),
                dest: dest.into(),
                at: self.pos + 1,
            }
        );

        self.asm_call(0);

        self
    }

    /// Adds a variable to the function
    pub fn create_var(&mut self, name: &'a str, typ: VarDataType) -> Variabel {
        let var = Variabel::new(typ, &name.to_string(), self.adrmng);
        self.vars.push(var);

       self.get_last_var()
    }

    fn get_last_var(&mut self) -> Variabel {
        let list = self.vars.clone();
        self.vars.get_mut(list.len() -1)
            .expect("error while getting last function (CodeGenLib/x86/function.rs/121").to_owned()
    }

    /// Returns the generated code of the function
    pub fn get_gen(&self) -> Vec<u8> {
        let out: Vec<u8> = vec![];
        for v in self.gen {
            for b in v {
                out.push(b);
            }
        }

        out
    }
}

/// A struct for storing extern functions
#[derive(Clone)]
pub struct ExternFunction {
    pub name: String,
}

/// A struct for storing extern symbols
#[derive(Clone)]
pub struct ExternSymbol {
    pub start: String,
    pub dest: String,
    pub at: usize,
}