use std::error::Error;

use iced_x86::Register;

use crate::{
    target::{Abi, Target}, error::CodeGenLibError, Builder
};

pub use super::{Type, AsmInstructionEnum::{self, *}};

/// A struct which builds a function's ir
#[derive(Debug, Clone)]
pub struct IrFunctionBuilder {
    pub generated: Vec<AsmInstructionEnum>,
    pub name: String,
    args: Vec<((String, u64, Option<Register>, Type), u64)>,
    vars: Vec<(String, i64, Type)>, // i64 -> stack offset
    funcs: Vec<(String, Vec<Type>)>,
    public: bool,

    /// for label names
    parsed_label_args: usize,

    abi: Abi,

    pub builder: Builder,
}

impl IrFunctionBuilder {
    /// Returns new `IrFunctionBuilder`
    pub fn new(name: &str, builder: &mut Builder, abi: &Abi) -> Self {
        Self {
            generated: vec![],

            name: name.into(),

            args: vec![],
            vars: vec![],
            funcs: vec![],

            public: false,

            builder: builder.to_owned(),

            abi: abi.to_owned(),

            parsed_label_args: 0,
        }
    }

    /// The input tuple values: `(String, u64)` represent:
    ///  * `String` -> The argument name
    ///  * `u64` -> The argument size in bytes
    pub fn args(&mut self, args: Vec<(&str, Type)>) {
        let mut mod_args: Vec<((String, u64, Option<Register>, Type), u64)> = vec![];

        let mut reg_pasted_args = 0;

        let mut prev_size = 0;

        for arg in args {
            let reg: Option<Register> = {
                if reg_pasted_args < self.abi.reg_args() && arg.1.in_reg() {
                    reg_pasted_args += 1;

                    if arg.1.size() == 8 { // u64/i64/str
                        Some(self.abi.arg64(reg_pasted_args - 1))
                    } else {
                        Some(self.abi.arg32(reg_pasted_args - 1))
                    }

                } else {
                    None
                }
            };

            mod_args.push(((arg.0.into(), arg.1.size(), reg, arg.1.clone()), prev_size));

            prev_size += arg.1.size();
        }

        self.args = mod_args;
    }

    /// !Needs to be called after setuped args !
    /// The input tuple values: `(String, u64)` represent:
    ///  * `String` -> The argument name
    ///  * `u64` -> The var size in bytes
    pub fn vars(&mut self, vars: Vec<(&str, Type)>) {
        let mut mod_vars: Vec<(String, i64, Type)> = vec![];
        let mut stack_args: Vec<(String, u64, Type)> = vec![];

        let mut stack_offset: i64 = 8;

        for arg in self.args.iter() {
            let name = &arg.0.0;
            let size = arg.0.1;

            if arg.0.2.is_some() {
                self.generated
                   .push(Store(arg.0.2.unwrap(), self.abi.stack(-stack_offset)));

                mod_vars.push((name.into(), -stack_offset, arg.0.3.clone()));
            } else {
               stack_args.push((name.into(), size, arg.0.3.clone()));
               mod_vars.push((name.to_string(), stack_offset, arg.0.3.clone()));
            }

            stack_offset += size as i64;
        }

        for var in stack_args {
            mod_vars.push((var.0, var.1 as i64, var.2));
        }

        for var in vars {
            mod_vars.push((var.0.into(), -stack_offset, var.1.clone()));

            stack_offset += var.1.size() as i64;
        }

        self.vars = mod_vars;
    }

    /// Sets the extern functions
    /// 
    /// Needs this tuple:
    /// ```
    /// (Name, vec![TypeOfArg1, TypeOfArg2, ...])
    /// ```
    pub fn efuncs(&mut self, funcs: Vec<(&str, Vec<Type>)>) {

        let mut mod_funcs: Vec<(String, Vec<Type>)> = vec![];

        for func in funcs {
            mod_funcs.push((func.0.into(), func.1));
        }

        self.funcs = mod_funcs;
    }

    pub fn efunc(&mut self, efunc: (String, Vec<Type>)) {
        self.funcs.push(efunc);
    }

    #[allow(dead_code)]
    fn get_arg(
        &self,
        name: String,
    ) -> Result<((String, u64, Option<Register>, Type), u64), CodeGenLibError> {
        for arg in self.args.iter() {
            let arg_1 = &arg.0;
            let arg_name = &arg_1.0;

            if arg_name.to_string() == name {
                return Ok((arg_1.clone(), arg.1));
            }
        }

        Err(CodeGenLibError::VarNotExist(name))
    }

    fn get_var(&self, name: String) -> Result<(String, i64, Type), CodeGenLibError> {
        for var in self.vars.iter() {
            if var.0 == name {
                return Ok(var.to_owned());
            }
        }

        Err(CodeGenLibError::VarNotExist(name))
    }

