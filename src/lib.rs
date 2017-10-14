#[macro_use]
extern crate enum_primitive;
extern crate elf;
extern crate gimli;
extern crate leb128;

mod dwarf_utils;
mod consts;

pub use dwarf_utils::ArgumentLocation;

use std::path;

#[derive(Debug)]
pub struct Argument {
    pub name: String,
    pub location: ArgumentLocation,
}

#[derive(Debug)]
pub struct Function {
    pub name: String,
    pub arguments: Vec<Argument>,
    pub start_address: u64,
}

#[derive(Debug)]
pub struct Executable {
    path: path::PathBuf,
    file: elf::File,
    endianess: gimli::RunTimeEndian,
}

fn process_function_child(
    entry: &gimli::DebuggingInformationEntry<gimli::EndianBuf<gimli::RunTimeEndian>>,
    strings: &gimli::DebugStr<gimli::EndianBuf<gimli::RunTimeEndian>>,
) -> Option<Argument> {
    if entry.tag() != gimli::DW_TAG_formal_parameter {
        return None;
    }
    let mut arg_name: Result<String, &'static str> = Err("Argument doesn't have a name attribute");
    let mut location: Result<ArgumentLocation, &'static str> =
        Err("Argument doesn't have a location attribute");

    let mut attrs = entry.attrs();
    while let Some(attr) = attrs.next().unwrap() {
        match attr.name() {
            gimli::DW_AT_name => {
                let arg_name_str = attr.string_value(strings).unwrap().to_string_lossy();
                arg_name = Ok(arg_name_str.into_owned());
            }
            gimli::DW_AT_location => {
                match attr.value() {
                    gimli::AttributeValue::Exprloc(expr) => {
                        let buf = expr.0.buf();
                        location = Ok(dwarf_utils::convert_dw_at_location(buf));
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
    node: gimli::EntriesTreeNode<gimli::EndianBuf<gimli::RunTimeEndian>>,
    strings: &gimli::DebugStr<gimli::EndianBuf<gimli::RunTimeEndian>>,
) -> Function {
    let mut func_name: Result<String, &'static str> = Err("Function doesn't have a name attribute");
    let mut start_address: Result<u64, &'static str> =
        Err("Function doesn't have starting address");

    let entry = {
        node.entry().clone()
    };
    let mut attrs = entry.attrs();
    while let Some(attr) = attrs.next().unwrap() {
        match attr.name() {
            gimli::DW_AT_name => {
                let func_name_str = attr.string_value(strings).unwrap().to_string_lossy();
                func_name = Ok(func_name_str.into_owned());
            }
            gimli::DW_AT_low_pc => {
                match attr.value() {
                    gimli::AttributeValue::Addr(val) => start_address = Ok(val),
                    _ => panic!("Invalid type for function start address"),
                }
            }
            _ => (),
        };
    }

    let mut args: Vec<Argument> = Vec::new();

    // iterate over the children to get the arguments
    let mut children = node.children();

    while let Some(child) = children.next().unwrap() {
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
    pub fn debug_info<'a>(
        &'a self,
    ) -> gimli::DebugInfo<gimli::EndianBuf<'a, gimli::RunTimeEndian>> {
        let debug_info = self.file.get_section(".debug_info").unwrap();
        return gimli::DebugInfo::new(&debug_info.data, self.endianess);
    }

    pub fn debug_abbrev<'a>(
        &'a self,
    ) -> gimli::DebugAbbrev<gimli::EndianBuf<'a, gimli::RunTimeEndian>> {
        let debug_abbrev = self.file.get_section(".debug_abbrev").unwrap();
        return gimli::DebugAbbrev::new(&debug_abbrev.data, self.endianess);
    }

    pub fn debug_str<'a>(&'a self) -> gimli::DebugStr<gimli::EndianBuf<'a, gimli::RunTimeEndian>> {
        let debug_str = self.file.get_section(".debug_str").unwrap();
        return gimli::DebugStr::new(&debug_str.data, self.endianess);
    }

    pub fn get_functions(self) -> Vec<Function> {
        let mut functions: Vec<Function> = Vec::new();

        let debug_info = self.debug_info();
        let debug_abbrev = self.debug_abbrev();
        let debug_str = self.debug_str();

        let mut units = debug_info.units();
        while let Some(unit) = units.next().unwrap() {
            let abbrev = unit.abbreviations(&debug_abbrev).unwrap();

            let mut tree = unit.entries_tree(&abbrev, None).unwrap();
            let mut tree = tree.root().unwrap().children();

            while let Some(child) = tree.next().unwrap() {
                let entry = {
                    child.entry().clone()
                };
                if entry.tag() == gimli::DW_TAG_subprogram {
                    let func = get_function_from_dwarf_entry(child, &debug_str);

                    functions.push(func);
                }
            }
        }

        return functions;
    }
}

#[derive(Debug)]
pub enum ExecutableLoadError {
    Parse(elf::ParseError),
    InvalidFile(&'static str),
    MissingSection(&'static str),
}

pub fn load_executable(path: &path::Path) -> Result<Executable, ExecutableLoadError> {
    let elf_file = try!(elf::File::open_path(&path).map_err(
        ExecutableLoadError::Parse,
    ));

    let endianess = match elf_file.ehdr.data {
        elf::types::ELFDATA2LSB => gimli::RunTimeEndian::Little,
        elf::types::ELFDATA2MSB => gimli::RunTimeEndian::Big,
        _ => {
            return Err(ExecutableLoadError::InvalidFile(
                "Invalid ELF file, not little endian or big endian",
            ))
        }
    };

    if elf_file.get_section(".debug_info").is_none() {
        return Err(ExecutableLoadError::MissingSection(
            "Missing debug_info section",
        ));
    }
    if elf_file.get_section(".debug_abbrev").is_none() {
        return Err(ExecutableLoadError::MissingSection(
            "Missing debug_abbrev section",
        ));
    }
    if elf_file.get_section(".debug_str").is_none() {
        return Err(ExecutableLoadError::MissingSection(
            "Missing debug_str section",
        ));
    }

    return Ok(Executable {
        path: path::PathBuf::from(path),
        file: elf_file,
        endianess: endianess,
    });
}
