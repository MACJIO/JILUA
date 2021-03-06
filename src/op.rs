use crate::types::*;

#[derive(Debug)]
pub enum Op {
    // Comparison ops
    ISLT(Var, Var),
    ISGE(Var, Var),
    ISLE(Var, Var),
    ISGT(Var, Var),
    ISEQV(Var, Var),
    ISNEV(Var, Var),
    ISEQS(Var, Str),
    ISNES(Var, Str),
    ISEQN(Var, Num),
    ISNEN(Var, Num),
    ISEQP(Var, Pri),
    ISNEP(Var, Pri),
    // Unary Test and Copy ops
    ISTC(Dst, Var),
    ISFC(Dst, Var),
    IST(Var),
    ISF(Var),
    ISTYPE(Var, Lit),
    ISNUM(Var, Lit),
    // Unary ops
    MOV(Dst, Var),
    NOT(Dst, Var),
    UNM(Dst, Var),
    LEN(Dst, Var),
    // Binary ops
    ADDVN(Dst, Var, Num),
    SUBVN(Dst, Var, Num),
    MULVN(Dst, Var, Num),
    DIVVN(Dst, Var, Num),
    MODVN(Dst, Var, Num),
    ADDNV(Dst, Var, Num),
    SUBNV(Dst, Var, Num),
    MULNV(Dst, Var, Num),
    DIVNV(Dst, Var, Num),
    MODNV(Dst, Var, Num),
    ADDVV(Dst, Var, Var),
    SUBVV(Dst, Var, Var),
    MULVV(Dst, Var, Var),
    DIVVV(Dst, Var, Var),
    MODVV(Dst, Var, Var),
    POW(Dst, Var, Var),
    CAT(Dst, RBase, RBase),
    // Constant ops
    KSTR(Dst, Str),
    KCDATA(Dst, CData),
    KSHORT(Dst, LitS),
    KNUM(Dst, Num),
    KPRI(Dst, Pri),
    KNIL(Base, Base),
    // Upvalue and Function ops
    UGET(Dst, UV),
    USETV(UV, Var),
    USETS(UV, Str),
    USETN(UV, Num),
    USETP(UV, Pri),
    UCLO(RBase, Jump),
    FNEW(Dst, Func),
    // Table ops
    TNEW(Dst, Lit),
    TDUP(Dst, Tab),
    GGET(Dst, Str),
    GSET(Var, Str),
    TGETV(Dst, Var, Var),
    TGETS(Dst, Var, Str),
    TGETB(Dst, Var, Lit),
    TGETR(Dst, Var, Var),
    TSETV(Var, Var, Var),
    TSETS(Var, Var, Str),
    TSETB(Var, Var, Lit),
    TSETM(Base, Num),
    TSETR(Var, Var, Var),
    // Calls and Vararg Handling
    CALLM(Base, Lit, Lit),
    CALL(Base, Lit, Lit),
    CALLMT(Base, Lit),
    CALLT(Base, Lit),
    ITERC(Base, Lit, Lit),
    ITERN(Base, Lit, Lit),
    VARG(Base, Lit, Lit),
    ISNEXT(Base, Jump),
    // Returns
    RETM(Base, Lit),
    RET(RBase, Lit),
    RET0(RBase, Lit),
    RET1(RBase, Lit),
    // Loops and branches
    FORI(Base, Jump),
    JFORI(Base, Jump),

    FORL(Base, Jump),
    IFORL(Base, Jump),
    JFORL(Base, Lit),

    ITERL(Base, Jump),
    IITERL(Base, Jump),
    JITERL(Base, Lit),

    LOOP(RBase, Jump),
    ILOOP(RBase, Jump),
    JLOOP(RBase, Lit),

    JMP(RBase, Jump),
    // Function headers
    FUNCF(RBase),
    IFUNCF(RBase),
    JFUNCF(RBase, Lit),
    FUNCV(RBase),
    IFUNCV(RBase),
    JFUNCV(RBase, Lit),
    FUNCC(RBase),
    FUNCCW(RBase),
}
