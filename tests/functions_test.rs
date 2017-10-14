extern crate dwarf_x86;

use std::path::Path;

#[test]
fn it_scrapes_functions() {
    let executable = dwarf_x86::load_executable(Path::new("./test_files/simple_executable"))
        .unwrap();

    executable.get_functions();
}

#[test]
fn it_gets_the_right_functions() {
    let executable = dwarf_x86::load_executable(Path::new("./test_files/register_argument"))
        .unwrap();

    for func in executable.get_functions() {
        if func.name == "main" {
            assert_eq!(func.arguments.len(), 2);

            assert_eq!(func.arguments[0].name, "argc");
            assert_eq!(func.arguments[1].name, "argv");

            // first argument is passed in RDI
            let location = func.arguments[0].location;
            match location {
                dwarf_x86::ArgumentLocation::Register(r) => {
                    assert_eq!(r, dwarf_x86::X86Register::rdi);
                }
                _ => panic!("argc should have been in a register"),
            }
        } else if func.name == "somefunc" {
            assert_eq!(func.arguments.len(), 2);

            assert_eq!(func.arguments[0].name, "a");
            assert_eq!(func.arguments[1].name, "b");

        } else if func.name == "someOtherFunc" {
            assert_eq!(func.arguments.len(), 3);

            assert_eq!(func.arguments[0].name, "a");
            assert_eq!(func.arguments[1].name, "b");
            assert_eq!(func.arguments[2].name, "c");
        } else {
            println!("{:?}", func.name);
            panic!("Wrong function detected");
        }
    }
}
