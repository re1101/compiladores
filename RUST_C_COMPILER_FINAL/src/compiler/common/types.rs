pub use struct_ref::StructRef;

use crate::compiler::common::{error::*, token::*};
use crate::compiler::parser::hir;
use crate::compiler::typechecker::mir;

use std::fmt::Display;
use std::rc::Rc;

static RETURN_REG: &[&str; 4] = &["%al", "%ax", "%eax", "%rax"];

/// A fully qualified type is made of a type and its qualifiers
#[derive(Clone, PartialEq, Debug)]
pub struct QualType {
    pub ty: Type,
    pub qualifiers: Qualifiers,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Qualifiers {
    pub is_const: bool,
    pub is_volatile: bool,
    pub is_restrict: bool,
}
impl Qualifiers {
    pub fn default() -> Qualifiers {
        Qualifiers {
            is_const: false,
            is_volatile: false,
            is_restrict: false,
        }
    }
    // self contains all and maybe more qualifiers than other
    fn contains_all(&self, other: &Qualifiers) -> bool {
        self.is_const >= other.is_const
            && self.is_volatile >= other.is_volatile
            && self.is_restrict >= other.is_restrict
    }
    pub fn is_empty(&self) -> bool {
        self == &Qualifiers::default()
    }
}
impl From<&Vec<hir::decl::Qualifier>> for Qualifiers {
    fn from(qualifiers: &Vec<hir::decl::Qualifier>) -> Self {
        let qualifiers: Vec<_> = qualifiers.into_iter().map(|q| &q.kind).collect();
        Qualifiers {
            is_const: qualifiers.contains(&&hir::decl::QualifierKind::Const),
            is_volatile: qualifiers.contains(&&hir::decl::QualifierKind::Volatile),
            is_restrict: qualifiers.contains(&&hir::decl::QualifierKind::Restrict),
        }
    }
}

/// All C-types in currently implemented in `wrecc`
#[derive(Clone, PartialEq, Debug)]
pub enum Type {
    Primitive(Primitive),
    Array(Box<QualType>, ArraySize),
    Pointer(Box<QualType>),
    Struct(StructKind),
    Union(StructKind),
    Enum(Option<String>, Vec<(Token, i32)>),
    Function(FuncType),
}

#[derive(Clone, Debug)]
pub enum ArraySize {
    Known(usize),
    Unknown,
}
impl PartialEq for ArraySize {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (ArraySize::Known(size1), ArraySize::Known(size2)) => size1 == size2,
            (ArraySize::Unknown, ArraySize::Unknown)
            | (ArraySize::Known(_), ArraySize::Unknown)
            | (ArraySize::Unknown, ArraySize::Known(_)) => true,
        }
    }
}
impl QualType {
    pub fn new(ty: Type) -> QualType {
        QualType { ty, qualifiers: Qualifiers::default() }
    }

