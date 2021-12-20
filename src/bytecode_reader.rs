use std::io::Read;

use crate::error::DecompileError;
use crate::Graph;
use byteorder::{LittleEndian, ReadBytesExt};
use thiserror::Error;

use crate::resolver::{resolve_basic_blocks, Block};

// byte code header constants
pub const BC_HEAD1: u8 = 0x1b;
pub const BC_HEAD2: u8 = 0x4c;
pub const BC_HEAD3: u8 = 0x4a;
pub const BC_VERSION: u8 = 2;

// byte code header compatibility flags
pub const BC_F_BE: u32 = 0x01;
pub const BC_F_STRIP: u32 = 0x02;
pub const BC_F_FFI: u32 = 0x04;
pub const BC_F_FR2: u32 = 0x08;

pub const BC_F_KNOWN: u32 = BC_F_FR2 * 2 - 1;

// type codes for the global constants of a prototype and length for strings
pub const GC_TYPE_PROTO_CHILD: u32 = 0;
pub const GC_TYPE_TABLE: u32 = 1;
pub const GC_TYPE_I64: u32 = 2;
pub const GC_TYPE_U64: u32 = 3;
pub const GC_TYPE_COMPLEX: u32 = 4;
pub const GC_TYPE_STR: u32 = 5;

// type codes for the keys/values of a constant table
pub const TABLE_ENTRY_TYPE_NIL: u32 = 0;
pub const TABLE_ENTRY_TYPE_FALSE: u32 = 1;
pub const TABLE_ENTRY_TYPE_TRUE: u32 = 2;
pub const TABLE_ENTRY_TYPE_INT: u32 = 3;
pub const TABLE_ENTRY_TYPE_NUM: u32 = 4;
pub const TABLE_ENTRY_TYPE_STR: u32 = 5;

// lua prototype aka function
#[derive(Debug)]
pub struct ByteCodeProto {
    flags: u8,
    num_params: u8,
    frame_size: u8,
    size_up_values: u8,
    size_global_consts: u32,
    size_num_consts: u32,
    size_bc: u32,

    flow_graph: Graph<Block, ()>,

    bc_raw: Vec<u32>,
    up_values: Vec<u16>,
    global_consts: Vec<GlobalConst>,
    num_consts: Vec<NumConst>,
}

impl ByteCodeProto {
    pub fn new() -> Self {
        ByteCodeProto {
            flags: 0,
            num_params: 0,
            frame_size: 0,
            size_up_values: 0,
            size_global_consts: 0,
            size_num_consts: 0,
            size_bc: 0,
            flow_graph: Graph::new(),
            bc_raw: vec![],
            up_values: vec![],
            global_consts: vec![],
            num_consts: vec![],
        }
    }
}

#[derive(Debug)]
pub struct ByteCodeDump<'a> {
    magic: [u8; 3],
    version: u8,
    flags: u32,
    name: &'a str,

    prototypes: Vec<ByteCodeProto>,
}

impl ByteCodeDump<'_> {
    pub fn new() -> Self {
        ByteCodeDump {
            magic: [0, 0, 0],
            version: 0,
            flags: 0,
            name: "",
            prototypes: vec![],
        }
    }
}

#[derive(Debug)]
pub enum GlobalConst {
    ProtoChild,
    Table(ConstTable),
    Num(u32, u32),
    Complex(u32, u32, u32, u32),
    Str(String),
}

#[derive(Debug)]
pub enum NumConst {
    Int(u32),
    Num(u32, u32),
}

#[derive(Debug)]
pub struct ConstTable {
    array: Vec<ConstTableVal>,
    hash: Vec<(ConstTableVal, ConstTableVal)>,
}

#[derive(Debug)]
pub enum ConstTableVal {
    Nil,
    True,
    False,
    Int(u32),
    Num(u32, u32),
    String(String),
}

pub fn read_uleb128<T: Read>(data: &mut T) -> Result<u32, DecompileError> {
    let mut result = 0u32;
    let mut shift = 0u8;
    loop {
        if shift > 28 {
            return Err(DecompileError::InvalidULeb128);
        }

        let mut arr = [0u8; 1];
        data.read_exact(&mut arr)?;

        result |= ((arr[0] as u32 & 0x7f) << shift) as u32;
        if arr[0] & 0x80 == 0 {
            break;
        }
        shift += 7;
    }

    Ok(result)
}

