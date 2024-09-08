use crate::ast;
use ast::*;
use std::collections::{HashMap, HashSet};

// perform dce on the bb once, return whether anything changed
// mutate bb in place
fn dce_bb(basic_block: &mut BasicBlock) -> bool {
    // because rust compiler sucks storing pointer isn't an option
    let mut unused_instructions: HashMap<String, usize> = HashMap::new(); // symbol -> inst index
                                                                          // in BB

    for (i, instruction) in basic_block.instrs.iter().enumerate() {
        for use_key in instruction.get_use_list() {
            unused_instructions.remove(&use_key);
        }

        if let Some(result_key) = instruction.get_result() {
            unused_instructions.insert(result_key.clone(), i);
        }
    }

    let mut unused_instructions_idx: Vec<usize> = unused_instructions
        .values()
        .map(|instruction_idx| instruction_idx.clone())
        .collect();

    // sort in descending order
    unused_instructions_idx.sort();
    unused_instructions_idx.reverse();

    let changed: bool = !unused_instructions_idx.is_empty();

    // pop instructions in descending order
    for idx in unused_instructions_idx {
        basic_block.instrs.remove(idx);
    }

    changed
}

fn dce_function(function: &mut Function) -> bool {
    let mut changed: bool = false;
    let mut basic_blocks = function.get_basic_blocks();

    for basic_block in basic_blocks.iter_mut() {
        loop {
            // keep doing bb's dce until no change
            let dce_bb_changed = dce_bb(basic_block);
            changed |= dce_bb_changed;
            if !dce_bb_changed {
                break;
            }
        }
    }

    if changed {
        // bb has changed, flush bb's instrs back to function
        function.instrs.clear();
        for basic_block in basic_blocks.iter_mut() {
            function.instrs.append(&mut basic_block.instrs); // append is okay because bbs are
                                                             // never used again
        }
    }

    changed
}

// local-scope dce
pub fn local_dce_pass(program: &mut Program) -> bool {
    let mut changed: bool = false;
    for function in program.functions.iter_mut() {
        changed |= dce_function(function);
    }
    changed
}

// function-scope naive dce
pub fn naive_dce_pass(program: &mut Program) -> bool {
    let mut changed: bool = false;
    let mut used_vars: HashSet<String> = HashSet::new();
    for Fn in program.functions.iter_mut() {
        let before = Fn.instrs.len();

        for Inst in &Fn.instrs {
            let uses = Inst.get_use_list();
            for U in uses {
                used_vars.insert(U);
            }
        }

        Fn.instrs.retain(|inst| {
            !inst.is_pure() || // not pure
                inst.get_result().map_or(false, |result| used_vars.contains(&result))
            // has a result that is being used somewhere else
        });

        changed = Fn.instrs.len() != before;
        used_vars.clear();
    }

    changed
}
