use crate::{private::Sealed, CtorBlocker};

use super::{ty::SemTyKind, ExprId, Span, SpanId};

use std::{fmt::Debug, marker::PhantomData};

mod block_expr;
mod call_exprs;
mod control_flow_expr;
mod ctor_expr;
mod lit_expr;
mod op_exprs;
mod path_expr;
mod place_expr;
mod unstable_expr;
pub use block_expr::*;
pub use call_exprs::*;
pub use control_flow_expr::*;
pub use ctor_expr::*;
pub use lit_expr::*;
pub use op_exprs::*;
pub use path_expr::*;
pub use place_expr::*;
pub use unstable_expr::*;

/// This trait combines methods, which are common between all expressions.
///
/// This trait is only meant to be implemented inside this crate. The `Sealed`
/// super trait prevents external implementations.
pub trait ExprData<'ast>: Debug + Sealed {
    /// Returns the [`ExprId`] of this expression.
    fn id(&self) -> ExprId;

    /// Returns the [`Span`] of this expression.
    fn span(&self) -> &Span<'ast>;

    /// Returns the semantic type of this expression.
    fn ty(&self) -> SemTyKind<'ast>;

    /// Returns the [`ExprPrecedence`] of this expression.
    fn precedence(&self) -> ExprPrecedence;

    /// Returns this expression wrapped in it's [`ExprKind`] variant.
    ///
    /// In function parameters, it's recommended to use `Into<ExprKind<'ast>>`
    /// as a bound to support all expressions and `ExprKind<'ast>` as parameters.
    fn as_expr(&'ast self) -> ExprKind<'ast>;
}

#[repr(C)]
#[non_exhaustive]
#[derive(Debug, Copy, Clone)]
pub enum ExprKind<'ast> {
    IntLit(&'ast IntLitExpr<'ast>),
    FloatLit(&'ast FloatLitExpr<'ast>),
    StrLit(&'ast StrLitExpr<'ast>),
    CharLit(&'ast CharLitExpr<'ast>),
    BoolLit(&'ast BoolLitExpr<'ast>),
    Block(&'ast BlockExpr<'ast>),
    Closure(&'ast ClosureExpr<'ast>),
    UnaryOp(&'ast UnaryOpExpr<'ast>),
    Ref(&'ast RefExpr<'ast>),
    BinaryOp(&'ast BinaryOpExpr<'ast>),
    QuestionMark(&'ast QuestionMarkExpr<'ast>),
    Assign(&'ast AssignExpr<'ast>),
    As(&'ast AsExpr<'ast>),
    Path(&'ast PathExpr<'ast>),
    Call(&'ast CallExpr<'ast>),
    Method(&'ast MethodExpr<'ast>),
    Array(&'ast ArrayExpr<'ast>),
    Tuple(&'ast TupleExpr<'ast>),
    Ctor(&'ast CtorExpr<'ast>),
    Range(&'ast RangeExpr<'ast>),
    Index(&'ast IndexExpr<'ast>),
    Field(&'ast FieldExpr<'ast>),
    If(&'ast IfExpr<'ast>),
    Let(&'ast LetExpr<'ast>),
    Match(&'ast MatchExpr<'ast>),
    Break(&'ast BreakExpr<'ast>),
    Return(&'ast ReturnExpr<'ast>),
    Continue(&'ast ContinueExpr<'ast>),
    For(&'ast ForExpr<'ast>),
    Loop(&'ast LoopExpr<'ast>),
    While(&'ast WhileExpr<'ast>),
    Await(&'ast AwaitExpr<'ast>),
    Unstable(&'ast UnstableExpr<'ast>),
}

