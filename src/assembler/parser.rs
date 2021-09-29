use pest::Parser;
use std::fs;

#[derive(Parser)]
#[grammar = "asm.pest"]
pub struct AssemblyParser;

pub fn parse() {
    let unparsed_file = fs::read_to_string("test.asm").unwrap();

    let file = AssemblyParser::parse(Rule::assembly, &unparsed_file).expect("unsuccessful parse").next().unwrap();

    for instruction in file.into_inner() {
        println!("{:?}", instruction);
    }
}