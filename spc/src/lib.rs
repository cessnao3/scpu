mod components;
mod parser;
mod types;

pub fn compile(s: &str) -> Vec<u16> {
    parser::parse(s);
    Vec::new()
}
