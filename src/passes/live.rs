// liveness analysis

use crate::ast;
use ast::*;
use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, HashSet, VecDeque};

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

// TODO: this is pretty jank, we're copying memory chunks around,
// not hashing states etc.... i need to get better at rust to do this
fn bb_update_liveness(
    bb: &BasicBlock,
    state: &LivenessState,
    predecessors: &Vec<&LivenessState>,
    successors: &Vec<&LivenessState>,
) -> (LivenessState, bool) {
    let mut new_state: LivenessState = LivenessState::new();
    // i'm agreeing with Jonathan Blow here, rust's problems is that it
    // creates so much friction in problem solving that it makes remotely difficult
    // problems 10x harder to solve, and perf isn't necessarily better unless you're a
    // master rust programmer
    new_state.defs = get_bb_def_list(bb);
    new_state.uses = get_bb_use_list(bb);

    // bb's live out is union of all its successors' live in
    for succ in successors.iter() {
        for var in succ.live_in.iter() {
            new_state.live_out.push(var.clone());
        }
    }

    // bb's live in is its used vars, union all its live out - all its defed vars
    for used in new_state.uses.iter() {
        new_state.live_in.push(used.clone());
    }

    let mut live_out_without_local_def: Vec<String> = new_state.live_out.clone();
    live_out_without_local_def.retain(|elem| new_state.defs.contains(elem));

    // FIXME: use hashmap, no time because it's overdue for now
    for elem in live_out_without_local_def.iter() {
        if !new_state.live_in.contains(elem) {
            new_state.live_in.push(elem.clone());
        }
    }

    let changed: bool = new_state.live_out.len() != state.live_out.len()
        || new_state.live_in.len() != state.live_in.len();
    (new_state, changed)
}

// function scope global dce
fn global_dce_on_function(function: &mut Function) -> bool {
    let mut changed: bool = false;

    let mut bbs = function.get_basic_blocks();
    let mut liveness_states: Vec<LivenessState> = Vec::new();
    let mut bb_pre_succ_liveness_states: Vec<(Vec<&LivenessState>, Vec<&LivenessState>)> =
        Vec::new();

    let mut work_list: VecDeque<usize> = VecDeque::new();
    let mut in_work_list: HashSet<usize> = HashSet::new(); // indices already in worklist to
                                                           // prevent repetition
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
    for i in bbs.len()..0 {
        work_list.push_back(i);
    }

    while let Some(bb_idx) = work_list.pop_front() {
        in_work_list.remove(&bb_idx);
        let bb = bbs.get(bb_idx).unwrap();
        let liveness_state: &LivenessState = liveness_states.get(bb_idx).unwrap();
        let mut parent_liveness_states: Vec<&LivenessState> = Vec::new();
        let mut children_liveness_states: Vec<&LivenessState> = Vec::new();
        for parent_idx in bb.in_bb_indices.iter() {
            parent_liveness_states.push(liveness_states.get(*parent_idx).unwrap());
        }
        for child_idx in bb.out_bb_indices.iter() {
            children_liveness_states.push(liveness_states.get(*child_idx).unwrap());
        }
        let res = bb_update_liveness(
            bb,
            liveness_state,
            &parent_liveness_states,
            &children_liveness_states,
        );
        if res.1 {
            // push all parents onto worklist
            for parent_idx in bb.in_bb_indices.iter() {
                if in_work_list.insert(parent_idx.clone()) {
                    work_list.push_back(parent_idx.clone());
                }
            }
        }
    }

    // liveness update done, perform dce

    changed
}

pub fn global_dce_pass_using_livenss(program: &mut Program) -> bool {
    let mut changed: bool = false;

    for function in program.functions.iter_mut() {
        changed |= global_dce_on_function(function);
    }

    changed
}