    pub fn type_compatible(&self, other: &mir::expr::Expr) -> bool {
        match (&self.ty, &other.qtype.ty) {
            (Type::Primitive(Primitive::Void), Type::Primitive(Primitive::Void)) => true,

            (Type::Primitive(Primitive::Void), Type::Primitive(_) | Type::Enum(..))
            | (Type::Primitive(_) | Type::Enum(..), Type::Primitive(Primitive::Void)) => false,

            (Type::Primitive(_) | Type::Enum(..), Type::Primitive(_) | Type::Enum(..)) => true,

            // pointer to null-pointer-constant is always valid
            (Type::Pointer(_), _) if other.is_zero() => true,

            // void* is compatible to any other pointer
            (Type::Pointer(t), Type::Pointer(_)) | (Type::Pointer(_), Type::Pointer(t))
                if matches!(t.ty, Type::Primitive(Primitive::Void)) =>
            {
                true
            }

            // 6.5.16.1 both operands are pointers to qualified or unqualified versions of compatible types,
            // and the type pointed to by the left has all the qualifiers of the type pointed to by the right
            (Type::Pointer(inner1), Type::Pointer(inner2)) => {
                inner1.qualifiers.contains_all(&inner2.qualifiers)
                    && if let (Type::Pointer(nested1), Type::Pointer(nested2)) = (&inner1.ty, &inner2.ty)
                    {
                        nested1 == nested2
                    } else {
                        inner1.ty == inner2.ty
                    }
            }

            // unspecified arrays are compatible if they have the same type and their sizes are
            // compatible (see PartialEq for ArraySize)
            (Type::Array(..), Type::Array(..)) => self.ty == other.qtype.ty,

            // two structs/unions are compatible if they refer to the same definition
            (Type::Struct(s_l), Type::Struct(s_r)) | (Type::Union(s_l), Type::Union(s_r)) => s_l == s_r,

            // func is compatible to func if they have the exact same signature
            (Type::Function(f1), Type::Function(f2)) => f1 == f2,

            _ => false,
        }
    }
    pub fn pointer_to(self) -> QualType {
        QualType {
            qualifiers: Qualifiers::default(),
            ty: Type::Pointer(Box::new(self.clone())),
        }
    }
    pub fn function_of(self, params: Vec<QualType>, variadic: bool) -> QualType {
        // functions cannot have qualifiers since they only describe the return type
        QualType::new(Type::Function(FuncType {
            return_type: Box::new(self),
            params,
            variadic,
        }))
    }
    pub fn deref_at(&self) -> Option<QualType> {
        match &self.ty {
            Type::Pointer(inner) => Some(*inner.clone()),
            _ => None,
        }
    }
    pub fn unqualified(&self) -> QualType {
        if self.qualifiers.is_empty() {
            self.clone()
        } else {
            QualType {
                qualifiers: Qualifiers::default(),
                ..self.clone()
            }
        }
    }
}

/// Trait all types have to implement to work in codegen
pub trait TypeInfo {
    /// Returns size of type in bytes
    fn size(&self) -> usize;

    /// Returns the correct suffix for a register of type
    fn reg_suffix(&self) -> String;

    /// Returns the instruction-suffixes
    fn suffix(&self) -> String;

    /// Returns the instruction-suffixes spelled out
    fn complete_suffix(&self) -> String;

    /// Returns the return register name of type
    fn return_reg(&self) -> String;
}
impl TypeInfo for Type {
    fn size(&self) -> usize {
        match self {
            Type::Primitive(t) => t.size(),
            Type::Struct(s) => s.members().iter().fold(0, |acc, (t, _)| acc + t.ty.size()),
            Type::Union(_) => self.union_biggest().ty.size(),
            Type::Pointer(_) => Type::Primitive(Primitive::Long(true)).size(),
            Type::Enum(..) => Type::Primitive(Primitive::Int(false)).size(),
            Type::Array(element_type, ArraySize::Known(amount)) => amount * element_type.ty.size(),
            // INFO: tentative array assumed to have one element
            Type::Array(element_type, ArraySize::Unknown) => element_type.ty.size(),
            Type::Function(_) => 1,
        }
    }
    fn reg_suffix(&self) -> String {
        match self {
            Type::Primitive(t) => t.reg_suffix(),
            Type::Union(_) => self.union_biggest().ty.reg_suffix(),
            Type::Enum(..) => Type::Primitive(Primitive::Int(false)).reg_suffix(),
            Type::Pointer(_) | Type::Array { .. } | Type::Struct(..) => {
                Type::Primitive(Primitive::Long(true)).reg_suffix()
            }
            Type::Function { .. } => unreachable!("no plain function type used"),
        }
    }
    fn suffix(&self) -> String {
        match self {
            Type::Primitive(t) => t.suffix(),
            Type::Union(_) => self.union_biggest().ty.suffix(),
            Type::Enum(..) => Type::Primitive(Primitive::Int(false)).suffix(),
            Type::Pointer(_) | Type::Array { .. } | Type::Struct(..) => {
                Type::Primitive(Primitive::Long(true)).suffix()
            }
            Type::Function { .. } => unreachable!("no plain function type used"),
        }
    }
    fn complete_suffix(&self) -> String {
        match self {
            Type::Primitive(t) => t.complete_suffix(),
            Type::Union(_) => self.union_biggest().ty.complete_suffix(),
            Type::Enum(..) => Type::Primitive(Primitive::Int(false)).complete_suffix(),
            Type::Pointer(_) | Type::Array { .. } | Type::Struct(..) => {
                Type::Primitive(Primitive::Long(true)).complete_suffix()
            }
            Type::Function { .. } => unreachable!("no plain function type used"),
        }
    }
    fn return_reg(&self) -> String {
        match self {
            Type::Primitive(t) => t.return_reg(),
            Type::Pointer(_) | Type::Array { .. } => Type::Primitive(Primitive::Long(true)).return_reg(),
            Type::Enum(..) => Type::Primitive(Primitive::Int(false)).return_reg(),
            Type::Union(..) => self.union_biggest().ty.return_reg(),
            Type::Struct(..) => unimplemented!("currently can't return structs"),
            Type::Function { .. } => unreachable!("no plain function type used"),
        }
    }
}

