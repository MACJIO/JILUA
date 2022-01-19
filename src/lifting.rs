use std::collections::HashMap;
use std::env::var;
use crate::disasm::disasm;
use crate::ir::{Block, Expr, Insn, Var, VarInfo};
use crate::op::Op;
use crate::resolver::BranchKind;
use crate::{ByteCodeProto, DecompileError, Graph};

pub struct Lifter {
    slots: HashMap<u16, VarInfo>,
}

impl Lifter {
    pub(crate) fn new() -> Self {
        Self {
            slots: HashMap::new()
        }
    }

    fn var_for_slot(&mut self, index: u16, table: bool) -> Var {
        // let len = self.slots.len();
        if let Some(var_info) = self.slots.get_mut(&index) {
            var_info.increment_usage_counter();

            Var(index)
        } else {
            self.slots.insert(index, VarInfo::new(index, table));
            Var(index)
        }
    }

    pub fn analyze_bc_proto(&mut self, bc_proto: &ByteCodeProto) -> Result<Graph<Block, BranchKind>, DecompileError> {
        let mut graph: Graph<Block, BranchKind> = bc_proto.basic_block_graph_ref().structure_copy();

        for (block_idx, basic_block) in bc_proto.basic_block_graph_ref().iter_node_weights() {
            let mut iter = basic_block.data().iter();

            let analyzed_block = graph.node_weight_mut(block_idx).unwrap();

            while let Some(&raw_ins) = iter.next() {
                match disasm(raw_ins)? {
                    // comparison
                    Op::ISLT(a, b) => {
                        analyzed_block.push_insn(Insn::If(Expr::lt(Expr::var(a.0), Expr::var(b.0))));
                    }
                    Op::ISGE(a, b) => {
                        analyzed_block.push_insn(Insn::If(Expr::ge(Expr::var(a.0), Expr::var(b.0))));
                    }
                    Op::ISLE(a, b) => {
                        analyzed_block.push_insn(Insn::If(Expr::le(Expr::var(a.0), Expr::var(b.0))));
                    }
                    Op::ISGT(a, b) => {
                        analyzed_block.push_insn(Insn::If(Expr::gt(Expr::var(a.0), Expr::var(b.0))));
                    }
                    Op::ISEQV(a, b) => {
                        analyzed_block.push_insn(Insn::If(Expr::eq(Expr::var(a.0), Expr::var(b.0))));
                    }
                    Op::ISNEV(a, b) => {
                        analyzed_block.push_insn(Insn::If(Expr::ne(Expr::var(a.0), Expr::var(b.0))));
                    }
                    Op::ISEQS(a, b) => {
                        analyzed_block.push_insn(Insn::If(Expr::eq(Expr::var(a.0), Expr::str(b.0))));
                    }
                    Op::ISNES(a, b) => {
                        analyzed_block.push_insn(Insn::If(Expr::ne(Expr::var(a.0), Expr::str(b.0))));
                    }
                    Op::ISEQN(a, b) => {
                        analyzed_block.push_insn(Insn::If(Expr::eq(Expr::var(a.0), Expr::num(b.0))));
                    }
                    Op::ISNEN(a, b) => {
                        analyzed_block.push_insn(Insn::If(Expr::ne(Expr::var(a.0), Expr::num(b.0))));
                    }
                    Op::ISEQP(a, b) => {
                        analyzed_block
                            .push_insn(Insn::If(Expr::eq(Expr::var(a.0), Expr::primitive(b))));
                    }
                    Op::ISNEP(a, b) => {
                        analyzed_block
                            .push_insn(Insn::If(Expr::ne(Expr::var(a.0), Expr::primitive(b))));
                    }
                    // unary copy and test
                    Op::ISTC(_, _) => {}
                    Op::ISFC(_, _) => {}
                    Op::IST(a) => {
                        analyzed_block.push_insn(Insn::If(Expr::var(a.0)));
                    }
                    Op::ISF(_) => {}
                    Op::ISTYPE(_, _) => {}
                    Op::ISNUM(_, _) => {}
                    // unary
                    Op::MOV(a, b) => {
                        let var = self.var_for_slot(a.0, false);
                        analyzed_block.push_insn(Insn::set_var(var, Expr::var(b.0)));
                    }
                    Op::NOT(a, b) => {
                        let var = self.var_for_slot(a.0, false);
                        analyzed_block.push_insn(Insn::set_var(var, Expr::not(Expr::var(b.0))))
                    }
                    Op::UNM(a, b) => {
                        let var = self.var_for_slot(a.0, false);
                        analyzed_block.push_insn(Insn::set_var(var, Expr::minus(Expr::var(b.0))))
                    }
                    Op::LEN(a, b) => {
                        let var = self.var_for_slot(a.0, false);
                        analyzed_block.push_insn(Insn::set_var(var, Expr::len(Expr::var(b.0))))
                    }
                    // binary
                    Op::ADDVN(a, b, c) => {
                        let var = self.var_for_slot(a.0, false);
                        analyzed_block.push_insn(Insn::set_var(
                            var,
                            Expr::add(Expr::var(b.0), Expr::num(c.0)),
                        ));
                    }
                    Op::SUBVN(a, b, c) => {
                        let var = self.var_for_slot(a.0, false);
                        analyzed_block.push_insn(Insn::set_var(
                            var,
                            Expr::sub(Expr::var(b.0), Expr::num(c.0)),
                        ));
                    }
                    Op::MULVN(a, b, c) => {
                        let var = self.var_for_slot(a.0, false);
                        analyzed_block.push_insn(Insn::set_var(
                            var,
                            Expr::mul(Expr::var(b.0), Expr::num(c.0)),
                        ));
                    }
                    Op::DIVVN(a, b, c) => {
                        let var = self.var_for_slot(a.0, false);
                        analyzed_block.push_insn(Insn::set_var(
                            var,
                            Expr::div(Expr::var(b.0), Expr::num(c.0)),
                        ));
                    }
                    Op::MODVN(a, b, c) => {
                        let var = self.var_for_slot(a.0, false);
                        analyzed_block.push_insn(Insn::set_var(
                            var,
                            Expr::mod_(Expr::var(b.0), Expr::num(c.0)),
                        ));
                    }
                    Op::ADDNV(a, b, c) => {
                        let var = self.var_for_slot(a.0, false);
                        analyzed_block.push_insn(Insn::set_var(
                            var,
                            Expr::add(Expr::num(c.0), Expr::var(b.0)),
                        ));
                    }
                    Op::SUBNV(a, b, c) => {
                        let var = self.var_for_slot(a.0, false);
                        analyzed_block.push_insn(Insn::set_var(
                            var,
                            Expr::sub(Expr::num(c.0), Expr::var(b.0)),
                        ));
                    }
                    Op::MULNV(a, b, c) => {
                        let var = self.var_for_slot(a.0, false);
                        analyzed_block.push_insn(Insn::set_var(
                            var,
                            Expr::mul(Expr::num(c.0), Expr::var(b.0)),
                        ));
                    }
                    Op::DIVNV(a, b, c) => {
                        let var = self.var_for_slot(a.0, false);
                        analyzed_block.push_insn(Insn::set_var(
                            var,
                            Expr::div(Expr::num(c.0), Expr::var(b.0)),
                        ));
                    }
                    Op::MODNV(a, b, c) => {
                        let var = self.var_for_slot(a.0, false);
                        analyzed_block.push_insn(Insn::set_var(
                            var,
                            Expr::mod_(Expr::num(c.0), Expr::var(b.0)),
                        ));
                    }
                    Op::ADDVV(a, b, c) => {
                        let var = self.var_for_slot(a.0, false);
                        analyzed_block.push_insn(Insn::set_var(
                            var,
                            Expr::add(Expr::var(b.0), Expr::var(c.0)),
                        ));
                    }
                    Op::SUBVV(a, b, c) => {
                        let var = self.var_for_slot(a.0, false);
                        analyzed_block.push_insn(Insn::set_var(
                            var,
                            Expr::sub(Expr::var(b.0), Expr::var(c.0)),
                        ));
                    }
                    Op::MULVV(a, b, c) => {
                        let var = self.var_for_slot(a.0, false);
                        analyzed_block.push_insn(Insn::set_var(
                            var,
                            Expr::mul(Expr::var(b.0), Expr::var(c.0)),
                        ));
                    }
                    Op::DIVVV(a, b, c) => {
                        let var = self.var_for_slot(a.0, false);
                        analyzed_block.push_insn(Insn::set_var(
                            var,
                            Expr::div(Expr::var(b.0), Expr::var(c.0)),
                        ));
                    }
                    Op::MODVV(a, b, c) => {
                        let var = self.var_for_slot(a.0, false);
                        analyzed_block.push_insn(Insn::set_var(
                            var,
                            Expr::mod_(Expr::var(b.0), Expr::var(c.0)),
                        ));
                    }
                    Op::POW(a, b, c) => {
                        let var = self.var_for_slot(a.0, false);
                        analyzed_block.push_insn(Insn::set_var(
                            var,
                            Expr::pow(Expr::var(b.0), Expr::var(c.0)),
                        ));
                    }
                    Op::CAT(_, _, _) => {}
                    // constants
                    Op::KSTR(a, b) => {
                        let var = self.var_for_slot(a.0, false);
                        analyzed_block.push_insn(Insn::set_var(var, Expr::str(b.0)))
                    }
                    Op::KCDATA(a, b) => {
                        let var = self.var_for_slot(a.0, false);
                        analyzed_block.push_insn(Insn::set_var(var, Expr::cdata(b.0)))
                    }
                    Op::KSHORT(a, b) => {
                        let var = self.var_for_slot(a.0, false);
                        analyzed_block.push_insn(Insn::set_var(var, Expr::short(b.0)))
                    }
                    Op::KNUM(a, b) => {
                        let var = self.var_for_slot(a.0, false);
                        analyzed_block.push_insn(Insn::set_var(var, Expr::num(b.0)))
                    }
                    Op::KPRI(a, b) => {
                        let var = self.var_for_slot(a.0, false);
                        analyzed_block.push_insn(Insn::set_var(var, Expr::primitive(b)))
                    }
                    Op::KNIL(a, b) => {
                        if b.0 > a.0 {
                            let mut vars: Vec<Var> = Vec::with_capacity((b.0 - a.0) as usize + 1);

                            for i in a.0..=b.0 {
                                let var = self.var_for_slot(i, false);
                                vars.push(var);
                            }

                            analyzed_block.push_insn(Insn::SetVars(vars.into_boxed_slice(), Expr::nil()));
                        } else {
                            panic!("KNIL slot a <= b.");
                        }
                    }
                    // up values
                    Op::UGET(_, _) => {}
                    Op::USETV(_, _) => {}
                    Op::USETS(_, _) => {}
                    Op::USETN(_, _) => {}
                    Op::USETP(_, _) => {}
                    Op::UCLO(_, _) => {}
                    Op::FNEW(_, _) => {}
                    // tables
                    Op::TNEW(_, _) => {}
                    Op::TDUP(a, b) => {
                        let var = self.var_for_slot(a.0, false);
                        analyzed_block.push_insn(Insn::set_var(
                            var,
                            Expr::var(b.0),
                        ))
                    }
                    Op::GGET(a, b) => {
                        let var = self.var_for_slot(a.0, false);
                        analyzed_block.push_insn(Insn::set_var(
                            var,
                            Expr::table(Box::new(Expr::GlobalTable), Expr::str(b.0)),
                        ));
                    }
                    Op::GSET(_, _) => {}
                    Op::TGETV(a, b, c) => {
                        let var = self.var_for_slot(a.0, false);
                        analyzed_block.push_insn(Insn::set_var(
                            var,
                            Expr::table(Expr::var(b.0), Expr::var(c.0)),
                        ));
                    }
                    Op::TGETS(a, b, c) => {
                        let var = self.var_for_slot(a.0, false);
                        analyzed_block.push_insn(Insn::set_var(
                            var,
                            Expr::table(Expr::var(b.0), Expr::str(c.0)),
                        ));
                    }
                    Op::TGETB(a, b, c) => {
                        let var = self.var_for_slot(a.0, false);
                        analyzed_block.push_insn(Insn::set_var(
                            var,
                            Expr::table(Expr::var(b.0), Expr::lit(c.0)),
                        ));
                    }
                    Op::TGETR(a, b, c) => unimplemented!("TGETR"),
                    Op::TSETV(..) => {}
                    Op::TSETS(_, _, _) => {}
                    Op::TSETB(_, _, _) => {}
                    Op::TSETM(_, _) => {}
                    Op::TSETR(_, _, _) => {}
                    // call and vararg
                    Op::CALLM(_, _, _) => {}
                    Op::CALL(a, b, c) => {
                        let returns: Box<[Var]> = if b.0 != 0 {
                            let mut res: Vec<Var> = vec![];

                            for idx in a.0..(a.0 + b.0 as u16 - 1) {
                                let var = self.var_for_slot(idx, false);
                                res.push(var)
                            }

                            res.into_boxed_slice()
                        } else {
                            vec![].into_boxed_slice()
                        };

                        // function to call
                        let mut args: Vec<Expr> = vec![Expr::Var(Var(a.0))];

                        // todo: if c.0 == 0 ?

                        if c.0 > 1 {
                            for idx in (a.0 + 1)..=(a.0 + c.0 as u16 - 1) {
                                args.push(Expr::Var(Var(idx)))
                            }
                        }

                        analyzed_block.push_insn(Insn::Call(returns, args.into_boxed_slice()));
                    }
                    Op::CALLMT(_, _) => {}
                    Op::CALLT(_, _) => {}
                    Op::ITERC(_, _, _) => {}
                    Op::ITERN(_, _, _) => {}
                    Op::VARG(_, _, _) => {}
                    Op::ISNEXT(_, _) => {}
                    // returns
                    Op::RETM(a, b) => {}
                    Op::RET(a, b) => {
                        let mut res: Vec<Expr> = Vec::with_capacity((a.0 + b.0 as u16 - 2) as usize);

                        for idx in a.0..=(b.0 as u16 - 2) {
                            res.push(Expr::Var(Var(idx)))
                        }

                        analyzed_block.push_insn(Insn::Return(res.into_boxed_slice()));
                    }
                    Op::RET0(_, _) => {
                        analyzed_block.push_insn(Insn::Return(vec![].into_boxed_slice()));
                    }
                    Op::RET1(a, _) => {
                        analyzed_block
                            .push_insn(Insn::Return(vec![Expr::Var(Var(a.0))].into_boxed_slice()));
                    }
                    // loops and branches
                    Op::FORI(a, _) => {
                        let mut args: Vec<Expr> = Vec::with_capacity(3);

                        for idx in a.0..(a.0 + 3) {
                            args.push(Expr::Var(Var(idx)));
                        }

                        analyzed_block.push_insn(Insn::For(args.into_boxed_slice()));
                    }
                    Op::JFORI(_, _) => {}
                    Op::FORL(_, _) => {}
                    Op::IFORL(_, _) => {}
                    Op::JFORL(_, _) => {}
                    Op::ITERL(_, _) => {}
                    Op::IITERL(_, _) => {}
                    Op::JITERL(_, _) => {}
                    Op::LOOP(a, _) => {
                        analyzed_block.push_insn(Insn::While(Expr::var(a.0)));
                    }
                    Op::ILOOP(_, _) => {}
                    Op::JLOOP(_, _) => {}
                    Op::JMP(_, _) => {}
                    // function headers
                    Op::FUNCF(_) => {}
                    Op::IFUNCF(_) => {}
                    Op::JFUNCF(_, _) => {}
                    Op::FUNCV(_) => {}
                    Op::IFUNCV(_) => {}
                    Op::JFUNCV(_, _) => {}
                    Op::FUNCC(_) => {}
                    Op::FUNCCW(_) => {}
                }
            }
        }

        print_lifted_graph(&graph);

        Ok(graph)
    }
}

fn print_lifted_graph(graph: &Graph<Block, BranchKind>) {
    for (idx, block) in graph.iter_node_weights() {
        println!("Block({})", idx);
        for ins in block.iter_insn() {
            println!("{}", ins);
        }
    }
}
