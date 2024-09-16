use crate::ast;
use ast::*;
use std::collections::{HashMap, HashSet};

// perform dce on the bb once, return whether anything changed
// mutate bb in place
fn dce_bb_dead_store(basic_block: &mut BasicBlock) -> bool {
    // because rust compiler sucks storing pointer isn't an option
    let mut unused_results_and_their_pure_insts: HashMap<String, usize> = HashMap::new(); // result id -> inst index
                                                                          // in BB
    let mut unused_instructions_idx: Vec<usize> = Vec::new(); // instruction indices that are dead
                                                            // code

    // iterate through all insts in sequence in this BB
    // this is important bc data flows this way.
    for (i, instruction) in basic_block.instrs.iter().enumerate() {
        if instruction.is_label() {
            continue; // don't mess with labels
        }
        for use_key in instruction.get_use_list() {
            unused_results_and_their_pure_insts.remove(&use_key);
            // note we don't unwrap here, because the use can come from:
            // 1. an outside BB
            // 2. function argument
        }

        if let Some(result_key) = instruction.get_result() {
            // the previous result hasn't ever been used until this reassignment
            // this means the previous instruction is probably dead.
            if let Some(instruction_idx) = unused_results_and_their_pure_insts.get(&result_key) {
                unused_instructions_idx.push(instruction_idx.clone());
            }
            // note that the pure check here is important
            // an instruction can have its result unused until the next result assignment,
            // but the instruction can still carry out side-effects
            if instruction.has_no_side_effects() {
                unused_results_and_their_pure_insts.insert(result_key.clone(), i);
            }
        }
    }

    // perform inst eliminination
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
            let dce_bb_changed = dce_bb_dead_store(basic_block);
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
    for function in program.functions.iter_mut() {
        let before = function.instrs.len();

        for inst in &function.instrs {
            let uses = inst.get_use_list();
            for u in uses {
                used_vars.insert(u);
            }
        }

        function.instrs.retain(|inst| {
            !inst.has_no_side_effects() || // not pure
                inst.get_result().map_or(false, |result| used_vars.contains(&result))
            // has a result that is being used somewhere else
        });

        changed = function.instrs.len() != before;
        used_vars.clear();
    }

    changed
}
