use crate::ast;
use ast::*;


fn global_constant_prop_on_fn(function: &mut Function) -> bool {
    let mut changed: bool = false;

    let bbs = function.get_basic_blocks();
    for bb in bbs {
        

    }

    changed
}
pub fn global_const_propagation_pass(program: &mut Program) -> bool {
    let mut changed: bool = false;
    for function in program.functions.iter_mut() {
        changed |= global_constant_prop_on_fn(function);
    }

    changed
}