    /// Builds an add which does:
    /// 
    /// ```
    /// result_var = var1 + var2
    /// ```
    pub fn build_add(
        &mut self,
        var1: &str,
        var2: &str,
        result_var: &str,
    ) -> Result<(), CodeGenLibError> {
        let var1 = self.get_var(var1.into())?;
        let var2 = self.get_var(var2.into())?;
        let ret = self.get_var(result_var.into())?;


        self.generated
            .push(Load(Register::RAX, self.abi.stack(var1.1)));
        self.generated
            .push(AddMem(Register::RAX, self.abi.stack(var2.1) ));

        self.generated
            .push(Store(Register::RAX, self.abi.stack(ret.1)));

        Ok(())
    }

    /// Returns the variable with the name `var_name`
    pub fn build_return_var(&mut self, var_name: &str) -> Result<(), CodeGenLibError> {
        let var = self.get_var(var_name.into())?;

        self.generated
           .push(Load(self.abi.ret_reg(), self.abi.stack(var.1)));

        self.generated.push( Ret );

        Ok(())
    }

    /// Returns given int
    pub fn build_return_int(&mut self, int: i64) -> Result<(), CodeGenLibError> {
        self.generated
           .push(MovVal(self.abi.ret_reg(), int));

        self.generated.push( Ret );

        Ok(())
    }

    pub fn gen_x_arg_for_func(&mut self, name: &str, index: usize, ref original_arg: Type, prev_args: &Vec<Type>) -> Result<(), CodeGenLibError> {
        // prepare func
        let mut func: (String, Vec<Type>) = (String::new(), vec![]);

        for _func in self.funcs.iter() {
            if _func.0 == name {
                func = _func.to_owned();
            }
        }
        if func.0 == String::new() { // Still uninitalised
            return Err(CodeGenLibError::FuncNotExist(name.into()));
        }

        let mut used_regs = 0;

        let mut arg = original_arg.clone();

        for mut _arg in prev_args.to_owned() {

            if _arg.empty() == Type::InVar(String::new()) {      
                let var = self.get_var(
                    match arg {
                        Type::InVar(x) => x.to_owned(),
                        _ => "".into(),
                    }
                )?;

                _arg = var.2.clone();
                arg = var.2;
            }

            if _arg.in_reg() {
                used_regs += 1;
            }
        }

        if arg.empty() == Type::InVar(String::new()) {
            println!("{:?}", original_arg);
            let var = self.get_var(
                match original_arg {
                    Type::InVar(x) => x.to_owned(),
                    _ => "".into(),
                }
            )?;

            match var.2 {
                Type::u64(_) => { 
                    if arg.in_reg() && used_regs <= self.abi.reg_args() {
                        self.generated.push(Store(self.abi.arg64(used_regs), self.abi.stack(var.1)));
                    } else {
                        self.generated.push(Load(Register::RAX, self.abi.stack(var.1)));
                        self.generated.push(Push(Register::RAX));
                    }
                },
                Type::u32(_) => { 
                    if arg.in_reg() && used_regs <= self.abi.reg_args() {
                        self.generated.push(Store(self.abi.arg32(used_regs), self.abi.stack(var.1)));
                    } else {
                        self.generated.push(Load(Register::RAX, self.abi.stack(var.1)));
                        self.generated.push(Push(Register::RAX));
                    }
                },
                Type::i64(_) => { 
                    if arg.in_reg() && used_regs <= self.abi.reg_args() {
                        self.generated.push(Store(self.abi.arg64(used_regs), self.abi.stack(var.1)));
                    } else {
                        self.generated.push(Load(Register::RAX, self.abi.stack(var.1)));
                        self.generated.push(Push(Register::RAX));
                    }
                },
                Type::i32(_) => { 
                    if arg.in_reg() && used_regs <= self.abi.reg_args() {
                        self.generated.push(Store(self.abi.arg32(used_regs), self.abi.stack(var.1)));
                    } else {
                        self.generated.push(Load(Register::RAX, self.abi.stack(var.1)));
                        self.generated.push(Push(Register::RAX));
                    }
                },
                Type::Bytes(_) => todo!(),
                Type::Str(_) => todo!(),
                Type::Ptr(_) => todo!(),
                _ => {},
            }
        } else if used_regs <= self.abi.reg_args() && arg.in_reg() {
            match arg {
                Type::u32(val) =>   {self.generated.push(MovVal(self.abi.arg32(used_regs), val as i64)); },
                Type::i32(val) =>   {self.generated.push(MovVal(self.abi.arg32(used_regs), val as i64)); },
                Type::u64(val) =>   {self.generated.push(MovVal(self.abi.arg64(used_regs), val as i64)); },
                Type::i64(val) =>   {self.generated.push(MovVal(self.abi.arg64(used_regs), val as i64)); },
                Type::Str(content) => {
                    let label_name = format!("{}.{}", self.name, self.parsed_label_args);

                    self.parsed_label_args += 1;

                    self.builder.define_label(&label_name, false, content);

                    self.generated.push(MovPtr(self.abi.arg64(index), label_name));
                },
                Type::Ptr(content) => {
                    let label_name = format!("{}.{}", self.name, self.parsed_label_args);

                    self.parsed_label_args += 1;

                    self.builder.define_label(&label_name, false, content.bytes());

                    self.generated.push(MovPtr(self.abi.arg64(index), label_name));
                },
                _ => {},
            };

        } else {
            match arg {
                Type::Str(content) => {
                    let label_name = format!("{}.{}",self.name, self.parsed_label_args);

                    self.parsed_label_args += 1;

                    self.builder.define_label(&label_name, false, content);

                    self.generated.push(PushPtr(label_name));
                },
                Type::Ptr(content) => {
                    let label_name = format!("{}.{}", self.name, self.parsed_label_args);
                    
                    self.parsed_label_args += 1;

                    self.builder.define_label(&label_name, false, content.bytes());

                    self.generated.push(PushPtr(label_name));
                },

                Type::u64(val) => { self.generated.push(PushVal(val as i64)) },
                Type::i64(val) => { self.generated.push(PushVal(val as i64)) },
                Type::u32(val) => { self.generated.push(PushVal(val as i64)) },
                Type::i32(val) => { self.generated.push(PushVal(val as i64)) },

                arg => {
                    let label_name = format!("{}.{}",self.name, self.parsed_label_args);
                    
                    self.parsed_label_args += 1;

                    self.builder.define_label(&label_name, false, arg.bytes());
        
                    self.generated.push(PushLabel(
                        label_name
                    ));
                }
            }
        }

        Ok(())
    }

