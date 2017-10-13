#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]

pub const DW_OP_regx: u8 = 0x90;
pub const DW_OP_fbreg: u8 = 0x91;
pub const DW_OP_reg0: u8 = 0x50;
pub const DW_OP_reg31: u8 = 0x6f;

enum_from_primitive! {
#[derive(Debug)]
pub enum X86Register {
    rax = 0,
    rdx = 1,
    rcx = 2,
    rbx = 3,
    rsi = 4,
    rdi = 5,
    rbp = 6,
    rsp = 7,
    r8 = 8,
    r9 = 9,
    r10 = 10,
    r11 = 11,
    r12 = 12,
    r13 = 13,
    r14 = 14,
    r15 = 15
}
}
