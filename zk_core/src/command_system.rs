use std::io::Write;
use clap::App;

pub struct Environment<'a> {
    pub output: &'a mut dyn Write,
}

impl Environment<'_> {
    pub fn new<'a, W: Write>(w: &'a mut W) -> Environment<'a> {
        Environment {
            output: w,
        }
    }
}

// TODO: consider error_chain crate?
#[derive(Debug)]
pub enum ZkCommandExecutionErrorKind {
    ArgumentParsing(clap::Error),
    UnknownCommand(String)
}

pub trait ZkCommand {
    fn new() -> Self where Self: Sized;
    fn name(&self) -> &'static str;
    fn run(&self, env: &mut Environment) -> Result<(), ZkCommandExecutionErrorKind>;
    fn build_clap_subcmd(&self) -> App<'static, 'static>;
}