// read top 32 bits of 33 bit ULEB128 value from buffer
pub fn read_uleb128_33<T: Read>(data: &mut T) -> Result<(u32, u8), DecompileError> {
    let mut arr = [0u8; 1];
    data.read_exact(&mut arr)?;

    // for is num check
    let tmp = arr[0];

    let mut v = arr[0] as u32 >> 1;

    if v >= 0x40 {
        let mut sh = 6u32;
        v &= 0x3f;

        loop {
            data.read_exact(&mut arr)?;
            v |= ((arr[0] & 0x7f) as u32) << sh;

            if arr[0] < 0x80 {
                break;
            }

            sh += 7;
        }
    }

    Ok((v, tmp))
}

pub fn read_header<T: ReadBytesExt>(
    file: &mut T,
    bc_dump: &mut ByteCodeDump,
) -> Result<(), DecompileError> {
    let mut arr = [0u8; 4];
    file.read_exact(&mut arr)?;

    if arr[0] != BC_HEAD1 || arr[1] != BC_HEAD2 || arr[2] != BC_HEAD3 {
        return Err(DecompileError::InvalidHeaderBytes(
            "Invalid byte code file magic.",
        ));
    }

    if arr[3] != BC_VERSION {
        return Err(DecompileError::InvalidHeaderBytes(
            "Invalid byte code version.",
        ));
    }

    let flags = read_uleb128(file)?;
    if flags & !BC_F_KNOWN != 0 || flags & BC_F_FFI == 1 {
        return Err(DecompileError::InvalidHeaderBytes("Invalid header flags."));
    }

    // todo: unknown logic with chunk string
    // if flags & BC_F_STRIP == 1 {
    //     // ...
    //     Ok(0)
    // } else {
    //     let len = read_uleb128(file)?;
    //     println!("len {:x}", len);
    //     Ok(len)
    // }

    bc_dump.flags = flags;
    bc_dump.magic = [arr[0], arr[1], arr[2]];
    bc_dump.version = arr[3];

    Ok(())
}

pub fn read_prototype<T: Read>(
    data: &mut T,
    bc_proto: &mut ByteCodeProto,
) -> Result<(), DecompileError> {
    // read prototype header
    let mut arr = [0u8; 4];
    data.read_exact(&mut arr)?;

    bc_proto.flags = arr[0];
    bc_proto.num_params = arr[1];
    bc_proto.frame_size = arr[2];
    bc_proto.size_up_values = arr[3];

    bc_proto.size_global_consts = read_uleb128(data)?;
    bc_proto.size_num_consts = read_uleb128(data)?;
    bc_proto.size_bc = read_uleb128(data)?; // byte code instruction count (+ 1 for repr)

    println!("flags 0x{:x} num_params 0x{:x} frame_size 0x{:x} size_uv 0x{:x} size_kgc 0x{:x} size_kn 0x{:x} size_bc 0x{:x}",
             bc_proto.flags, bc_proto.num_params, bc_proto.frame_size, bc_proto.size_up_values, bc_proto.size_global_consts, bc_proto.size_num_consts, bc_proto.size_bc);

    // todo: check debug flags and collect debug data if it exists

    // read bytecode instructions and up values
    read_prototype_bytecode(data, bc_proto)?;
    read_prototype_up_values(data, bc_proto)?;

    // read consts
    read_prototype_global_constants(data, bc_proto)?;
    read_prototype_num_constants(data, bc_proto)?;

    Ok(())
}

pub fn read_prototype_up_values<T: Read>(
    data: &mut T,
    bc_proto: &mut ByteCodeProto,
) -> Result<(), DecompileError> {
    if bc_proto.size_up_values > 0 {
        let mut uv_buff: Vec<u16> = Vec::with_capacity(bc_proto.size_up_values as usize);

        unsafe { uv_buff.set_len(bc_proto.size_up_values as usize) }
        data.read_u16_into::<LittleEndian>(&mut uv_buff[..])?;

        bc_proto.up_values = uv_buff;
    }

    Ok(())
}

pub fn read_prototype_bytecode<T: Read>(
    data: &mut T,
    bc_proto: &mut ByteCodeProto,
) -> Result<(), DecompileError> {
    if bc_proto.size_bc > 0 {
        let mut ins_buff: Vec<u32> = Vec::with_capacity(bc_proto.size_bc as usize);

        unsafe {
            ins_buff.set_len(bc_proto.size_bc as usize);
        }
        data.read_u32_into::<LittleEndian>(&mut ins_buff[..])?;

        bc_proto.bc_raw = ins_buff;
    }

    // analyze control flow graph
    bc_proto.flow_graph = resolve_basic_blocks(&bc_proto.bc_raw[..])?;

    Ok(())
}