impl Type {
    pub fn is_void(&self) -> bool {
        *self == Type::Primitive(Primitive::Void)
    }
    pub fn is_func(&self) -> bool {
        matches!(self, Type::Function { .. })
    }
    pub fn is_unbounded_array(&self) -> bool {
        matches!(self, Type::Array(_, ArraySize::Unknown))
    }
    pub fn is_array(&self) -> bool {
        matches!(self, Type::Array { .. })
    }
    pub fn is_ptr(&self) -> bool {
        matches!(self, Type::Pointer(_))
    }
    pub fn is_scalar(&self) -> bool {
        match self {
            Type::Primitive(Primitive::Void) => false,
            Type::Primitive(_) | Type::Pointer(_) | Type::Enum(..) => true,
            _ => false,
        }
    }
    pub fn is_integer(&self) -> bool {
        match self {
            Type::Primitive(Primitive::Void) => false,
            Type::Primitive(_) | Type::Enum(..) => true,
            _ => false,
        }
    }
    pub fn is_struct(&self) -> bool {
        matches!(self, Type::Struct(_) | Type::Union(_))
    }
    pub fn is_aggregate(&self) -> bool {
        matches!(self, Type::Struct(_) | Type::Union(_) | Type::Array(..))
    }
    fn union_biggest(&self) -> QualType {
        match self {
            Type::Union(s) => s
                .members()
                .iter()
                .max_by_key(|(qtype, _)| qtype.ty.size())
                .expect("union can't be empty, checked in parser")
                .0
                .clone(),
            _ => unreachable!("not union"),
        }
    }
    pub fn is_complete(&self) -> bool {
        match self {
            Type::Struct(s) | Type::Union(s) => s.is_complete(),
            Type::Array(of, ArraySize::Known(_)) => of.ty.is_complete(),
            Type::Array(_, ArraySize::Unknown) => false,
            _ if self.is_void() => false,
            _ => true,
        }
    }

    pub fn max(&self) -> u64 {
        match self {
            Type::Primitive(t) => t.max(),
            Type::Enum(..) => i32::MAX as u64,
            Type::Pointer(_) => u64::MAX,
            _ => unreachable!(),
        }
    }
    pub fn min(&self) -> i64 {
        match self {
            Type::Primitive(t) => t.min(),
            Type::Enum(..) => i32::MIN as i64,
            Type::Pointer(_) => u64::MIN as i64,
            _ => unreachable!(),
        }
    }

    pub fn get_primitive(&self) -> Option<&Primitive> {
        if let Type::Primitive(prim) = self {
            Some(prim)
        } else {
            None
        }
    }
    pub fn is_char_array(&self) -> Option<&ArraySize> {
        if let Type::Array(of, size) = self {
            if let Type::Primitive(Primitive::Char(_)) = of.ty {
                return Some(size);
            }
        }
        None
    }
    pub fn is_unsigned(&self) -> bool {
        match self {
            Type::Primitive(prim) => prim.is_unsigned(),
            Type::Pointer(_) => true,
            _ => false,
        }
    }
}

