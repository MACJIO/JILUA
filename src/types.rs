#[derive(Debug)]
#[repr(transparent)]
pub struct Var(pub u16);

impl From<u8> for Var {
    #[inline(always)]
    fn from(val: u8) -> Self {
        Var(val as u16)
    }
}

impl From<u16> for Var {
    #[inline(always)]
    fn from(val: u16) -> Self {
        Var(val)
    }
}

#[derive(Debug)]
#[repr(transparent)]
pub struct Dst(pub u16);

impl From<u8> for Dst {
    #[inline(always)]
    fn from(val: u8) -> Self {
        Dst(val as u16)
    }
}

impl From<u16> for Dst {
    #[inline(always)]
    fn from(val: u16) -> Self {
        Dst(val)
    }
}

#[derive(Debug)]
#[repr(transparent)]
pub struct Base(pub u16);

impl From<u8> for Base {
    #[inline(always)]
    fn from(val: u8) -> Self {
        Base(val as u16)
    }
}

impl From<u16> for Base {
    #[inline(always)]
    fn from(val: u16) -> Self {
        Base(val)
    }
}

#[derive(Debug)]
#[repr(transparent)]
pub struct RBase(pub u16);

impl From<u8> for RBase {
    #[inline(always)]
    fn from(val: u8) -> Self {
        RBase(val as u16)
    }
}

impl From<u16> for RBase {
    #[inline(always)]
    fn from(val: u16) -> Self {
        RBase(val)
    }
}

#[derive(Debug)]
#[repr(transparent)]
pub struct UV(pub u16);

impl From<u8> for UV {
    #[inline(always)]
    fn from(val: u8) -> Self {
        UV(val as u16)
    }
}

impl From<u16> for UV {
    #[inline(always)]
    fn from(val: u16) -> Self {
        UV(val)
    }
}

#[derive(Debug)]
#[repr(transparent)]
pub struct Lit(pub u8);

impl From<u8> for Lit {
    #[inline(always)]
    fn from(val: u8) -> Self {
        Lit(val)
    }
}

impl From<u16> for Lit {
    fn from(val: u16) -> Self {
        Self((val & 0xff) as u8)
    }
}

#[derive(Debug)]
#[repr(transparent)]
pub struct LitS(pub i8);

impl From<u8> for LitS {
    #[inline(always)]
    fn from(val: u8) -> Self {
        LitS(unsafe { std::mem::transmute(val) })
    }
}

impl From<u16> for LitS {
    fn from(val: u16) -> Self {
        ((val & 0xff) as u8).into()
    }
}

#[derive(Debug)]
#[repr(transparent)]
pub struct Str(pub u16);

impl From<u8> for Str {
    #[inline(always)]
    fn from(val: u8) -> Self {
        Str(val as u16)
    }
}

impl From<u16> for Str {
    #[inline(always)]
    fn from(val: u16) -> Self {
        Str(val)
    }
}

#[derive(Debug)]
#[repr(transparent)]
pub struct Num(pub u16);

impl From<u8> for Num {
    #[inline(always)]
    fn from(val: u8) -> Self {
        Num(val as u16)
    }
}

impl From<u16> for Num {
    #[inline(always)]
    fn from(val: u16) -> Self {
        Num(val)
    }
}

#[derive(Debug)]
#[repr(u8)]
pub enum Pri {
    Nil = 0,
    False = 1,
    True = 2,
}

impl From<u8> for Pri {
    fn from(val: u8) -> Self {
        match val {
            0 => Pri::Nil,
            1 => Pri::False,
            2 => Pri::True,
            _ => panic!("Unexpected value"),
        }
    }
}

impl From<u16> for Pri {
    fn from(val: u16) -> Self {
        match val as u8 {
            0 => Pri::Nil,
            1 => Pri::False,
            2 => Pri::True,
            _ => panic!("Unexpected value"),
        }
    }
}

#[derive(Debug)]
#[repr(transparent)]
pub struct Tab(pub u16);

impl From<u8> for Tab {
    #[inline(always)]
    fn from(val: u8) -> Self {
        Tab(val as u16)
    }
}

impl From<u16> for Tab {
    #[inline(always)]
    fn from(val: u16) -> Self {
        Tab(val)
    }
}

#[derive(Debug)]
#[repr(transparent)]
pub struct Func(pub u16);

impl From<u8> for Func {
    #[inline(always)]
    fn from(val: u8) -> Self {
        Func(val as u16)
    }
}

impl From<u16> for Func {
    #[inline(always)]
    fn from(val: u16) -> Self {
        Func(val)
    }
}

#[derive(Debug)]
#[repr(transparent)]
pub struct CData(pub u16);

impl From<u8> for CData {
    #[inline(always)]
    fn from(val: u8) -> Self {
        CData(val as u16)
    }
}

impl From<u16> for CData {
    #[inline(always)]
    fn from(val: u16) -> Self {
        CData(val)
    }
}

#[derive(Debug)]
#[repr(transparent)]
pub struct Jump(pub i16);

impl From<u8> for Jump {
    #[inline(always)]
    fn from(val: u8) -> Self {
        Jump((0x8000 - val as u16) as i16)
    }
}

impl From<u16> for Jump {
    #[inline(always)]
    fn from(val: u16) -> Self {
        if val >= 0x8000 {
            Jump((val - 0x8000) as i16)
        } else {
            Jump(-((0x8000 - val) as i16))
        }
    }
}
