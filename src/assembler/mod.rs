#[derive(Debug)]
pub struct Assembler{
    input_asm: String,
}

impl Assembler {
    pub fn new(input_asm: String) -> Assembler {
        Assembler {
            input_asm: input_asm,
        }
    }
}