#[derive(Clone, Debug)]
pub struct FuncType {
    pub return_type: Box<QualType>,
    pub params: Vec<QualType>,
    pub variadic: bool,
}
impl PartialEq for FuncType {
    fn eq(&self, other: &Self) -> bool {
        self.return_type == other.return_type
            && self.variadic == other.variadic
            && self.params.len() == other.params.len()
            && self
                .params
                .iter()
                .zip(&other.params)
                .all(|(p1, p2)| p1.unqualified() == p2.unqualified())
    }
}
impl FuncType {
    pub fn check_main_signature(&self, token: &Token, is_inline: bool) -> Result<(), Error> {
        if self.return_type.ty != Type::Primitive(Primitive::Int(false)) {
            return Err(Error::new(
                token,
                ErrorKind::InvalidMainReturn(*self.return_type.clone()),
            ));
        }
        if is_inline {
            return Err(Error::new(
                token,
                ErrorKind::Regular("'main' function cannot be declared 'inline'"),
            ));
        }
        if self.variadic {
            return Err(Error::new(
                token,
                ErrorKind::Regular("'main' function cannot be declared variadic"),
            ));
        }
        let unqualified_params = self.params.iter().map(|qtype| &qtype.ty).collect::<Vec<_>>();

        match unqualified_params.as_slice() {
            [] => (),
            [Type::Primitive(Primitive::Int(false)), Type::Pointer(to)] => match &to.ty {
                Type::Pointer(nested_to) if nested_to.ty == Type::Primitive(Primitive::Char(false)) => {
                    ()
                }
                _ => {
                    return Err(Error::new(
                        token,
                        ErrorKind::Regular("second parameter of 'main' must be of type 'char **'"),
                    ))
                }
            },
            _ => {
                return Err(Error::new(
                    token,
                    ErrorKind::Regular("invalid parameters to 'main' function"),
                ))
            }
        }

        Ok(())
    }
}

#[derive(Clone, PartialEq, PartialOrd, Debug)]
pub enum Primitive {
    Void,
    // true if unsigned
    Char(bool),
    Short(bool),
    Int(bool),
    Long(bool),
}

impl TypeInfo for Primitive {
    /// Returns type-size in bytes
    fn size(&self) -> usize {
        match self {
            Primitive::Void => 0,
            Primitive::Char(_) => 1,
            Primitive::Short(_) => 2,
            Primitive::Int(_) => 4,
            Primitive::Long(_) => 8,
        }
    }
    fn reg_suffix(&self) -> String {
        String::from(match self {
            Primitive::Void => unreachable!(),
            Primitive::Char(_) => "b",
            Primitive::Short(_) => "w",
            Primitive::Int(_) => "d",
            Primitive::Long(_) => "",
        })
    }
    fn suffix(&self) -> String {
        self.complete_suffix().get(0..1).unwrap().to_string()
    }
    fn complete_suffix(&self) -> String {
        String::from(match self {
            Primitive::Void => "zero",
            Primitive::Char(_) => "byte",
            Primitive::Short(_) => "word",
            Primitive::Int(_) => "long",
            Primitive::Long(_) => "quad",
        })
    }
    fn return_reg(&self) -> String {
        String::from(match self {
            Primitive::Void => unreachable!("doesnt have return register when returning void"),
            Primitive::Char(_) => RETURN_REG[0],
            Primitive::Short(_) => RETURN_REG[1],
            Primitive::Int(_) => RETURN_REG[2],
            Primitive::Long(_) => RETURN_REG[3],
        })
    }
}
impl Primitive {
    /// 6.4.4.1.5 Determines smallest possible integer-type capable of holding literal number
    pub fn new(n: u64, radix: Radix, suffix: Option<IntSuffix>) -> Primitive {
        if let Some(IntSuffix::U | IntSuffix::UL | IntSuffix::ULL) = suffix {
            return if u32::try_from(n).is_ok() && matches!(suffix, Some(IntSuffix::U)) {
                Primitive::Int(true)
            } else {
                Primitive::Long(true)
            };
        }
        if let Some(IntSuffix::L | IntSuffix::LL) = suffix {
            return if i64::try_from(n).is_ok() {
                Primitive::Long(false)
            } else {
                Primitive::Long(true)
            };
        }

        if i32::try_from(n).is_ok() {
            Primitive::Int(false)
        } else if u32::try_from(n).is_ok() && matches!(radix, Radix::Octal | Radix::Hex) {
            Primitive::Int(true)
        } else if i64::try_from(n).is_ok() {
            Primitive::Long(false)
        } else {
            Primitive::Long(true)
        }
    }
    fn fmt(&self) -> &str {
        match self {
            Primitive::Void => "void",
            Primitive::Char(false) => "char",
            Primitive::Char(true) => "unsigned char",
            Primitive::Short(false) => "short",
            Primitive::Short(true) => "unsigned short",
            Primitive::Int(false) => "int",
            Primitive::Int(true) => "unsigned int",
            Primitive::Long(false) => "long",
            Primitive::Long(true) => "unsigned long",
        }
    }

