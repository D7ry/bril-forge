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
    pub defs: HashSet<String>,
    pub uses: HashSet<String>,
}

impl LivenessState {
    pub fn new() -> LivenessState {
        LivenessState {
            live_in: Vec::new(),
            live_out: Vec::new(),
            defs: HashSet::new(),
            uses: HashSet::new(),
        }
    }
}

fn get_bb_use_list(bb: &BasicBlock) -> HashSet<String> {
    return get_bb_meaningful_use_list(bb);
    let mut use_list: HashSet<String> = HashSet::new();
    for inst in bb.instrs.iter() {
        let inst_use_list: Vec<String> = inst.get_use_list();
        for elem in inst_use_list {
            use_list.insert(elem);
        }
    }
    use_list
}

fn get_bb_meaningful_use_list(bb: &BasicBlock) -> HashSet<String> {
    let mut use_list: HashSet<String> = HashSet::new();
    for inst in bb.instrs.iter() {
        if !inst.is_meaningful() {
            continue;
        }
        let mut inst_use_list: Vec<String> = inst.get_use_list();
        for elem in inst_use_list {
            use_list.insert(elem);
        }
    }
    use_list
}

fn get_bb_def_list(bb: &BasicBlock) -> HashSet<String> {
    let mut def_list: HashSet<String> = HashSet::new();
    for inst in bb.instrs.iter() {
        if let Some(result) = inst.get_result() {
            def_list.insert(result);
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
    live_out_without_local_def.retain(|elem| !new_state.defs.contains(elem));

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
        state.defs = get_bb_def_list(bb);
        state.uses = get_bb_use_list(bb);
    }

    // debug print def-use
    // for i in 0..bbs.len() {
    //     let state = liveness_states.get(i).unwrap();
    //     println!("bb {} defs: ", i);
    //     for d in state.defs.iter() {
    //         print!(" {}", d);
    //     }
    //     println!();
    //
    //     println!("bb {} uses: ", i);
    //     for u in state.uses.iter() {
    //         print!(" {}", u);
    //     }
    //     println!();
    // }

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
    for i in (0..bbs.len()).rev() {
        work_list.push_back(i);
        in_work_list.insert(i);
    }

    while let Some(bb_idx) = work_list.pop_front() {
        //println!("updating liveness for bb {}", bb_idx);
        in_work_list.remove(&bb_idx);
        let bb = bbs.get_mut(bb_idx).unwrap();
        let liveness_state: &LivenessState = liveness_states.get(bb_idx).unwrap();
        let mut parent_liveness_states: Vec<&LivenessState> = Vec::new();
        let mut children_liveness_states: Vec<&LivenessState> = Vec::new();
        //println!(" has {} parents", bb.in_bb_indices.len());
        for parent_idx in bb.in_bb_indices.iter() {
            println!("parent {}", parent_idx);
            parent_liveness_states.push(liveness_states.get(*parent_idx).unwrap());
        }

        //println!(" has {} children", bb.out_bb_indices.len());
        for child_idx in bb.out_bb_indices.iter() {
            //println!("children {}", child_idx);
            children_liveness_states.push(liveness_states.get(*child_idx).unwrap());
        }
        let res = bb_update_liveness(
            bb,
            liveness_state,
            &parent_liveness_states,
            &children_liveness_states,
        );

        if res.1 {
            // update liveness
            *liveness_states.get_mut(bb_idx).unwrap() = res.0;
            // push all parents onto worklist
            for parent_idx in bb.in_bb_indices.iter() {
                if in_work_list.insert(parent_idx.clone()) {
                    work_list.push_back(parent_idx.clone());
                }
            }
        }
    }

    // debug print
    if true {
        for i in 0..bbs.len() {
            println!("bb {}'s liveness:", i);
            let liveness = liveness_states.get(i).unwrap();
            println!("Live in: ");
            for elem in liveness.live_in.iter() {
                print!(" {}", elem);
            }
            println!();

            println!("Live out: ");
            for elem in liveness.live_out.iter() {
                print!(" {}", elem);
            }

            println!();
        }
    }

    // block-scope liveness analysis done,
    // now perform instruction-granularity liveness analysis/DCE
    for bb_idx in 0..bbs.len() {
        let bb: &mut BasicBlock = bbs.get_mut(bb_idx).unwrap();

        let live_out: Vec<String> = liveness_states.get(bb_idx).unwrap().live_out.clone();
        let mut live_out: HashSet<String> = live_out.into_iter().collect();

        let mut insts_to_pop: Vec<usize> = Vec::new();

        // reverse traverse the insts
        for inst_idx in (0..bb.instrs.len()).rev() {
            let inst = bb.instrs.get(inst_idx).unwrap();
            let mut inst_is_dead: bool = !inst.is_meaningful();
            if let Some(dest) = inst.get_result() {
                if live_out.contains(&dest) {
                    inst_is_dead = false;
                    // this is the latest point where we assign to
                    // the live_out, we now can safely remove it
                    live_out.remove(&dest);
                    // all vars used by the inst are needed
                    // this gracefully handles self-referential vars as a regular case.
                    for var_used in inst.get_use_list() {
                        live_out.insert(var_used);
                    }
                }
            }

            if inst_is_dead {
                insts_to_pop.push(inst_idx);
            }
        }

        changed |= insts_to_pop.len() != 0;
        for inst_idx in insts_to_pop {
            bb.instrs.remove(inst_idx);
        }
    }

    // flush back on change
    if changed {
        function.instrs.clear();
        for bb in bbs.iter_mut() {
            function.instrs.append(&mut bb.instrs); 
        }
    }

    changed
}

pub fn global_dce_pass_using_livenss(program: &mut Program) -> bool {
    let mut changed: bool = false;

    for function in program.functions.iter_mut() {
        changed |= global_dce_on_function(function);
    }

    changed
}
