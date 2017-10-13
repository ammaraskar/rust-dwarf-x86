extern crate dwarf;

use enum_primitive::FromPrimitive;
use dwarf::die;

use std::iter::FromIterator;
use std::ffi::CString;
use std::str;

use consts::X86Register;
use consts;

#[derive(Debug)]
pub enum ArgumentLocation {
    OffsetFromStackPointer { offset: i64 },
    Register(X86Register),
}

fn length_until_first_null(strings: &Vec<u8>, offset: usize) -> usize {
    let mut len: usize = 0;
    while strings[offset + len] != 0 {
        len += 1;
    }
    return len;
}

pub fn str_offset_to_string(offset: u64, strings: &Vec<u8>) -> String {
    let offset = offset as usize;
    let length = length_until_first_null(strings, offset);

    let string = Vec::from_iter(strings[offset..(offset + length)].iter().cloned());
    let string = unsafe { CString::from_vec_unchecked(string) };
    string.to_str().unwrap().to_owned()
}

pub fn dwarf_str_to_string(string: &[u8]) -> String {
    String::from_utf8_lossy(string).into_owned()
}

pub fn convert_dw_at_name(attr: &die::Attribute, strings: &Vec<u8>) -> String {
    match attr.data {
        die::AttributeData::StringOffset(val) => str_offset_to_string(val, strings),
        die::AttributeData::String(val) => dwarf_str_to_string(val),
        _ => panic!("Unexpected type for argument name"),
    }
}

pub fn convert_dw_at_location(dwarf_loc: &[u8]) -> ArgumentLocation {
    if dwarf_loc.len() < 1 {
        panic!("Invalid location");
    }

    match dwarf_loc[0] {
        consts::DW_OP_regx => {
            let offset = read_leb128_i64(&dwarf_loc[1..]).unwrap();
            ArgumentLocation::OffsetFromStackPointer { offset }
        }
        consts::DW_OP_fbreg => {
            let register_number = read_u64(&dwarf_loc[1..]).unwrap();
            let register = X86Register::from_u64(register_number).unwrap();
            ArgumentLocation::Register(register)
        }
        consts::DW_OP_reg0...consts::DW_OP_reg31 => {
            // in this case the DW_OP_regn is the register number
            let register_number = dwarf_loc[0] - consts::DW_OP_reg0;
            let register = X86Register::from_u8(register_number).unwrap();
            ArgumentLocation::Register(register)
        }
        _ => panic!("Invalid location value"),
    }
}

fn read_leb128_i64(r: &[u8]) -> Result<i64, &'static str> {
    let mut result = 0;
    let mut shift = 0;
    let size = 64;
    let mut i = 0;
    loop {
        let byte = r[i];
        if shift == 63 && byte != 0x00 && byte != 0x7f {
            return Err("Invalid signed leb128 number");
        }
        result |= i64::from(byte & 0x7f) << shift;
        shift += 7;
        if byte & 0x80 == 0 {
            if shift < size && (byte & 0x40) != 0 {
                // Sign extend
                result |= !0 << shift;
            }
            return Ok(result);
        }
        i += 1;
    }
}

pub fn read_u64(r: &[u8]) -> Result<u64, &'static str> {
    let mut result = 0;
    let mut shift = 0;
    let mut i = 0;
    loop {
        let byte = r[i];
        if shift == 63 && byte != 0x00 && byte != 0x01 {
            return Err("Invalid signed leb128 number");
        }
        result |= u64::from(byte & 0x7f) << shift;
        if byte & 0x80 == 0 {
            return Ok(result);
        }
        shift += 7;
        i += 1;
    }
}