    pub fn is_unsigned(&self) -> bool {
        match self {
            Primitive::Char(true)
            | Primitive::Short(true)
            | Primitive::Int(true)
            | Primitive::Long(true) => true,
            _ => false,
        }
    }

    fn max(&self) -> u64 {
        match self {
            Primitive::Void => unreachable!(),
            Primitive::Char(false) => i8::MAX as u64,
            Primitive::Char(true) => u8::MAX as u64,
            Primitive::Short(false) => i16::MAX as u64,
            Primitive::Short(true) => u16::MAX as u64,
            Primitive::Int(false) => i32::MAX as u64,
            Primitive::Int(true) => u32::MAX as u64,
            Primitive::Long(false) => i64::MAX as u64,
            Primitive::Long(true) => u64::MAX,
        }
    }
    fn min(&self) -> i64 {
        match self {
            Primitive::Void => unreachable!(),
            Primitive::Char(true)
            | Primitive::Short(true)
            | Primitive::Int(true)
            | Primitive::Long(true) => 0,
            Primitive::Char(false) => i8::MIN as i64,
            Primitive::Short(false) => i16::MIN as i64,
            Primitive::Int(false) => i32::MIN as i64,
            Primitive::Long(false) => i64::MIN,
        }
    }
}

macro_rules! wrap_to {
    ($ty:expr,$n:expr,$prim:tt) => {
        match $ty {
            Type::Primitive(Primitive::Char(false)) => $n as i8 as $prim,
            Type::Primitive(Primitive::Char(true)) => $n as u8 as $prim,
            Type::Primitive(Primitive::Short(false)) => $n as i16 as $prim,
            Type::Primitive(Primitive::Short(true)) => $n as u16 as $prim,
            Type::Primitive(Primitive::Int(false)) | Type::Enum(..) => $n as i32 as $prim,
            Type::Primitive(Primitive::Int(true)) => $n as u32 as $prim,
            Type::Primitive(Primitive::Long(false)) => $n as i64 as $prim,
            Type::Pointer(_) | Type::Primitive(Primitive::Long(true)) => $n as u64 as $prim,
            _ => unreachable!("cast can only be scalar"),
        }
    };
}

