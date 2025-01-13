//! Operands used in codegen

use crate::compiler::common::token::TokenKind;
use crate::compiler::common::types::*;
use crate::compiler::typechecker::mir::expr::ValueKind;

use super::lir::maybe_prefix_underscore;

/// Registers used for passing arguments to functions
pub static ARG_REGS: &[[&str; 4]; 6] = &[
    ["%rdi", "%edi", "%di", "%dil"],
    ["%rsi", "%esi", "%si", "%sil"],
    ["%rdx", "%edx", "%dx", "%dl"],
    ["%rcx", "%ecx", "%cx", "%cl"],
    ["%r8", "%r8d", "%r8w", "%r8b"],
    ["%r9", "%r9d", "%r9w", "%r9b"],
];

/// All possible operands to an instruction in [LIR](crate::compiler::codegen::lir)
#[derive(Debug, Clone)]
pub enum Register {
    /// Virtual register that can be infinite in amount; get transformed into pysical registers
    /// in register-allocation pass
    Temp(TempRegister),
    /// Variables that live on the local function-stack
    Stack(StackRegister),
    /// Labels can be Strings and global variables
    Label(LabelRegister),
    /// Registers used in function calls for arguments, and in special operations
    Arg(ArgRegister),
    /// Register used for return values
    Return(Type),
    /// Numerical constants
    Literal(LiteralKind, Type),
    /// Indicator register for functions returning void
    Void,
}
impl Register {
    pub fn name(&self) -> String {
        match self {
            Register::Void => unimplemented!(),
            Register::Stack(reg) => reg.name(),
            Register::Label(reg) => reg.name(),
            Register::Literal(n, ty) => format!("${}", literal_name(n, ty)),
            Register::Temp(reg) => reg.name(),
            Register::Return(t) => t.return_reg(),
            Register::Arg(reg) => reg.name(),
        }
    }
    // name as 64bit register
    pub fn base_name(&self) -> String {
        match self {
            Register::Void | Register::Return(..) => unimplemented!(),
            Register::Stack(reg) => reg.name(),
            Register::Label(reg) => reg.base_name(),
            Register::Literal(n, ty) => literal_name(n, ty),
            Register::Temp(reg) => reg.base_name(),
            Register::Arg(reg) => reg.base_name(),
        }
    }
    pub fn set_type(&mut self, ty: Type) {
        match self {
            Register::Void | Register::Return(..) => unimplemented!(),
            Register::Label(reg) => reg.set_type(ty),
            Register::Literal(_, old_type) => *old_type = ty,
            Register::Stack(reg) => reg.ty = ty,
            Register::Temp(reg) => reg.ty = ty,
            Register::Arg(reg) => reg.ty = ty,
        }
    }
    pub fn get_type(&self) -> Type {
        match self {
            Register::Void => unimplemented!(),
            Register::Label(reg) => reg.get_type(),
            Register::Literal(_, ty) => ty.clone(),
            Register::Stack(reg) => reg.ty.clone(),
            Register::Temp(reg) => reg.ty.clone(),
            Register::Return(t) => t.clone(),
            Register::Arg(reg) => reg.ty.clone(),
        }
    }
    pub fn is_lval(&self) -> bool {
        matches!(self, Register::Temp(reg) if reg.value_kind == ValueKind::Lvalue)
    }
    pub fn set_value_kind(&mut self, new_val_kind: ValueKind) {
        if let Register::Temp(reg) = self {
            reg.value_kind = new_val_kind
        }
    }
}
#[derive(Debug, PartialEq, Clone)]
pub enum LabelRegister {
    // LS0:
    //    .string "foo"
    // label-index
    String(usize),

    //    .data
    // _varname:
    //    .zero 4
    // identifier name
    // its type with which it was declared
    // wether its address has to be retrieved from GlobalAddressTable during runtime
    Var(String, Type, bool),
}
impl LabelRegister {
    pub fn get_type(&self) -> Type {
        match self {
            LabelRegister::String(_) => {
                Type::Pointer(Box::new(QualType::new(Type::Primitive(Primitive::Char(false)))))
            }
            LabelRegister::Var(_, ty, _) => ty.clone(),
        }
    }
    fn set_type(&mut self, new_type: Type) {
        match self {
            LabelRegister::String(_) => (),
            LabelRegister::Var(_, ty, _) => *ty = new_type,
        }
    }
    fn name(&self) -> String {
        format!("{}(%rip)", self.base_name())
    }

