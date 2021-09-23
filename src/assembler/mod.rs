use std::fs::File;

#[derive(Debug)]
pub struct Assembler{
    input_asm: File,
}

impl Assembler {
    pub fn new(input_asm_name: String) -> Assembler {
        let file = File::open(input_asm_name);
        let file = match file {
            Ok(file) => file,
            Err(error) => panic!("Problem opening file: {:?}", error),
        };
        
        Assembler {
            input_asm: file,
        }
    }
}