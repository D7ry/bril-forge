// liveness analysis

use crate::ast;
use ast::*;
use std::collections::{HashMap, HashSet};

// state of liveness passed as program analysis goes,
// on a bb granularity
struct LivenessState {
    pub live_in: Vec<String>,  // vars that are alive when we enter bb
    pub live_out: Vec<String>, // vars that are alive when we exit bb
    pub defs: Vec<String>,
    pub uses: Vec<String>,
}

impl LivenessState {
    pub fn new() -> LivenessState {
        LivenessState {
            live_in: Vec::new(),
            live_out: Vec::new(),
            defs: Vec::new(),
            uses: Vec::new(),
        }
    }
}

fn get_bb_use_list(bb: &BasicBlock) -> Vec<String> {
    let mut use_list: Vec<String> = Vec::new();
    for inst in bb.instrs.iter() {
        let mut inst_use_list: Vec<String> = inst.get_use_list();
        use_list.append(&mut inst_use_list);
    }
    use_list
}

fn get_bb_meaningful_use_list(bb: &BasicBlock) -> Vec<String> {
    let mut use_list: Vec<String> = Vec::new();
    for inst in bb.instrs.iter() {
        if !inst.is_meaningful() {
            continue;
        }
        let mut inst_use_list: Vec<String> = inst.get_use_list();
        use_list.append(&mut inst_use_list);
    }
    use_list
}

fn get_bb_def_list(bb: &BasicBlock) -> Vec<String> {
    let mut def_list: Vec<String> = Vec::new();
    for inst in bb.instrs.iter() {
        if let Some(result) = inst.get_result() {
            def_list.push(result);
        }
    }
    def_list
}

fn bb_update_liveness(
    state: &mut LivenessState,
    predecessors: &Vec<&LivenessState>,
    successors: &Vec<&LivenessState>,
) -> bool {
    let mut changed: bool = false;

    changed
}

// function scope global dce
fn global_dce_on_function(function: &mut Function) -> bool {
    let mut changed: bool = false;

    let mut bbs = function.get_basic_blocks();
    let mut liveness_states: Vec<LivenessState> = Vec::new();
    let mut bb_pre_succ_liveness_states: Vec<(Vec<&LivenessState>, Vec<&LivenessState>)> =
        Vec::new();

    // populate default states
    for _i in 0..bbs.len() {
        liveness_states.push(LivenessState::new());
    }

    // populate def-use
    for i in 0..bbs.len() {
        let bb = &bbs[i];
        let state: &mut LivenessState = liveness_states.get_mut(i).unwrap();
        state.defs.append(&mut get_bb_def_list(bb));
        state.uses.append(&mut get_bb_use_list(bb));
    }

    // populate lookup table to pre and successors of a bb's liveness states
    for bb in bbs.iter() {
        let mut pre_succ_liveness_states: (Vec<&LivenessState>, Vec<&LivenessState>) =
            (Vec::new(), Vec::new());
        for i in bb.in_bb_indices.iter() {
            pre_succ_liveness_states.0.push(&liveness_states[*i]);
        }
        for i in bb.out_bb_indices.iter() {
            pre_succ_liveness_states.1.push(&liveness_states[*i]);
        }
        bb_pre_succ_liveness_states.push(pre_succ_liveness_states);
    }

    // construct worklist

    changed
}

pub fn global_dce_pass_using_livenss(program: &mut Program) -> bool {
    let mut changed: bool = false;

    for function in program.functions.iter_mut() {
        changed |= global_dce_on_function(function);
    }

    changed
}
