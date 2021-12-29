use crate::DecompileError;
use thiserror::Error;

use crate::op::Op;

#[inline(always)]
fn get_op(ins: u32) -> u8 {
    (ins & 0xff) as u8
}

#[inline(always)]
fn get_a<R: From<u8>>(ins: u32) -> R {
    R::from(((ins >> 8) & 0xff) as u8)
}

#[inline(always)]
fn get_b<R: From<u8>>(ins: u32) -> R {
    (((ins >> 16) & 0xff) as u8).into()
}

#[inline(always)]
fn get_c<R: From<u8>>(ins: u32) -> R {
    ((ins >> 24) as u8).into()
}

#[inline(always)]
fn get_d<R: From<u16>>(ins: u32) -> R {
    R::from((ins >> 16) as u16)
}

pub fn disasm(ins_raw: u32) -> Result<Op, DecompileError> {
    Ok(match get_op(ins_raw) {
        0x00 => Op::ISLT(get_a(ins_raw), get_d(ins_raw)),
        0x01 => Op::ISGE(get_a(ins_raw), get_d(ins_raw)),
        0x02 => Op::ISLE(get_a(ins_raw), get_d(ins_raw)),
        0x03 => Op::ISGT(get_a(ins_raw), get_d(ins_raw)),
        0x04 => Op::ISEQV(get_a(ins_raw), get_d(ins_raw)),
        0x05 => Op::ISNEV(get_a(ins_raw), get_d(ins_raw)),
        0x06 => Op::ISEQS(get_a(ins_raw), get_d(ins_raw)),
        0x07 => Op::ISNES(get_a(ins_raw), get_d(ins_raw)),
        0x08 => Op::ISEQN(get_a(ins_raw), get_d(ins_raw)),
        0x09 => Op::ISNEN(get_a(ins_raw), get_d(ins_raw)),
        0x0a => Op::ISEQP(get_a(ins_raw), get_d(ins_raw)),
        0x0b => Op::ISNEP(get_a(ins_raw), get_d(ins_raw)),
        0x0c => Op::ISTC(get_a(ins_raw), get_d(ins_raw)),
        0x0d => Op::ISFC(get_a(ins_raw), get_d(ins_raw)),
        0x0e => Op::IST(get_d(ins_raw)),
        0x0f => Op::ISF(get_d(ins_raw)),
        0x10 => Op::ISTYPE(get_a(ins_raw), get_d(ins_raw)),
        0x11 => Op::ISNUM(get_a(ins_raw), get_d(ins_raw)),
        0x12 => Op::MOV(get_a(ins_raw), get_d(ins_raw)),
        0x13 => Op::NOT(get_a(ins_raw), get_d(ins_raw)),
        0x14 => Op::UNM(get_a(ins_raw), get_d(ins_raw)),
        0x15 => Op::LEN(get_a(ins_raw), get_d(ins_raw)),
        0x16 => Op::ADDVN(get_a(ins_raw), get_b(ins_raw), get_c(ins_raw)),
        0x17 => Op::SUBVN(get_a(ins_raw), get_b(ins_raw), get_c(ins_raw)),
        0x18 => Op::MULVN(get_a(ins_raw), get_b(ins_raw), get_c(ins_raw)),
        0x19 => Op::DIVVN(get_a(ins_raw), get_b(ins_raw), get_c(ins_raw)),
        0x1a => Op::MODVN(get_a(ins_raw), get_b(ins_raw), get_c(ins_raw)),
        0x1b => Op::ADDNV(get_a(ins_raw), get_b(ins_raw), get_c(ins_raw)),
        0x1c => Op::SUBNV(get_a(ins_raw), get_b(ins_raw), get_c(ins_raw)),
        0x1d => Op::MULNV(get_a(ins_raw), get_b(ins_raw), get_c(ins_raw)),
        0x1e => Op::DIVNV(get_a(ins_raw), get_b(ins_raw), get_c(ins_raw)),
        0x1f => Op::MODNV(get_a(ins_raw), get_b(ins_raw), get_c(ins_raw)),
        0x20 => Op::ADDVV(get_a(ins_raw), get_b(ins_raw), get_c(ins_raw)),
        0x21 => Op::SUBVV(get_a(ins_raw), get_b(ins_raw), get_c(ins_raw)),
        0x22 => Op::MULVV(get_a(ins_raw), get_b(ins_raw), get_c(ins_raw)),
        0x23 => Op::DIVVV(get_a(ins_raw), get_b(ins_raw), get_c(ins_raw)),
        0x24 => Op::MODVV(get_a(ins_raw), get_b(ins_raw), get_c(ins_raw)),
        0x25 => Op::POW(get_a(ins_raw), get_b(ins_raw), get_c(ins_raw)),
        0x26 => Op::CAT(get_a(ins_raw), get_b(ins_raw), get_c(ins_raw)),
        0x27 => Op::KSTR(get_a(ins_raw), get_d(ins_raw)),
        0x28 => Op::KCDATA(get_a(ins_raw), get_d(ins_raw)),
        0x29 => Op::KSHORT(get_a(ins_raw), get_d(ins_raw)),
        0x2a => Op::KNUM(get_a(ins_raw), get_d(ins_raw)),
        0x2b => Op::KPRI(get_a(ins_raw), get_d(ins_raw)),
        0x2c => Op::KNIL(get_a(ins_raw), get_d(ins_raw)),
        0x2d => Op::UGET(get_a(ins_raw), get_d(ins_raw)),
        0x2e => Op::USETV(get_a(ins_raw), get_d(ins_raw)),
        0x2f => Op::USETS(get_a(ins_raw), get_d(ins_raw)),
        0x30 => Op::USETN(get_a(ins_raw), get_d(ins_raw)),
        0x31 => Op::USETP(get_a(ins_raw), get_d(ins_raw)),
        0x32 => Op::UCLO(get_a(ins_raw), get_d(ins_raw)),
        0x33 => Op::FNEW(get_a(ins_raw), get_d(ins_raw)),
        0x34 => Op::TNEW(get_a(ins_raw), get_d(ins_raw)),
        0x35 => Op::TDUP(get_a(ins_raw), get_d(ins_raw)),
        0x36 => Op::GGET(get_a(ins_raw), get_d(ins_raw)),
        0x37 => Op::GSET(get_a(ins_raw), get_d(ins_raw)),
        0x38 => Op::TGETV(get_a(ins_raw), get_b(ins_raw), get_c(ins_raw)),
        0x39 => Op::TGETS(get_a(ins_raw), get_b(ins_raw), get_c(ins_raw)),
        0x3a => Op::TGETB(get_a(ins_raw), get_b(ins_raw), get_c(ins_raw)),
        0x3b => Op::TGETR(get_a(ins_raw), get_b(ins_raw), get_c(ins_raw)),
        0x3c => Op::TSETV(get_a(ins_raw), get_b(ins_raw), get_c(ins_raw)),
        0x3d => Op::TSETS(get_a(ins_raw), get_b(ins_raw), get_c(ins_raw)),
        0x3e => Op::TSETB(get_a(ins_raw), get_b(ins_raw), get_c(ins_raw)),
        0x3f => Op::TSETM(get_a(ins_raw), get_d(ins_raw)),
        0x40 => Op::TSETR(get_a(ins_raw), get_b(ins_raw), get_c(ins_raw)),
        0x41 => Op::CALLM(get_a(ins_raw), get_b(ins_raw), get_c(ins_raw)),
        0x42 => Op::CALL(get_a(ins_raw), get_b(ins_raw), get_c(ins_raw)),
        0x43 => Op::CALLMT(get_a(ins_raw), get_d(ins_raw)),
        0x44 => Op::CALLT(get_a(ins_raw), get_d(ins_raw)),
        0x45 => Op::ITERC(get_a(ins_raw), get_b(ins_raw), get_c(ins_raw)),
        0x46 => Op::ITERN(get_a(ins_raw), get_b(ins_raw), get_c(ins_raw)),
        0x47 => Op::VARG(get_a(ins_raw), get_b(ins_raw), get_c(ins_raw)),
        0x48 => Op::ISNEXT(get_a(ins_raw), get_d(ins_raw)),
        0x49 => Op::RETM(get_a(ins_raw), get_d(ins_raw)),
        0x4a => Op::RET(get_a(ins_raw), get_d(ins_raw)),
        0x4b => Op::RET0(get_a(ins_raw), get_d(ins_raw)),
        0x4c => Op::RET1(get_a(ins_raw), get_d(ins_raw)),
        0x4d => Op::FORI(get_a(ins_raw), get_d(ins_raw)),
        0x4e => Op::JFORI(get_a(ins_raw), get_d(ins_raw)),
        0x4f => Op::FORL(get_a(ins_raw), get_d(ins_raw)),
        0x50 => Op::IFORL(get_a(ins_raw), get_d(ins_raw)),
        0x51 => Op::JFORL(get_a(ins_raw), get_d(ins_raw)),
        0x52 => Op::ITERL(get_a(ins_raw), get_d(ins_raw)),
        0x53 => Op::IITERL(get_a(ins_raw), get_d(ins_raw)),
        0x54 => Op::JITERL(get_a(ins_raw), get_d(ins_raw)),
        0x55 => Op::LOOP(get_a(ins_raw), get_d(ins_raw)),
        0x56 => Op::ILOOP(get_a(ins_raw), get_d(ins_raw)),
        0x57 => Op::JLOOP(get_a(ins_raw), get_d(ins_raw)),
        0x58 => Op::JMP(get_a(ins_raw), get_d(ins_raw)),
        0x59 => Op::FUNCF(get_a(ins_raw)),
        0x5a => Op::IFUNCF(get_a(ins_raw)),
        0x5b => Op::JFUNCF(get_a(ins_raw), get_d(ins_raw)),
        0x5c => Op::FUNCV(get_a(ins_raw)),
        0x5d => Op::IFUNCV(get_a(ins_raw)),
        0x5e => Op::JFUNCV(get_a(ins_raw), get_d(ins_raw)),
        0x5f => Op::FUNCC(get_a(ins_raw)),
        0x60 => Op::FUNCCW(get_a(ins_raw)),
        _ => panic!("Unknown bytecode instruction opcode."),
    })
}
