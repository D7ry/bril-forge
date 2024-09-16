// local value numbering
use crate::ast;
use ast::*;
use std::collections::{HashMap, HashSet};

fn lvn_bb(bb: &mut BasicBlock) -> bool {
    let mut changed: bool = false;

    changed
}

fn lvn_fn(function: &mut Function) -> bool {
    let mut changed: bool = false;
    let mut bbs = function.get_basic_blocks();

    for bb in bbs.iter_mut() {
        let bb_changed = lvn_bb(bb);
        changed |= bb_changed;
    }

    if changed {
        // bb has changed, flush bb's instrs back to function
        function.instrs.clear();
        for basic_block in bbs.iter_mut() {
            function.instrs.append(&mut basic_block.instrs); // append is okay because bbs are
                                                             // never used again
        }
    }
    changed
}

pub fn lvn_pass(program: &mut Program) -> bool {
    let mut changed: bool = false;

    for function in program.functions.iter_mut() {
        changed |= lvn_fn(function);
    }

    changed
}
