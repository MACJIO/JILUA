use std::fmt;
use std::fmt::{write, Formatter};

#[derive(Debug)]
pub enum Expr {
    // variable slot
    Var(Var),

    // constants
    Cdata(u16),
    Str(u16),
    Num(u16),
    Lit(u8),
    Short(i16),
    Uv(u16),
    Bool(bool),
    Nil,

    // comparison expressions
    Lt([Box<Expr>; 2]),
    Ge([Box<Expr>; 2]),
    Le([Box<Expr>; 2]),
    Gt([Box<Expr>; 2]),
    Eq([Box<Expr>; 2]),
    Ne([Box<Expr>; 2]),

    // unary expressions
    Not(Box<Expr>),
    Len(Box<Expr>),
    Minus(Box<Expr>),

    // binary expressions
    Add([Box<Expr>; 2]),
    Sub([Box<Expr>; 2]),
    Mul([Box<Expr>; 2]),
    Div([Box<Expr>; 2]),
    Mod([Box<Expr>; 2]),
    Pow([Box<Expr>; 2]),

    GlobalTable,
    Table([Box<Expr>; 2]), // (table, index)
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Expr::Var(a) => format!("v{}", a.0),
                Expr::Str(a) => format!("String({})", a),
                Expr::Num(a) => format!("Num({})", a),
                Expr::Lit(a) => format!("Lit({})", a),
                Expr::Short(a) => format!("Short({})", a),
                Expr::Uv(a) => format!("UV({})", a),
                Expr::Bool(a) => format!("Bool({})", a),
                Expr::Cdata(a) => format!("CData({})", a),
                Expr::Nil => "Nil".to_string(),
                Expr::Lt([a, b]) => format!("{} < {}", a, b),
                Expr::Ge([a, b]) => format!("{} ≥ {}", a, b),
                Expr::Le([a, b]) => format!("{} ≤ {}", a, b),
                Expr::Gt([a, b]) => format!("{} > {}", a, b),
                Expr::Eq([a, b]) => format!("{} == {}", a, b),
                Expr::Ne([a, b]) => format!("{} != {}", a, b),
                Expr::Not(a) => format!("!{}", a),
                Expr::Len(a) => format!("len({})", a),
                Expr::Minus(a) => format!("-{}", a),
                Expr::Add([a, b]) => format!("{} + {}", a, b),
                Expr::Sub([a, b]) => format!("{} - {}", a, b),
                Expr::Mul([a, b]) => format!("{} * {}", a, b),
                Expr::Div([a, b]) => format!("{} / {}", a, b),
                Expr::Mod([a, b]) => format!("{} % {}", a, b),
                Expr::Pow([a, b]) => format!("{}^{}", a, b),
                Expr::GlobalTable => format!("_G"),
                Expr::Table([a, b]) => format!("{}[{}]", a, b),
            }
        )
    }
}

#[derive(Debug)]
pub struct Var(pub u16);

impl fmt::Display for Var {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "v{}", self.0)
    }
}

#[derive(Debug)]
pub struct VarInfo {
    name: String,
}

#[derive(Debug)]
pub enum Insn {
    SetVar(Var, Box<Expr>),
    SetGlobalTableVar(Var, Box<Expr>),
    SetTableVar(Var, Box<Expr>),
    Call(Box<[Var]>, Box<[Expr]>),
    Cat(Var, Box<[Expr]>),
    If(Box<Expr>),
    For(Box<Expr>),
    While(Box<Expr>),
    Repeat(Box<Expr>),
}

impl fmt::Display for Insn {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Insn::SetVar(v, exp) => format!("{} = {}", v, exp),
                Insn::SetGlobalTableVar(..) => format!(""),
                Insn::SetTableVar(..) => format!(""),
                Insn::Call(v, exp) => format!(""),
                Insn::Cat(..) => format!(""),
                Insn::If(..) => format!(""),
                Insn::For(..) => format!(""),
                Insn::While(..) => format!(""),
                Insn::Repeat(..) => format!(""),
            }
        )
    }
}

#[derive(Default)]
pub struct Block {
    data: Vec<Insn>,
}

#[cfg(test)]
mod tests {
    use crate::ir::Expr;
    use crate::ir::Var;

    #[test]
    fn expressions_fmt() {
        let expr1 = Expr::Var(Var(1));
        let expr2 = Expr::Nil;
        let expr3 = Expr::Ge([Box::new(expr1), Box::new(expr2)]);

        assert_eq!("v1 ≥ Nil", format!("{}", expr3));
    }
}