/// Differentiates between signed and unsigned number literals,
/// necessary to do correct integer-constant-folding
#[derive(Debug, PartialEq, Clone)]
pub enum LiteralKind {
    Unsigned(u64),
    Signed(i64),
}
impl LiteralKind {
    pub fn try_i64(&self) -> Option<i64> {
        match self {
            LiteralKind::Signed(n) => Some(*n),
            LiteralKind::Unsigned(n) => i64::try_from(*n).ok(),
        }
    }
    pub fn wrap(&self, ty: &Type) -> LiteralKind {
        match self {
            LiteralKind::Signed(n) => LiteralKind::Signed(wrap_to!(ty, *n, i64)),
            LiteralKind::Unsigned(n) => LiteralKind::Unsigned(wrap_to!(ty, *n, u64)),
        }
    }
    pub fn type_overflow(&self, ty: &Type) -> bool {
        match self {
            LiteralKind::Signed(n) if *n < 0 => *n < ty.min(),
            LiteralKind::Signed(n) => *n as u64 > ty.max(),
            LiteralKind::Unsigned(n) => *n > ty.max(),
        }
    }
    pub fn is_zero(&self) -> bool {
        match self {
            LiteralKind::Signed(0) | LiteralKind::Unsigned(0) => true,
            _ => false,
        }
    }
    pub fn is_negative(&self) -> bool {
        match self {
            LiteralKind::Signed(n) => *n < 0,
            LiteralKind::Unsigned(_) => false,
        }
    }
}
impl ToString for LiteralKind {
    fn to_string(&self) -> String {
        match self {
            LiteralKind::Signed(n) => n.to_string(),
            LiteralKind::Unsigned(n) => n.to_string(),
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum StructKind {
    Named(String, StructRef),
    Unnamed(Token, Vec<(QualType, Token)>),
}
impl StructKind {
    pub fn members(&self) -> Rc<Vec<(QualType, Token)>> {
        match self {
            StructKind::Named(_, s) => s.get_members(),
            StructKind::Unnamed(_, m) => Rc::new(m.clone()),
        }
    }
    pub fn member_offset(&self, member_to_find: &str) -> usize {
        self.members()
            .iter()
            .take_while(|(_, name)| name.unwrap_string() != member_to_find)
            .fold(0, |acc, (t, _)| acc + t.ty.size())
    }
    pub fn member_type(&self, member_to_find: &str) -> QualType {
        self.members()
            .iter()
            .find(|(_, name)| name.unwrap_string() == member_to_find)
            .unwrap()
            .0
            .clone()
    }
    fn name(&self) -> String {
        match self {
            StructKind::Named(name, _) => name.to_string(),
            StructKind::Unnamed(token, _) => format!(
                "(<unnamed> at {}:{}:{})",
                token.filename.display(),
                token.line_index,
                token.column
            ),
        }
    }
    pub fn is_complete(&self) -> bool {
        match self {
            Self::Named(_, s) => s.is_complete(),
            Self::Unnamed(..) => true,
        }
    }
}

mod struct_ref {
    use super::QualType;
    use super::Token;
    use super::TokenKind;
    use std::cell::RefCell;
    use std::rc::Rc;

    struct StructInfo {
        members: Rc<Vec<(QualType, Token)>>,
        is_complete: bool,
        in_definition: bool,
    }

    thread_local! {
        static CUSTOMS: RefCell<Vec<StructInfo>> = Default::default();
    }

    #[derive(Clone, PartialEq, Debug)]
    pub struct StructRef {
        index: usize,
        kind: TokenKind,
    }

    impl StructRef {
        pub fn new(kind: TokenKind, is_definition: bool) -> StructRef {
            CUSTOMS.with(|list| {
                let mut types = list.borrow_mut();
                let index = types.len();
                types.push(StructInfo {
                    members: Rc::new(Vec::new()),
                    is_complete: false,
                    in_definition: is_definition,
                });

                StructRef { index, kind }
            })
        }
        pub fn get_kind(&self) -> &TokenKind {
            &self.kind
        }
        pub fn get_members(&self) -> Rc<Vec<(QualType, Token)>> {
            CUSTOMS.with(|list| list.borrow()[self.index].members.clone())
        }
        pub fn update_members(&self, members: Vec<(QualType, Token)>) {
            CUSTOMS.with(|list| {
                let mut types = list.borrow_mut();
                types[self.index].members = members.into();
            });
        }
        pub fn complete_def(&self, members: Vec<(QualType, Token)>) {
            CUSTOMS.with(|list| {
                let mut types = list.borrow_mut();
                types[self.index].is_complete = true;
                types[self.index].in_definition = false;
            });

            self.update_members(members);
        }
        pub fn is_complete(&self) -> bool {
            CUSTOMS.with(|list| list.borrow()[self.index].is_complete)
        }
        pub fn in_definition(&self) -> bool {
            CUSTOMS.with(|list| list.borrow()[self.index].in_definition)
        }

        pub fn being_defined(&self) {
            CUSTOMS.with(|list| list.borrow_mut()[self.index].in_definition = true)
        }
    }
}

impl Display for QualType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fn suffix_exists(modifiers: &[&QualType], i: usize) -> bool {
            modifiers
                .iter()
                .skip(i)
                .any(|m| matches!(m.ty, Type::Array { .. } | Type::Function { .. }))
        }
        fn closing_precedence(modifiers: &[&QualType], i: usize) -> &'static str {
            if matches!(modifiers.get(i + 1).map(|m| &m.ty), Some(Type::Pointer(_)))
                && suffix_exists(modifiers, i + 1)
            {
                ")"
            } else {
                ""
            }
        }
        fn pointer_precedence(modifiers: &[&QualType], i: usize) -> bool {
            matches!(
                modifiers.get(i + 1).map(|m| &m.ty),
                Some(Type::Array { .. } | Type::Function { .. })
            )
        }
        fn print_qualifiers(qualifiers: &Qualifiers) -> String {
            let mut result = String::new();
            for (has, qual) in [
                (qualifiers.is_const, "const"),
                (qualifiers.is_volatile, "volatile"),
                (qualifiers.is_restrict, "restrict"),
            ] {
                if has {
                    result.push_str(qual);
                    result.push(' ');
                }
            }
            result
        }

        fn print_type(qtype: &QualType) -> String {
            let mut current = qtype;
            let mut modifiers = Vec::new();

            while let Type::Pointer(new)
            | Type::Array(new, _)
            | Type::Function(FuncType { return_type: new, .. }) = &current.ty
            {
                modifiers.push(current);
                current = new;
            }
            let mut result = print_qualifiers(&current.qualifiers);

            result.push_str(&match &current.ty {
                Type::Primitive(prim) => prim.fmt().to_string(),
                Type::Union(s) => "union ".to_string() + &s.name(),
                Type::Struct(s) => "struct ".to_string() + &s.name(),
                Type::Enum(Some(name), ..) => "enum ".to_string() + &name,
                Type::Enum(None, ..) => "enum <unnamed>".to_string(),
                _ => unreachable!("all modifiers were removed"),
            });
            if !modifiers.is_empty() {
                result.push(' ');
            }
            let mut pointers = Vec::new();
            let mut suffixes = Vec::new();

            for (i, modifier) in modifiers.iter().enumerate() {
                match &modifier.ty {
                    Type::Array(_, size) => suffixes.push(format!(
                        "[{}]{}",
                        match size {
                            ArraySize::Known(size) => size.to_string(),
                            ArraySize::Unknown => String::new(),
                        },
                        closing_precedence(&modifiers, i)
                    )),
                    Type::Pointer(_) => {
                        let precedence = pointer_precedence(&modifiers, i);
                        let mut quals = print_qualifiers(&modifier.qualifiers);
                        // trim trailing whitespace of last pointer to be printed
                        // otherwise `int *const` would be printed `int *const `
                        if pointers.is_empty() {
                            quals = quals.trim_end().to_string()
                        }

                        pointers.push(match precedence {
                            true if pointers.is_empty() && suffixes.is_empty() => {
                                format!("(*{})", quals)
                            }
                            true => format!("(*{}", quals),
                            false
                                if suffixes.is_empty()
                                    && suffix_exists(&modifiers, i)
                                    && pointers.is_empty() =>
                            {
                                format!("*{})", quals)
                            }
                            _ => format!("*{}", quals),
                        });
                    }
                    Type::Function(FuncType { params, variadic, .. }) => suffixes.push(format!(
                        "({}{}){}",
                        params
                            .iter()
                            .map(|ty| ty.to_string())
                            .collect::<Vec<_>>()
                            .join(", "),
                        if *variadic { ", ..." } else { "" },
                        closing_precedence(&modifiers, i)
                    )),
                    _ => unreachable!("not modifier"),
                }
            }
            for s in pointers.iter().rev() {
                result.push_str(s);
            }
            for s in suffixes {
                result.push_str(&s);
            }

            result
        }
        write!(f, "{}", print_type(self))
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[macro_export]
    macro_rules! setup_type {
        ($input:expr) => {
            if let Ok(ty) = crate::compiler::parser::tests::setup($input).type_name() {
                if let Ok(actual_ty) = crate::compiler::typechecker::TypeChecker::new().parse_type(
                    &crate::compiler::common::token::Token::default(
                        crate::compiler::common::token::TokenKind::Semicolon,
                    ),
                    ty,
                ) {
                    actual_ty
                } else {
                    unreachable!("not type declaration")
                }
            } else {
                unreachable!("not type declaration")
            }
        };
        // if type depends on an already existing environment, supply said environment
        ($input:expr,$typechecker:expr) => {
            if let Ok(ty) = setup($input).type_name() {
                if let Ok(actual_ty) = $typechecker.parse_type(&Token::default(TokenKind::Semicolon), ty)
                {
                    actual_ty
                } else {
                    unreachable!("not type declaration")
                }
            } else {
                unreachable!("not type declaration")
            }
        };
    }
    fn assert_type_print(input: &str, expected: &str) {
        let type_string = setup_type!(input);
        assert_eq!(type_string.to_string(), expected);
    }

    #[test]
    fn multi_dim_arr_print() {
        assert_type_print("int [4][2]", "int [4][2]");
        assert_type_print("int ([3])[4][2]", "int [3][4][2]");

        assert_type_print("long int *[3][4][2]", "long *[3][4][2]");
        assert_type_print("char ***[2]", "char ***[2]");

        assert_type_print("char *((*))[2]", "char *(*)[2]");
        assert_type_print("char *(**)[2]", "char *(**)[2]");
        assert_type_print("char *(**)", "char ***");

        assert_type_print("char *(*)[3][4][2]", "char *(*)[3][4][2]");
        assert_type_print("short (**[3][4])[2]", "short (**[3][4])[2]");
        assert_type_print("char (**(*)[4])[2]", "char (**(*)[4])[2]");
        assert_type_print("char(**(*[3])[4])[2]", "char (**(*[3])[4])[2]");

        assert_type_print("char (*(*[3]))[2]", "char (**[3])[2]");
    }
    #[test]
    fn function_type_print() {
        assert_type_print("int ()", "int ()"); // should this rather be `int (int)`?
        assert_type_print("int (int)", "int (int)");
        assert_type_print("int (int ())", "int (int (*)())");

        assert_type_print("int ((()))", "int (int (*)(int (*)()))");
        assert_type_print("int (char[2])", "int (char *)");

        assert_type_print("void *(*(int[2], char (void)))", "void **(int *, char (*)())");
        assert_type_print("int (*(void))[3]", "int (*())[3]");
        assert_type_print(
            "int (**(int[2], char(void)))[3];",
            "int (**(int *, char (*)()))[3]",
        );

        assert_type_print("short *(short int**, ...)", "short *(short **, ...)");
    }
    #[test]
    fn qualifers() {
        assert_type_print("const int", "const int");
        assert_type_print("const int*", "const int *");
        assert_type_print("int *const", "int *const");
        assert_type_print("int *const[4]", "int *const[4]");
        assert_type_print(
            "char (*const restrict *(*volatile)[4])[2]",
            "char (*const restrict *(*volatile)[4])[2]",
        );

        assert_type_print("const int (int *restrict)", "const int (int *restrict)");
    }

    #[test]
    fn contains_all_qualifiers() {
        let left = Qualifiers {
            is_const: true,
            is_volatile: false,
            is_restrict: false,
        };
        let right = Qualifiers::default();

        assert!(left.contains_all(&right));
    }

    #[test]
    fn unsigned() {
        assert_type_print("unsigned int", "unsigned int");
        assert_type_print("unsigned", "unsigned int");
        assert_type_print("unsigned long", "unsigned long");
        assert_type_print("unsigned char (unsigned)", "unsigned char (unsigned int)");
    }
}
