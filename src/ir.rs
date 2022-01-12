use crate::types::Pri;
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

impl Expr {
    pub fn var(val: u16) -> Box<Expr> {
        Box::new(Expr::Var(Var(val)))
    }

    pub fn num(val: u16) -> Box<Expr> {
        Box::new(Expr::Num(val))
    }

    pub fn uv(val: u16) -> Box<Expr> {
        Box::new(Expr::Uv(val))
    }

    pub fn str(val: u16) -> Box<Expr> {
        Box::new(Expr::Str(val))
    }

    pub fn cdata(val: u16) -> Box<Expr> {
        Box::new(Expr::Cdata(val))
    }

    pub fn short(val: i16) -> Box<Expr> {
        Box::new(Expr::Short(val as i16))
    }

    pub fn primitive(val: Pri) -> Box<Expr> {
        match val {
            Pri::Nil => Box::new(Expr::Nil),
            Pri::True => Box::new(Expr::Bool(true)),
            Pri::False => Box::new(Expr::Bool(false)),
        }
    }

    pub fn add(a: Box<Expr>, b: Box<Expr>) -> Box<Expr> {
        Box::new(Expr::Add([a, b]))
    }

    pub fn sub(a: Box<Expr>, b: Box<Expr>) -> Box<Expr> {
        Box::new(Expr::Sub([a, b]))
    }

    pub fn mod_(a: Box<Expr>, b: Box<Expr>) -> Box<Expr> {
        Box::new(Expr::Mod([a, b]))
    }

    pub fn mul(a: Box<Expr>, b: Box<Expr>) -> Box<Expr> {
        Box::new(Expr::Mul([a, b]))
    }

    pub fn div(a: Box<Expr>, b: Box<Expr>) -> Box<Expr> {
        Box::new(Expr::Div([a, b]))
    }

    pub fn pow(a: Box<Expr>, b: Box<Expr>) -> Box<Expr> {
        Box::new(Expr::Pow([a, b]))
    }

    pub fn table(a: Box<Expr>, b: Box<Expr>) -> Box<Expr> {
        Box::new(Expr::Table([a, b]))
    }

    pub fn lt(a: Box<Expr>, b: Box<Expr>) -> Box<Expr> {
        Box::new(Expr::Lt([a, b]))
    }

    pub fn gt(a: Box<Expr>, b: Box<Expr>) -> Box<Expr> {
        Box::new(Expr::Gt([a, b]))
    }

    pub fn le(a: Box<Expr>, b: Box<Expr>) -> Box<Expr> {
        Box::new(Expr::Le([a, b]))
    }

    pub fn ge(a: Box<Expr>, b: Box<Expr>) -> Box<Expr> {
        Box::new(Expr::Ge([a, b]))
    }

    pub fn eq(a: Box<Expr>, b: Box<Expr>) -> Box<Expr> {
        Box::new(Expr::Eq([a, b]))
    }

    pub fn ne(a: Box<Expr>, b: Box<Expr>) -> Box<Expr> {
        Box::new(Expr::Ne([a, b]))
    }
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
    Return(Box<[Expr]>),
}

impl fmt::Display for Insn {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Insn::SetVar(v, expr) => format!("{} = {}", v, expr),
                Insn::SetGlobalTableVar(..) => format!(""),
                Insn::SetTableVar(..) => format!(""),
                Insn::Call(rets, args) => {
                    let mut res = "".to_string();

                    if rets.len() > 0 {
                        res.push_str(&format!("{}", rets[0]));

                        for ret in rets[1..].iter() {
                            res.push_str(&format!(", {}", ret));
                        }

                        res.push_str(" = ");
                    }

                    res.push_str(&format!("{}(", args[0]));

                    if args.len() > 1 {
                        res.push_str(&format!("{}", args[1]));

                        for arg in args[2..].iter() {
                            res.push_str(&format!(", {}", arg));
                        }

                        res.push_str(")");
                    }

                    res
                }
                Insn::Cat(..) => format!(""),
                Insn::If(expr) => format!("if {}", expr),
                Insn::For(..) => format!(""),
                Insn::While(..) => format!(""),
                Insn::Repeat(..) => format!(""),
                Insn::Return(expr) => {
                    let mut res = "return".to_string();

                    if expr.len() >= 1 {
                        res.push_str(&format!(" {}", expr[0]));

                        if expr.len() > 1 {
                            println!("TEST!")
                        }
                        for ret in expr[1..].iter() {
                            res.push_str(&format!(", {}", ret));
                        }
                    }

                    res
                }
            }
        )
    }
}

#[derive(Default)]
pub struct Block {
    data: Vec<Insn>,
}

impl Block {
    pub fn push_insn(&mut self, ins: Insn) {
        self.data.push(ins);
    }

    pub fn iter_insn(&self) -> impl Iterator<Item = &Insn> {
        self.data.iter()
    }
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