impl<'ast> ExprKind<'ast> {
    impl_expr_kind_fn!(ExprKind: span() -> &Span<'ast>);
    impl_expr_kind_fn!(ExprKind: id() -> ExprId);
    impl_expr_kind_fn!(ExprKind: ty() -> SemTyKind<'ast>);
    impl_expr_kind_fn!(ExprKind: precedence() -> ExprPrecedence);
}

impl Sealed for ExprKind<'_> {}

#[repr(C)]
#[non_exhaustive]
#[derive(Debug, Copy, Clone)]
pub enum LitExprKind<'ast> {
    Int(&'ast IntLitExpr<'ast>),
    Float(&'ast FloatLitExpr<'ast>),
    Str(&'ast StrLitExpr<'ast>),
    Char(&'ast CharLitExpr<'ast>),
    Bool(&'ast BoolLitExpr<'ast>),
    /// Rust represents negative numbers as positive literals with a unary
    /// negation operator in front. This unary expression should therefore
    /// be seen as a part of a literal, that it wraps.
    UnaryOp(&'ast UnaryOpExpr<'ast>, CtorBlocker),
}

impl<'ast> LitExprKind<'ast> {
    impl_expr_kind_fn!(LitExprKind: span() -> &Span<'ast>);
    impl_expr_kind_fn!(LitExprKind: id() -> ExprId);
    impl_expr_kind_fn!(LitExprKind: ty() -> SemTyKind<'ast>);
    impl_expr_kind_fn!(LitExprKind: precedence() -> ExprPrecedence);
}

impl<'ast> From<LitExprKind<'ast>> for ExprKind<'ast> {
    fn from(value: LitExprKind<'ast>) -> Self {
        match value {
            LitExprKind::Int(expr) => ExprKind::IntLit(expr),
            LitExprKind::Float(expr) => ExprKind::FloatLit(expr),
            LitExprKind::Str(expr) => ExprKind::StrLit(expr),
            LitExprKind::Char(expr) => ExprKind::CharLit(expr),
            LitExprKind::Bool(expr) => ExprKind::BoolLit(expr),
            LitExprKind::UnaryOp(expr, ..) => ExprKind::UnaryOp(expr),
        }
    }
}

impl<'ast> TryFrom<ExprKind<'ast>> for LitExprKind<'ast> {
    type Error = ();

    fn try_from(value: ExprKind<'ast>) -> Result<Self, Self::Error> {
        match value {
            ExprKind::IntLit(expr) => Ok(LitExprKind::Int(expr)),
            ExprKind::FloatLit(expr) => Ok(LitExprKind::Float(expr)),
            ExprKind::StrLit(expr) => Ok(LitExprKind::Str(expr)),
            ExprKind::CharLit(expr) => Ok(LitExprKind::Char(expr)),
            ExprKind::BoolLit(expr) => Ok(LitExprKind::Bool(expr)),
            ExprKind::UnaryOp(expr) => {
                // Only accept this conversion if this operation negates a literal.
                if expr.kind() == UnaryOpKind::Neg && TryInto::<LitExprKind<'_>>::try_into(expr.expr()).is_ok() {
                    Ok(LitExprKind::UnaryOp(expr, CtorBlocker::new()))
                } else {
                    Err(())
                }
            },
            _ => Err(()),
        }
    }
}

#[repr(u32)]
#[non_exhaustive]
#[derive(Debug, Copy, Clone)]
pub enum ExprPrecedence {
    Lit = 0x1400_0000,
    Block = 0x1400_0001,
    Ctor = 0x1400_0002,
    Assign = 0x1400_0003,
    For = 0x1400_0004,
    Loop = 0x1400_0005,
    While = 0x1400_0006,
    Await = 0x1400_0007,

    Path = 0x1300_0000,

    Method = 0x1200_0000,
    Call = 0x1200_0001,
    // These three are just a guess, as they're not listed in the precedence table
    If = 0x1200_0002,
    Let = 0x1200_0003,
    Match = 0x1200_0004,

    Field = 0x1100_0000,

    Fn = 0x1000_0000,
    Index = 0x1000_0001,

    QuestionMark = 0x0F00_0000,

    /// The unary `-` operator
    Neg = 0x0E00_0000,
    /// The `!` operator
    Not = 0x0E00_0001,
    /// The unary `*` operator
    Deref = 0x0E00_0002,
    /// The unary `&` operator
    Ref = 0x0E00_0003,

    As = 0x0D00_0000,

    /// The binary `*` operator
    Mul = 0x0C00_0000,
    /// The `/` operator
    Div = 0x0C00_0001,
    /// The `%` operator
    Rem = 0x0C00_0002,

    /// The `+` operator
    Add = 0x0B00_0000,
    /// The binary `-` operator
    Sub = 0x0B00_0001,

    /// The `>>` operator
    Shr = 0x0A00_0000,
    /// The `<<` operator
    Shl = 0x0A00_0001,

    /// The binary `&` operator
    BitAnd = 0x0900_0000,

    /// The `^` operator
    BitXor = 0x0800_0000,

