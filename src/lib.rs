#[macro_use]
extern crate enum_primitive;
extern crate dwarf;

mod dwarf_utils;
mod consts;

pub use dwarf_utils::ArgumentLocation;

use std::path;
use dwarf::elf;
use dwarf::die;

#[derive(Debug)]
pub struct Argument {
    name: String,
    location: ArgumentLocation,
}

#[derive(Debug)]
pub struct Function {
    name: String,
    arguments: Vec<Argument>,
    start_address: u64,
}

#[derive(Debug)]
pub struct Executable {
    path: path::PathBuf,
    sections: dwarf::Sections<dwarf::AnyEndian>,
    functions: Vec<Function>,
}

fn process_function_child(entry: &die::Die, strings: &Vec<u8>) -> Option<Argument> {
    if entry.tag != dwarf::constant::DW_TAG_formal_parameter {
        return None;
    }
    let mut arg_name: Result<String, &'static str> = Err("Argument doesn't have a name attribute");
    let mut location: Result<ArgumentLocation, &'static str> =
        Err("Argument doesn't have a location attribute");

    for attr in &entry.attributes {
        match attr.at {
            dwarf::constant::DW_AT_name => {
                arg_name = Ok(dwarf_utils::convert_dw_at_name(attr, strings));
            }
            dwarf::constant::DW_AT_location => {
                match attr.data {
                    die::AttributeData::ExprLoc(val) => {
                        location = Ok(dwarf_utils::convert_dw_at_location(val));
                    }
                    _ => panic!("Unexpected type for argument location"),
                }
            }
            _ => (),
        }
    }

    return Some(Argument {
        name: arg_name.unwrap(),
        location: location.unwrap(),
    });
}

fn get_function_from_dwarf_entry(
    entry: &die::Die,
    tree: die::DieTree<dwarf::AnyEndian>,
    strings: &Vec<u8>,
) -> Function {
    let mut func_name: Result<String, &'static str> = Err("Function doesn't have a name attribute");
    let mut start_address: Result<u64, &'static str> =
        Err("Function doesn't have starting address");

    for attr in &entry.attributes {
        match attr.at {
            dwarf::constant::DW_AT_name => {
                func_name = Ok(dwarf_utils::convert_dw_at_name(attr, strings))
            }
            dwarf::constant::DW_AT_low_pc => {
                start_address = Ok(match attr.data {
                    die::AttributeData::Address(val) => val,
                    _ => panic!("Unexpected type for low_pc"),
                });
            }
            _ => (),
        };
    }

    let mut args: Vec<Argument> = Vec::new();

    // iterate over the children to get the arguments
    let mut tree = tree;
    // skip the first element as that is the function itself
    let mut tree = tree.iter();
    let mut tree = tree.next().unwrap().unwrap();

    while let Some(child) = tree.next().unwrap() {
        let argument = process_function_child(child.entry(), strings);
        match argument {
            Some(argument) => {
                args.push(argument);
            }
            _ => {}
        }
    }

    return Function {
        name: func_name.unwrap(),
        arguments: args,
        start_address: start_address.unwrap(),
    };
}

impl Executable {
    fn new(path: &path::Path, sections: dwarf::Sections<dwarf::AnyEndian>) -> Executable {
        return Executable {
            path: path::PathBuf::from(path),
            sections: sections,
            functions: Vec::new(),
        };
    }

    pub fn locate_functions(mut self) {
        if self.functions.len() > 0 {
            self.functions.clear();
        }

        let mut units = self.sections.compilation_units();
        while let Some(unit) = units.next().unwrap() {
            let abbrev = self.sections.abbrev(&unit.common).unwrap();
            let mut entries = unit.entries(&abbrev);

            while let Some(entry) = entries.next().unwrap() {
                if entry.tag == dwarf::constant::DW_TAG_subprogram {
                    let tree = unit.entry(entry.offset, &abbrev).unwrap().tree();
                    let func = get_function_from_dwarf_entry(entry, tree, &self.sections.debug_str);

                    println!("{:?}", func);
                    self.functions.push(func);
                }
            }
        }
    }
}

pub fn load_executable(path: &path::Path) -> Result<Executable, dwarf::ReadError> {
    let sections = elf::load(path)?;
    return Ok(Executable::new(path, sections));
}
