use enum_primitive::FromPrimitive;
use leb128;

use consts::X86Register;
use consts;

#[derive(Debug, Copy, Clone)]
pub enum ArgumentLocation {
    OffsetFromStackPointer(i64),
    Register(X86Register),
}


pub fn convert_dw_at_location(dwarf_loc: &[u8]) -> ArgumentLocation {
    if dwarf_loc.len() < 1 {
        panic!("Invalid location");
    }

    match dwarf_loc[0] {
        consts::DW_OP_regx => {
            let register_number = leb128::read::unsigned(&mut &dwarf_loc[1..]).unwrap();
            let register = X86Register::from_u64(register_number).unwrap();
            ArgumentLocation::Register(register)
        }
        consts::DW_OP_fbreg => {
            let offset = leb128::read::signed(&mut &dwarf_loc[1..]).unwrap();
            ArgumentLocation::OffsetFromStackPointer(offset)
        }
        consts::DW_OP_reg0...consts::DW_OP_reg31 => {
            // in this case the DW_OP_reg<n> is the register number
            let register_number = dwarf_loc[0] - consts::DW_OP_reg0;
            let register = X86Register::from_u8(register_number).unwrap();
            ArgumentLocation::Register(register)
        }
        _ => panic!("Invalid location value"),
    }
}
