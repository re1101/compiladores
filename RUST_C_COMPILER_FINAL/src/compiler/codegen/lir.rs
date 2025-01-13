//! Low-level Intermediate representation as an x86-64 assembly instruction abstraction to simplify register allocation

use crate::compiler::codegen::register::*;
use crate::compiler::common::types::*;

/// INFO: Needs owned register-values so that later register transformations like type-casts don't change previous references
#[derive(Debug)]
pub enum Lir {
    // name, if needs alignment, if static decl
    GlobalDeclaration(String, bool, bool),
    // type, value
    GlobalInit(Type, StaticRegister),
    // label index, value
    StringDeclaration(usize, String),
    // label index
    LabelDefinition(usize),
    // label index
    Jmp(usize),
    // jump condition, label index
    JmpCond(&'static str, usize), // maybe encode conditions into enum

    Push(Register),
    Pop(Register),

    Call(Register),

    // Function stuff
    // usize to allocate/deallocate stack-space
    FuncSetup(String, usize, bool),
    FuncTeardown(usize),
    SaveRegs,
    RestoreRegs,
    AddSp(usize),
    SubSp(usize),

    // binary operations
    Mov(Register, Register),
    Movs(Register, Register),
    Movz(Register, Register),
    Cmp(Register, Register),
    Sub(Register, Register),
    Add(Register, Register),
    Imul(Register, Register),
    Div(Register),

    // shift direction, reg, dest
    Shift(&'static str, Register, Register),

    Load(Register, Register),

    Set(&'static str),

    // repeatedly store value
    Rep,

    // bit operations
    Xor(Register, Register),
    Or(Register, Register),
    And(Register, Register),
    Not(Register),

    // unary
    Neg(Register),
}
impl Lir {
    pub fn get_regs_mut(&mut self) -> (Option<&mut Register>, Option<&mut Register>) {
        match self {
            Lir::Call(reg) | Lir::Push(reg) | Lir::Pop(reg) => (None, Some(reg)),
            Lir::Mov(left, right)
            | Lir::Movs(left, right)
            | Lir::Movz(left, right)
            | Lir::Cmp(left, right)
            | Lir::Sub(left, right)
            | Lir::Add(left, right)
            | Lir::Imul(left, right)
            | Lir::Xor(left, right)
            | Lir::Or(left, right)
            | Lir::And(left, right)
            | Lir::Load(left, right)
            | Lir::Shift(_, left, right) => (Some(left), Some(right)),
            Lir::Neg(reg) | Lir::Not(reg) | Lir::Div(reg) => (None, Some(reg)),
            // global initializer can only have static-registers and no temporaries
            Lir::GlobalInit(..) => (None, None),
            _ => (None, None),
        }
    }
    pub fn as_string(self) -> String {
        match self {
            Lir::GlobalDeclaration(name, is_pointer, is_static) => {
                let name = maybe_prefix_underscore(&name);
                format!(
                    "\n\t.data{}\n{}{}:",
                    if !is_static {
                        format!("\n\t.globl {}", name)
                    } else {
                        String::new()
                    },
                    if is_pointer { "\t.align 4\n" } else { "" },
                    name
                )
            }
            Lir::GlobalInit(ty, reg) => {
                format!("\t.{} {}", ty.complete_suffix(), reg.name())
            }
            Lir::StringDeclaration(label_index, s) =>
            // INFO: use {:?} so escapes aren't applied:
            // "hel\nlo"
            // is .string "hel\nlo"
            // not .string "hel
            // lo"
            {
                format!("LS{}:\n\t.string {:?}", label_index, s)
            }
            Lir::LabelDefinition(label_index) => format!("L{}:", label_index),
            Lir::Jmp(label_index) => format!("\tjmp     L{}", label_index),
            Lir::JmpCond(cond, label_index) => format!("\tj{}     L{}", cond, label_index),
            Lir::FuncSetup(name, stack_size, is_static) => {
                let name = maybe_prefix_underscore(&name);
                let mut result = format!(
                    "\n\t.text\n\t{}\n{}:\n\tpushq   %rbp\n\tmovq    %rsp, %rbp\n",
                    if !is_static {
                        format!(".globl {}", name)
                    } else {
                        String::new()
                    },
                    name
                );
                // have to keep stack 16B aligned
                if stack_size > 0 {
                    let size = format!(
                        "\tsubq    ${},%rsp",
                        crate::compiler::typechecker::align_by(stack_size, 16)
                    );
                    result.push_str(&size);
                }
                result
            }
            Lir::FuncTeardown(stack_size) => match stack_size {
                0 => String::from("\tpopq    %rbp\n\tret"),
                n => format!(
                    "\taddq    ${},%rsp\n\tpopq    %rbp\n\tret",
                    crate::compiler::typechecker::align_by(n, 16)
                ),
            },
            Lir::SubSp(value) => format!("\tsubq    ${},%rsp", value),
            Lir::AddSp(value) => format!("\taddq    ${},%rsp", value),
            Lir::Push(reg) => format!("\tpushq   {}", reg.base_name()),
            Lir::Pop(reg) => format!("\tpopq    {}", reg.base_name()),
            Lir::Call(mut reg) => {
                // call uses `*%r10` and not `(%r10)`to indicate dereference
                let reg_name = if reg.is_lval() {
                    reg.set_value_kind(crate::compiler::typechecker::mir::expr::ValueKind::Rvalue);
                    format!("*{}", reg.base_name())
                } else {
                    reg.base_name()
                };

                format!("\tcall    {}", reg_name)
            }
            Lir::Mov(from, to) => format!(
                "\tmov{}    {}, {}",
                to.get_type().suffix(),
                from.name(),
                to.name()
            ),
            Lir::Movs(from, to) => format!(
                "\tmovs{}{}  {}, {}",
                from.get_type().suffix(),
                to.get_type().suffix(),
                from.name(),
                to.name()
            ),
            Lir::Movz(from, to) => format!(
                "\tmovz{}{}  {}, {}",
                from.get_type().suffix(),
                to.get_type().suffix(),
                from.name(),
                to.name()
            ),
            Lir::Cmp(left, right) => format!(
                "\tcmp{}    {}, {}",
                right.get_type().suffix(),
                left.name(),
                right.name()
            ),
            Lir::Sub(left, right) => format!(
                "\tsub{}    {}, {}",
                right.get_type().suffix(),
                left.name(),
                right.name()
            ),
            Lir::Add(left, right) => format!(
                "\tadd{}    {}, {}",
                right.get_type().suffix(),
                left.name(),
                right.name()
            ),
            Lir::Imul(left, right) => format!(
                "\timul{}   {}, {}",
                right.get_type().suffix(),
                left.name(),
                right.name()
            ),
            Lir::Div(reg) => format!(
                "\t{}\n\t{}div{}   {}",
                match reg.get_type() {
                    // zero extend %edx for unsigned division, works even for %rdx since x86-64
                    // zero-extends upper 32bits aswell
                    ty if ty.is_unsigned() => "movl\t$0, %edx",
                    // sign-extends %eax for to %edx:eax used for signed division
                    ty if ty.size() < 8 => "cdq",
                    // sign-extends %rax for to %rdx:rax
                    _ => "cqo",
                },
                if reg.get_type().is_unsigned() { "" } else { "i" },
                reg.get_type().suffix(),
                reg.name()
            ),
            Lir::Shift(direction, left, right) => format!(
                "\tsa{}{}    {}, {}",
                direction,
                right.get_type().suffix(),
                left.name(),
                right.name()
            ),
            Lir::Load(from, to) => {
                format!(
                    "\tlea{}    {}, {}",
                    to.get_type().suffix(),
                    from.name(),
                    to.name()
                )
            }
            Lir::Set(operator) => format!("\t{}   %al", operator),
            Lir::Xor(left, right) => format!(
                "\txor{}   {}, {}",
                right.get_type().suffix(),
                left.name(),
                right.name(),
            ),
            Lir::Or(left, right) => format!(
                "\tor{}     {}, {}",
                right.get_type().suffix(),
                left.name(),
                right.name(),
            ),
            Lir::And(left, right) => format!(
                "\tand{}   {}, {}",
                right.get_type().suffix(),
                left.name(),
                right.name(),
            ),
            Lir::Rep => "\trep     stosb".to_string(),
            Lir::Not(reg) => format!("\tnot{}    {}", reg.get_type().suffix(), reg.name()),
            Lir::Neg(reg) => format!("\tneg{}    {}", reg.get_type().suffix(), reg.name()),
            Lir::SaveRegs | Lir::RestoreRegs => unreachable!("will be replaced in register-allocation"),
        }
    }
}

/// macos x86-64 requires an underscore preceding labels
pub fn maybe_prefix_underscore(label: &String) -> String {
    if cfg!(target_os = "macos") {
        format!("_{}", label)
    } else {
        label.to_string()
    }
}
