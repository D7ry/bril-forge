use crate::ast;
use ast::*;

pub fn delete_everything_pass(program: &mut Program) -> bool {
    program.functions.clear();
    true
}

pub fn do_nothing_pass(_program: &mut Program) -> bool {
    false
}