    /// The `|` operator
    BitOr = 0x0700_0000,

    /// The `==`, `!=`, `<`, `<=`, `>`, `>=` operators
    Comparison = 0x0600_0000,

    /// The `&&` operator
    And = 0x0500_0000,

    /// The `||` operator
    Or = 0x0400_0000,

    /// Ranges `0..10`, `0..=8`
    Range = 0x0300_0000,

    /// This precedence level includes compound assignment operators, like:
    /// `+=`, `-=`, `*=`, `/=`, `%=`, `&=`, `|=`, `^=`, `<<=`, `>>=`
    AssignOp = 0x0200_0000,

    Closure = 0x0100_0000,
    Break = 0x0100_0001,
    Return = 0x0100_0002,
    Continue = 0x0100_0003,
    /// The precedence originates from an unstable source. The stored value provides
    /// the current precedence of this expression. This might change in the future
    Unstable(i32),
}

macro_rules! impl_expr_kind_fn {
    (ExprKind: $method:ident () -> $return_ty:ty) => {
        impl_expr_kind_fn!((ExprKind) $method() -> $return_ty,
            IntLit, FloatLit, StrLit, CharLit, BoolLit,
            Block, Closure,
            UnaryOp, Ref, BinaryOp, QuestionMark, As, Assign,
            Path, Index, Field,
            Call, Method,
            Array, Tuple, Ctor, Range,
            If, Let, Match, Break, Return, Continue, For, Loop, While,
            Await,
            Unstable
        );
    };
    (LitExprKind: $method:ident () -> $return_ty:ty) => {
        impl_expr_kind_fn!((LitExprKind) $method() -> $return_ty,
            Int, Float, Str, Char, Bool, UnaryOp
        );
    };
    (($self:ident) $method:ident () -> $return_ty:ty $(, $kind:ident)+) => {
        pub fn $method(&self) -> $return_ty {
            match self {
                $($self::$kind(data, ..) => data.$method(),)*
            }
        }
    };
}

use impl_expr_kind_fn;

#[repr(C)]
#[derive(Debug)]
#[cfg_attr(feature = "driver-api", visibility::make(pub))]
struct CommonExprData<'ast> {
    /// The lifetime is not needed right now, but it's safer to include it for
    /// future additions. Having it in this struct makes it easier for all
    /// expression structs, as they will have a valid use for `'ast` even if they
    /// don't need it. Otherwise, we might need to declare this field in each
    /// expression.
    _lifetime: PhantomData<&'ast ()>,
    id: ExprId,
    span: SpanId,
}

#[cfg(feature = "driver-api")]
impl<'ast> CommonExprData<'ast> {
    pub fn new(id: ExprId, span: SpanId) -> Self {
        Self {
            _lifetime: PhantomData,
            id,
            span,
        }
    }
}

macro_rules! impl_expr_data {
    ($self_ty:ty, $enum_name:ident) => {
        $crate::ast::expr::impl_expr_data!($self_ty, $enum_name,
            fn precedence(&self) -> $crate::ast::expr::ExprPrecedence {
                $crate::ast::expr::ExprPrecedence::$enum_name
            }
        );
    };
    ($self_ty:ty, $enum_name:ident, $precedence_fn:item) => {
        impl<'ast> super::ExprData<'ast> for $self_ty {
            fn id(&self) -> crate::ast::ExprId {
                self.data.id
            }

            fn span(&self) -> &crate::ast::Span<'ast> {
                $crate::context::with_cx(self, |cx| cx.span(self.data.span))
            }

            fn ty(&self) -> $crate::ast::ty::SemTyKind<'ast> {
                $crate::context::with_cx(self, |cx| cx.expr_ty(self.data.id))
            }

            $precedence_fn

            fn as_expr(&'ast self) -> crate::ast::expr::ExprKind<'ast> {
                $crate::ast::expr::ExprKind::$enum_name(self)
            }
        }

        impl<'ast> $crate::private::Sealed for $self_ty {}

        impl<'ast> From<&'ast $self_ty> for $crate::ast::expr::ExprKind<'ast> {
            fn from(from: &'ast $self_ty) -> Self {
                $crate::ast::expr::ExprKind::$enum_name(from)
            }
        }
    };
}

use impl_expr_data;

