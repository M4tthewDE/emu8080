extern crate pest;
#[macro_use]
extern crate pest_derive;

mod assembler;
mod cpu;

fn main() {
    let mut cpu = cpu::initialize_cpu();

    let assembler = assembler::Assembler::new("test.asm".to_owned(), "output".to_owned());

    assembler.assemble();
    let instructions = assembler.disassemble("output".to_owned());

    cpu.run(instructions);
}

#[cfg(test)]
mod tests {
    use crate::assembler;
    use crate::cpu;

    #[test]
    fn test_end_to_end() {
        let mut cpu = cpu::initialize_cpu();

        let assembler = assembler::Assembler::new("test.asm".to_owned(), "output".to_owned());

        assembler.assemble();
        let instructions = assembler.disassemble("output".to_owned());

        cpu.run(instructions);
    }
}
