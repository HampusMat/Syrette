use proc_macro2::Ident;
use syn::punctuated::Punctuated;
use syn::token::Paren;
use syn::{
    Expr,
    ExprCall,
    ExprLit,
    ExprMethodCall,
    ExprPath,
    GenericMethodArgument,
    Lit,
    MethodTurbofish,
    Path,
    Token,
};

pub trait ExprMethodCallExt
{
    fn new(receiver: Expr, method: Ident, args: impl IntoIterator<Item = Expr>) -> Self;

    fn with_turbofish(self, turbofish: MethodTurbofish) -> Self;
}

impl ExprMethodCallExt for ExprMethodCall
{
    fn new(receiver: Expr, method: Ident, args: impl IntoIterator<Item = Expr>) -> Self
    {
        Self {
            attrs: Vec::new(),
            receiver: Box::new(receiver),
            dot_token: <Token![.]>::default(),
            method,
            turbofish: None,
            paren_token: Paren::default(),
            args: Punctuated::from_iter(args),
        }
    }

    fn with_turbofish(mut self, turbofish: MethodTurbofish) -> Self
    {
        self.turbofish = Some(turbofish);

        self
    }
}

pub trait ExprPathExt
{
    fn new(path: Path) -> Self;
}

impl ExprPathExt for ExprPath
{
    fn new(path: Path) -> Self
    {
        Self {
            attrs: Vec::new(),
            qself: None,
            path,
        }
    }
}

pub trait MethodTurbofishExt
{
    fn new(args: impl IntoIterator<Item = GenericMethodArgument>) -> Self;
}

impl MethodTurbofishExt for MethodTurbofish
{
    fn new(args: impl IntoIterator<Item = GenericMethodArgument>) -> Self
    {
        Self {
            colon2_token: <Token![::]>::default(),
            lt_token: <Token![<]>::default(),
            args: Punctuated::from_iter(args),
            gt_token: <Token![>]>::default(),
        }
    }
}

pub trait ExprCallExt
{
    fn new(function: Expr, args: impl IntoIterator<Item = Expr>) -> Self;
}

impl ExprCallExt for ExprCall
{
    fn new(function: Expr, args: impl IntoIterator<Item = Expr>) -> Self
    {
        Self {
            attrs: Vec::new(),
            func: Box::new(function),
            paren_token: Paren::default(),
            args: Punctuated::from_iter(args),
        }
    }
}

pub trait ExprLitExt
{
    fn new(lit: impl Into<Lit>) -> Self;
}

impl ExprLitExt for ExprLit
{
    fn new(lit: impl Into<Lit>) -> Self
    {
        Self {
            attrs: Vec::new(),
            lit: lit.into(),
        }
    }
}