/// An expression that is evaluated at compile time. These show up in array
/// indices and constant generics.
///
/// Marker currently doesn't provide a way to calculate the result of a constant
/// expression.
#[repr(C)]
#[derive(Debug)]
pub struct ConstExpr<'ast> {
    expr: ExprKind<'ast>,
}

impl<'ast> ConstExpr<'ast> {
    /// This returns the wrapped expression that will be evaluated at compile time.
    pub fn expr(&self) -> ExprKind<'ast> {
        self.expr
    }
}

#[cfg(feature = "driver-api")]
impl<'ast> ConstExpr<'ast> {
    pub fn new(expr: ExprKind<'ast>) -> Self {
        Self { expr }
    }
}

#[cfg(all(test, target_arch = "x86_64", target_pointer_width = "64"))]
mod test {
    use super::*;
    use std::mem::size_of;

    #[test]
    fn test_expr_struct_size() {
        // These sizes are allowed to change, this is just a check to have a
        // general overview and to prevent accidental changes
        assert_eq!(40, size_of::<IntLitExpr<'_>>(), "IntLitExpr<'_>");
        assert_eq!(32, size_of::<FloatLitExpr<'_>>(), "FloatLitExpr<'_>");
        assert_eq!(48, size_of::<StrLitExpr<'_>>(), "StrLitExpr<'_>");
        assert_eq!(24, size_of::<CharLitExpr<'_>>(), "CharLitExpr<'_>");
        assert_eq!(24, size_of::<BoolLitExpr<'_>>(), "BoolLitExpr<'_>");
        assert_eq!(96, size_of::<BlockExpr<'_>>(), "BlockExpr<'_>");
        assert_eq!(72, size_of::<ClosureExpr<'_>>(), "ClosureExpr<'_>");
        assert_eq!(40, size_of::<UnaryOpExpr<'_>>(), "UnaryOpExpr<'_>");
        assert_eq!(40, size_of::<RefExpr<'_>>(), "RefExpr<'_>");
        assert_eq!(56, size_of::<BinaryOpExpr<'_>>(), "BinaryOpExpr<'_>");
        assert_eq!(32, size_of::<QuestionMarkExpr<'_>>(), "QuestionMarkExpr<'_>");
        assert_eq!(80, size_of::<AssignExpr<'_>>(), "AssignExpr<'_>");
        assert_eq!(48, size_of::<AsExpr<'_>>(), "AsExpr<'_>");
        assert_eq!(96, size_of::<PathExpr<'_>>(), "PathExpr<'_>");
        assert_eq!(48, size_of::<CallExpr<'_>>(), "CallExpr<'_>");
        assert_eq!(80, size_of::<MethodExpr<'_>>(), "MethodExpr<'_>");
        assert_eq!(56, size_of::<ArrayExpr<'_>>(), "ArrayExpr<'_>");
        assert_eq!(32, size_of::<TupleExpr<'_>>(), "TupleExpr<'_>");
        assert_eq!(136, size_of::<CtorExpr<'_>>(), "CtorExpr<'_>");
        assert_eq!(72, size_of::<RangeExpr<'_>>(), "RangeExpr<'_>");
        assert_eq!(48, size_of::<IndexExpr<'_>>(), "IndexExpr<'_>");
        assert_eq!(48, size_of::<FieldExpr<'_>>(), "FieldExpr<'_>");
        assert_eq!(72, size_of::<IfExpr<'_>>(), "IfExpr<'_>");
        assert_eq!(72, size_of::<LetExpr<'_>>(), "LetExpr<'_>");
        assert_eq!(48, size_of::<MatchExpr<'_>>(), "MatchExpr<'_>");
        assert_eq!(72, size_of::<BreakExpr<'_>>(), "BreakExpr<'_>");
        assert_eq!(40, size_of::<ReturnExpr<'_>>(), "ReturnExpr<'_>");
        assert_eq!(48, size_of::<ContinueExpr<'_>>(), "ContinueExpr<'_>");
        assert_eq!(112, size_of::<ForExpr<'_>>(), "ForExpr<'_>");
        assert_eq!(56, size_of::<LoopExpr<'_>>(), "LoopExpr<'_>");
        assert_eq!(72, size_of::<WhileExpr<'_>>(), "WhileExpr<'_>");
        assert_eq!(24, size_of::<UnstableExpr<'_>>(), "UnstableExpr<'_>");
    }
}