    /// Calls function with name `func` and args `args`
    /// 
    /// **!** func needs to be definied via the efuncs-function else there will be sus errors
    /// 
    /// Example:
    /// ```
    /// func.build_call("printf", vec![Type::Str(b"Hello World!".into())])?;
    /// ```
    pub fn build_call(&mut self, func: &str, args: Vec<Type>) -> Result<(), Box<dyn Error>> {
        let mut index = 0;

        let mut prev_args = vec![];

        for arg in args {
            self.gen_x_arg_for_func(func, index, arg.clone(), &prev_args)?;

            prev_args.push( arg );

            println!("{:?}", prev_args);

            index += 1;
        } 

        self.generated.push(Call(func.into()));

        Ok(())
    }

    pub fn build_set(&mut self, name: &str, content: Type) -> Result<(), Box<dyn Error>> {

        let var = self.get_var(name.into())?;

        match content {
            Type::u64(val) => { 
                self.generated.push(MovVal(Register::RAX, val as i64));
                self.generated.push(Store(Register::RAX, self.abi.stack(var.1)));
            },
            Type::u32(val) => { 
                self.generated.push(MovVal(Register::EAX, val as i64));
                self.generated.push(Store(Register::EAX, self.abi.stack(var.1)));
            },
            Type::i64(val) => { 
                self.generated.push(MovVal(Register::RAX, val as i64));
                self.generated.push(Store(Register::RAX, self.abi.stack(var.1)));
            },
            Type::i32(val) => { 
                self.generated.push(MovVal(Register::EAX, val as i64));
                self.generated.push(Store(Register::EAX, self.abi.stack(var.1)));
            },
            Type::Bytes(_) => {},
            Type::Str(_) => {},
            Type::Ptr(_adr) => {},
            Type::InVar(_) => {},
            Type::Unlim(_) => {},
        }


        Ok(())
    }
    
    /// Sets the function public
    pub fn set_public(&mut self) {
        self.public = true;
    }
}

/// Builder which handels `IrFunctionBuilders`
pub struct IrBuilder {
    functs: Vec<IrFunctionBuilder>,
    pub build: Builder,

    abi: Target,
}

impl IrBuilder {
    /// Returns new `IrFunctionBuilder` with the `target` Abi
    pub fn new(target: Target) -> Self {
        Self { 
            functs: vec![], 
            build: Builder::new(),
            abi: target,
        }
    }

    /// Adds new function with name `name` and returns mutable reference
    pub fn add(&mut self, name: &str) -> &mut IrFunctionBuilder {
        self.functs.push(
            IrFunctionBuilder::new(name, &mut self.build, &self.abi.abi)
        );

        self.functs.last_mut().unwrap()
    }

    /// Writes all functions/data etc. into outfile with path `outpath`
    pub fn write(&mut self, outpath: &str) -> Result<(), Box<dyn std::error::Error>> {
        for func in self.functs.iter() {
            let func = func.to_owned();

            let mut code = func.generated;

            if code.last() != Some(&Ret) {
                code.push( MovVal(self.abi.abi.ret_reg(), 0) ); // return 0;
                code.push( Ret );
            }

            self.build.sync(&func.builder);

            self.build.define(&func.name, func.public, code)?;
        }

        self.build.write(outpath, self.abi.bin)
    }
}
