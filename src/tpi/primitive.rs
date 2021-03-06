// Copyright 2017 pdb Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use common::*;
use super::data::TypeData;

// References for primitive types:
//
// cvinfo.h provides an enumeration:
//   https://github.com/Microsoft/microsoft-pdb/blob/082c5290e5aff028ae84e43affa8be717aa7af73/include/cvinfo.h#L328-L750
//
// pdbparse.cpp describes them as strings:
//   https://github.com/Microsoft/microsoft-pdb/blob/082c5290e5aff028ae84e43affa8be717aa7af73/pdbdump/pdbdump.cpp#L1896-L1974
//
// The most obscure: MSDN Library October 2001 Disk 2 contains a \MSDN\specs.chm file which contains
// html\S66CD.HTM which actually documents the *format* of the primitive type descriptors rather
// than just listing them. TypeData::Primitive is designed to model the orthogonal information
// encoded into the bits of the TypeIndex rather than exploding the matrix like the reference
// implementations.

#[derive(Debug,Copy,Clone,PartialEq,Eq)]
pub enum PrimitiveType {
    Void,

    Char,

    UChar,

    /// "Really a char"
    RChar,

    //// Wide characters, i.e. 16 bits
    WChar,

    /// "Really a 16-bit char"
    RChar16,

    /// "Really a 32-bit char"
    RChar32,

    /// Signed 8-bit integer
    I8,

    /// Unsigned 8-bit integer
    U8,

    /// Signed 16-bit integer
    I16,

    /// Unsigned 16-bit integer
    U16,

    /// Signed 32-bit integer
    I32,

    /// Unsigned 32-bit inteer
    U32,

    /// Signed 64-bit integer
    I64,

    /// Unsigned 64-bit integer
    U64,

    /// Signed 128-bit integer
    I128,

    /// Unsigned 128-bit integer
    U128,

    /// 16-bit floating point
    F16,

    /// 32-bit floating point
    F32,

    /// 32-bit partial precision floating point
    F32PP,

    /// 48-bit floating point
    F48,

    /// 64-bit floating point
    F64,

    /// 80-bit floating point
    F80,

    /// 128-bit floating point
    F128,

    /// 32-bit complex number
    Complex32,

    /// 64-bit complex number
    Complex64,

    /// 80-bit complex number
    Complex80,

    /// 128-bit complex number
    Complex128,

    /// 8-bit boolean value
    Bool8,

    /// 16-bit boolean value
    Bool16,

    /// 32-bit boolean value
    Bool32,

    /// 16-bit boolean value
    Bool64,
}

#[derive(Debug,Copy,Clone,PartialEq,Eq)]
pub enum Indirection {
    None,

    /// 16-bit ("near") pointer
    Pointer16,

    /// 16:16 far pointer
    FarPointer1616,

    /// 16:16 huge pointer
    HugePointer1616,

    /// 32-bit pointer
    Pointer32,

    /// 48-bit 16:32 pointer
    Pointer1632,

    /// 64-bit pointer
    Pointer64,
}

pub fn type_data_for_primitive(index: TypeIndex) -> Result<TypeData<'static>> {
    // https://github.com/Microsoft/microsoft-pdb/blob/082c5290e5aff028ae84e43affa8be717aa7af73/include/cvinfo.h#L326-L750

    // primitives live under 0x1000, and we should never reach here for non-primitive indexes
    assert!(index < 0x1000);

    // indirection is stored in these bits
    let indirection = match index & 0xf00 {
        0x000 => Indirection::None,
        0x100 => Indirection::Pointer16,
        0x200 => Indirection::FarPointer1616,
        0x300 => Indirection::FarPointer1616,
        0x400 => Indirection::Pointer32,
        0x500 => Indirection::Pointer1632,
        0x600 => Indirection::Pointer64,
        _ => { return Err(Error::TypeNotFound(index)); }
    };

    // primitive types are stored in the lowest octet
    // this groups "short" and "16-bit integer" together, but... right? *scratches head*
    let primitive_type = match index & 0xff {
        0x03 => PrimitiveType::Void,

        0x10 => PrimitiveType::Char,
        0x20 => PrimitiveType::UChar,
        0x68 => PrimitiveType::I8,
        0x69 => PrimitiveType::U8,

        0x70 => PrimitiveType::RChar,
        0x71 => PrimitiveType::WChar,
        0x7a => PrimitiveType::RChar16,
        0x7b => PrimitiveType::RChar32,

        0x11 => PrimitiveType::I16,
        0x21 => PrimitiveType::U16,
        0x72 => PrimitiveType::I16,
        0x73 => PrimitiveType::U16,

        0x12 => PrimitiveType::I32,
        0x22 => PrimitiveType::U32,
        0x74 => PrimitiveType::I32,
        0x75 => PrimitiveType::U32,

        0x13 => PrimitiveType::I64,
        0x23 => PrimitiveType::U64,
        0x76 => PrimitiveType::I64,
        0x77 => PrimitiveType::U64,

        0x14 => PrimitiveType::I128,
        0x24 => PrimitiveType::U128,
        0x78 => PrimitiveType::I128,
        0x79 => PrimitiveType::U128,

        0x46 => PrimitiveType::F16,
        0x40 => PrimitiveType::F32,
        0x45 => PrimitiveType::F32PP,
        0x44 => PrimitiveType::F48,
        0x41 => PrimitiveType::F64,
        0x42 => PrimitiveType::F80,
        0x43 => PrimitiveType::F128,

        0x50 => PrimitiveType::Complex32,
        0x51 => PrimitiveType::Complex64,
        0x52 => PrimitiveType::Complex80,
        0x53 => PrimitiveType::Complex128,

        0x30 => PrimitiveType::Bool8,
        0x31 => PrimitiveType::Bool16,
        0x32 => PrimitiveType::Bool32,
        0x33 => PrimitiveType::Bool64,

        _ => { return Err(Error::TypeNotFound(index)); }
    };

    Ok(TypeData::Primitive{
        indirection: indirection,
        primitive_type: primitive_type,
    })
}
