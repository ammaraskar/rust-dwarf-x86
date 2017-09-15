extern crate dwarf_x86;

use std::path::Path;

#[test]
#[should_panic]
fn it_doesnt_load_non_existent_file() {
    dwarf_x86::load_executable(Path::new("non_existent")).unwrap();
}

#[test]
#[should_panic]
fn it_doesnt_load_non_executable() {
    dwarf_x86::load_executable(Path::new("./README.md")).unwrap();
} 

#[test]
fn it_loads_an_executable() {
    dwarf_x86::load_executable(Path::new("./test_files/simple_executable")).unwrap();
}