    fn base_name(&self) -> String {
        match self {
            LabelRegister::String(index) => format!("LS{index}"),
            LabelRegister::Var(name, _, runtime_address) => {
                if *runtime_address {
                    format!("{}@GOTPCREL", maybe_prefix_underscore(name))
                } else {
                    maybe_prefix_underscore(name).to_string()
                }
            }
        }
    }
}

fn literal_name(literal: &LiteralKind, ty: &Type) -> String {
    literal.wrap(ty).to_string()
}

/// Operands that are allowed in data/bss sections
#[derive(Debug)]
pub enum StaticRegister {
    Label(LabelRegister),
    LabelOffset(LabelRegister, i64, TokenKind),
    Literal(LiteralKind, Type),
}
impl StaticRegister {
    pub fn set_type(&mut self, new: Type) {
        match self {
            StaticRegister::Label(reg) | StaticRegister::LabelOffset(reg, ..) => reg.set_type(new),
            StaticRegister::Literal(_, ty) => *ty = new,
        }
    }

    pub fn get_type(&self) -> Type {
        match self {
            StaticRegister::Label(reg) | StaticRegister::LabelOffset(reg, ..) => reg.get_type(),
            StaticRegister::Literal(_, ty) => ty.clone(),
        }
    }
    pub fn name(&self) -> String {
        match self {
            StaticRegister::Label(reg) => reg.base_name(),
            StaticRegister::LabelOffset(reg, offset, term) => format!(
                "{}{}{}",
                reg.base_name(),
                match term {
                    TokenKind::Plus => '+',
                    TokenKind::Minus => '-',
                    _ => unreachable!(),
                },
                *offset
            ),
            StaticRegister::Literal(n, ty) => literal_name(n, ty),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum StackKind {
    Signed,
    Unsigned,
}
#[derive(Debug, Clone, PartialEq)]
pub struct StackRegister {
    pub bp_offset: usize,
    kind: StackKind,
    ty: Type,
}
impl StackRegister {
    pub fn new(bp_offset: &mut usize, ty: Type) -> Self {
        *bp_offset += ty.size();
        *bp_offset = crate::compiler::codegen::align(*bp_offset, &ty);

        StackRegister {
            bp_offset: *bp_offset,
            kind: StackKind::Signed,
            ty,
        }
    }
    pub fn new_pushed(arg_index: usize) -> Self {
        assert!(arg_index >= 6);
        let arg_stack_index: usize = (arg_index as isize - ARG_REGS.len() as isize) as usize;
        const PUSHED_PARAM_OFFSET: usize = 16;
        let bp_offset =
            PUSHED_PARAM_OFFSET + arg_stack_index * Type::Primitive(Primitive::Long(true)).size();

        Self {
            bp_offset,
            kind: StackKind::Unsigned,
            ty: Type::Primitive(Primitive::Long(true)),
        }
    }
    pub fn name(&self) -> String {
        match self.kind {
            StackKind::Signed => format!("-{}(%rbp)", self.bp_offset),
            StackKind::Unsigned => format!("{}(%rbp)", self.bp_offset),
        }
    }
}

#[derive(Debug, Clone)]
pub enum TempKind {
    Scratch(Box<dyn ScratchRegister>),
    Spilled(StackRegister),
    Pushed(usize),
}

/// Virtual registers filled in by register allocation
#[derive(Debug, Clone)]
pub struct TempRegister {
    pub ty: Type,
    pub reg: Option<TempKind>,
    pub value_kind: ValueKind,
    pub start_idx: usize,

    /// Key into interval hashmap
    pub id: usize,
}
impl TempRegister {
    pub fn new(ty: Type, key_counter: &mut usize, instr_counter: usize) -> Self {
        *key_counter += 1;
        TempRegister {
            id: *key_counter,
            ty,
            reg: None,
            start_idx: instr_counter,
            value_kind: ValueKind::Rvalue,
        }
    }
    // boilerplate register that is only used to access it's base-name
    pub fn default(reg: Box<dyn ScratchRegister>) -> Self {
        TempRegister {
            ty: Type::Primitive(Primitive::Int(false)),
            id: 0,
            reg: Some(TempKind::Scratch(reg)),
            start_idx: 0,
            value_kind: ValueKind::Rvalue,
        }
    }
    fn name(&self) -> String {
        match (&self.reg, &self.value_kind) {
            (Some(TempKind::Scratch(reg)), ValueKind::Rvalue) => reg.name(&self.ty),
            (Some(TempKind::Scratch(..)), ValueKind::Lvalue) => self.base_name(),
            (Some(TempKind::Spilled(reg)), ..) => reg.name(),
            _ => unreachable!("register should always be filled by allocator"),
        }
    }
    fn base_name(&self) -> String {
        match (&self.reg, &self.value_kind) {
            // base_name for scratch-register is just it's 64bit name
            (Some(TempKind::Scratch(reg)), ValueKind::Rvalue) => reg.base_name().to_string(),
            (Some(TempKind::Scratch(reg)), ValueKind::Lvalue) => {
                format!("({})", reg.base_name())
            }
            (Some(TempKind::Spilled(reg)), ..) => reg.name(),
            _ => unreachable!(),
        }
    }
}

pub trait ScratchRegister: ScratchClone {
    fn base_name(&self) -> &'static str;
    fn name(&self, ty: &Type) -> String;
    fn is_used(&self) -> bool;
    fn in_use(&mut self);
    fn free(&mut self);
}

impl std::fmt::Debug for dyn ScratchRegister {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.base_name(), self.is_used())
    }
}

// hacky way to get clone to work on trait object
pub trait ScratchClone {
    fn clone_box(&self) -> Box<dyn ScratchRegister>;
}
impl<T> ScratchClone for T
where
    T: 'static + ScratchRegister + Clone,
{
    fn clone_box(&self) -> Box<dyn ScratchRegister> {
        Box::new(self.clone())
    }
}
impl Clone for Box<dyn ScratchRegister> {
    fn clone(&self) -> Box<dyn ScratchRegister> {
        self.clone_box()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RegularRegister {
    in_use: bool,
    base_name: &'static str,
}

impl RegularRegister {
    pub fn new(base_name: &'static str) -> Self {
        RegularRegister { in_use: false, base_name }
    }
}

impl ScratchRegister for RegularRegister {
    fn base_name(&self) -> &'static str {
        self.base_name
    }
    fn name(&self, ty: &Type) -> String {
        format!("{}{}", self.base_name, ty.reg_suffix())
    }
    fn is_used(&self) -> bool {
        self.in_use
    }
    fn in_use(&mut self) {
        self.in_use = true
    }
    fn free(&mut self) {
        self.in_use = false
    }
}
#[derive(Debug, PartialEq, Clone)]
pub struct ArgRegister {
    pub ty: Type,
    pub start_idx: usize,
    pub id: usize,
    pub reg: ArgRegisterKind,
}
impl ArgRegister {
    pub fn new(arg_index: usize, ty: Type, key_counter: &mut usize, instr_counter: usize) -> Self {
        *key_counter += 1;
        ArgRegister {
            id: *key_counter,
            start_idx: instr_counter,
            ty,
            reg: ArgRegisterKind::new(arg_index),
        }
    }
    fn name(&self) -> String {
        self.reg.name(&self.ty)
    }
    fn base_name(&self) -> String {
        self.reg.base_name().to_string()
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct ArgRegisterKind {
    in_use: bool,
    names: [&'static str; 4],
}
impl ArgRegisterKind {
    pub fn new(index: usize) -> Self {
        ArgRegisterKind { in_use: false, names: ARG_REGS[index] }
    }
}
impl ScratchRegister for ArgRegisterKind {
    fn base_name(&self) -> &'static str {
        self.names[0]
    }
    fn name(&self, ty: &Type) -> String {
        match ty {
            Type::Primitive(Primitive::Char(_)) => self.names[3],
            Type::Primitive(Primitive::Short(_)) => self.names[2],
            Type::Primitive(Primitive::Int(_)) | Type::Enum(..) => self.names[1],
            Type::Primitive(Primitive::Long(_)) | Type::Pointer(_) | Type::Array { .. } => self.names[0],
            _ => unimplemented!("aggregate types are not yet implemented as function args"),
        }
        .to_string()
    }
    fn is_used(&self) -> bool {
        self.in_use
    }
    fn in_use(&mut self) {
        self.in_use = true
    }
    fn free(&mut self) {
        self.in_use = false
    }
}