pub fn read_prototype_const_table<T: Read>(
    data: &mut T,
    bc_proto: &mut ByteCodeProto,
) -> Result<(), DecompileError> {
    // read template table
    let n_array = read_uleb128(data)?;
    let n_hash = read_uleb128(data)?;

    let mut ktab = ConstTable {
        array: Vec::with_capacity(n_array as usize),
        hash: Vec::with_capacity(n_hash as usize),
    };

    if n_array > 0 {
        for _ in 0..n_array {
            ktab.array.push(read_prototype_const_table_val(data)?);
        }
    }

    if n_hash > 0 {
        for _ in 0..n_hash {
            ktab.hash.push((
                read_prototype_const_table_val(data)?,
                read_prototype_const_table_val(data)?,
            ));
        }
    }

    bc_proto.global_consts.push(GlobalConst::Table(ktab));

    Ok(())
}

pub fn read_prototype_global_constants<T: Read>(
    data: &mut T,
    bc_proto: &mut ByteCodeProto,
) -> Result<(), DecompileError> {
    for _ in 0..bc_proto.size_global_consts {
        // get gc type
        let tp = read_uleb128(data)?;

        match tp {
            GC_TYPE_PROTO_CHILD => bc_proto.global_consts.push(GlobalConst::ProtoChild),
            GC_TYPE_TABLE => read_prototype_const_table(data, bc_proto)?,
            GC_TYPE_I64 | GC_TYPE_U64 => bc_proto
                .global_consts
                .push(GlobalConst::Num(read_uleb128(data)?, read_uleb128(data)?)),
            GC_TYPE_COMPLEX => bc_proto.global_consts.push(GlobalConst::Complex(
                read_uleb128(data)?,
                read_uleb128(data)?,
                read_uleb128(data)?,
                read_uleb128(data)?,
            )),
            _ => {
                // const string type
                let len = tp - GC_TYPE_STR;
                let mut str = vec![0u8; len as usize];
                data.read_exact(&mut str)?;

                bc_proto.global_consts.push(GlobalConst::Str(
                    String::from_utf8_lossy(str.as_slice()).to_string(),
                ));
            }
        }
    }

    Ok(())
}

// read single key/value of a template table
pub fn read_prototype_const_table_val<T: Read>(
    data: &mut T,
) -> Result<ConstTableVal, DecompileError> {
    let tp = read_uleb128(data)?;

    match tp {
        TABLE_ENTRY_TYPE_NIL => Ok(ConstTableVal::Nil),
        TABLE_ENTRY_TYPE_FALSE => Ok(ConstTableVal::False),
        TABLE_ENTRY_TYPE_TRUE => Ok(ConstTableVal::True),
        TABLE_ENTRY_TYPE_INT => Ok(ConstTableVal::Int(read_uleb128(data)?)),
        TABLE_ENTRY_TYPE_NUM => Ok(ConstTableVal::Num(read_uleb128(data)?, read_uleb128(data)?)),
        _ => {
            let len = tp - TABLE_ENTRY_TYPE_STR;
            let mut str = vec![0u8; len as usize];
            data.read_exact(&mut str)?;

            Ok(ConstTableVal::String(
                String::from_utf8_lossy(str.as_slice()).to_string(),
            ))
        }
    }
}

pub fn read_prototype_num_constants<T: Read>(
    data: &mut T,
    bc_proto: &mut ByteCodeProto,
) -> Result<(), DecompileError> {
    for _ in 0..bc_proto.size_num_consts {
        let (lo, first) = read_uleb128_33(data)?;
        if first & 1 == 1 {
            let hi = read_uleb128(data)?;
            bc_proto.num_consts.push(NumConst::Num(lo, hi));
        } else {
            bc_proto.num_consts.push(NumConst::Int(lo));
        }
    }

    Ok(())
}

pub fn read_bytecode_dump<T: Read>(data: &mut T) -> Result<ByteCodeDump, DecompileError> {
    let mut bc_dump = ByteCodeDump::new();

    // read byte code header
    read_header(data, &mut bc_dump)?;

    loop {
        // read next prototype len
        if let Ok(proto_len) = read_uleb128(data) {
            if proto_len == 0 {
                continue;
            }

            println!("Prototype len 0x{:x}", proto_len);

            // read prototype data
            let mut proto_data: Vec<u8> = Vec::with_capacity(proto_len as usize);

            unsafe {
                proto_data.set_len(proto_len as usize);
            }
            data.read_exact(&mut proto_data)?;

            let mut proto_data = proto_data.as_slice();
            let mut proto = ByteCodeProto::new();

            read_prototype(&mut proto_data, &mut proto)?;

            println!("Prototype object: {:?}", proto);
            bc_dump.prototypes.push(proto);
        } else {
            break;
        }
    }

    Ok(bc_dump)
}
