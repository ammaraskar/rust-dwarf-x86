extern crate dwarf;

use std::path;
pub use dwarf::elf;

#[derive(Debug)]
pub struct Executable {
    path: path::PathBuf
}

impl Executable {
    pub fn new(path: &path::Path) -> Executable {
        return Executable {
            path: path::PathBuf::from(path)
        }
    }
}

pub fn load_executable(path: &path::Path) -> Result<Executable, dwarf::ReadError> {
    elf::load(path)?;
    return Ok(Executable::new(path));
}
