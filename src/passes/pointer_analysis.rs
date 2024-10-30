use crate::ast;
use crate::dom;
use ast::*;
use dom::*;

use std::collections::{HashMap, HashSet, VecDeque};

fn pointer_analysis_pass_bb(bb: &mut BasicBlock) -> bool {}

// build a points-to graph using information from a bb
fn build_point_to_graph(
    bb: &BasicBlock,
    bb_inst_offset: usize,
    point_to_graph: &mut HashMap<String, HashSet<usize>>,
    num_fn_insts: usize,
) -> bool {
    let mut changed: bool = false;

    for (inst_id_local, inst) in bb.instrs.iter().enumerate() {
        let inst_id_global: usize = inst_id_local + bb_inst_offset; // function-scope instruction
        //x = alloc n: x points to this allocations
        //x = id y: x points to the same locations as y did
        //x = ptradd p offset: same as id (conservative)
        //x = load p: we aren't tracking anything about p, so x points to all memory locations
        match inst {
            // why don't we have cpp iterators ugh
            Instruction::Opcode(inst) => match inst {
                OpcodeInstruction::Alloc { args, dest, typ } => {
                    if point_to_graph.contains_key(dest) == false {
                        point_to_graph.insert(dest.clone(), HashSet::new());
                    }
                    let pointed_to: &mut HashSet<usize> = point_to_graph.get_mut(dest).unwrap();
                    pointed_to.insert(inst_id_global);
                    changed |= true;
                }
                OpcodeInstruction::Ptradd { args, dest, typ } |
                OpcodeInstruction::Id { args, dest, typ } => {
                    assert!(args.len() == 1);
                    let src_var_name = args.first().unwrap();
                    let mut src_pointed_to: HashSet<usize> = HashSet::new();
                    if let Some(src_pointed_to_it) = point_to_graph.get(src_var_name) {
                        src_pointed_to = src_pointed_to_it.clone();
                    }

                    if point_to_graph.contains_key(dest) == false {
                        point_to_graph.insert(dest.clone(), HashSet::new());
                    }
                    let pointed_to: &mut HashSet<usize> = point_to_graph.get_mut(dest).unwrap();
                    pointed_to.extend(src_pointed_to);
                    changed |= true;
                }
                OpcodeInstruction::Load { args, dest, typ } => {
                    if point_to_graph.contains_key(dest) == false {
                        point_to_graph.insert(dest.clone(), HashSet::new());
                    }
                    let pointed_to: &mut HashSet<usize> = point_to_graph.get_mut(dest).unwrap();
                    for i in 0..num_fn_insts { // points to everything
                        pointed_to.insert(i);
                    }
                    changed |= true;
                }
                _ => {}
            },
            _ => {}
        }
    }

    changed
}

fn dead_store_elimination(function: &mut Function, point_to_graph: &HashMap<String, HashSet<usize>>) -> bool {
    let mut changed: bool = false;

    changed
}

fn pointer_analysis_pass_fn(function: &mut Function) -> bool {
    let mut changed: bool = false;
    let mut bbs = function.get_basic_blocks();

    // collect pointer alias info, building point-to graph
    // variable name -> allocation site(location in the function block)
    let mut point_to_graph: HashMap<String, HashSet<usize>> = HashMap::new();
    let mut bb_inst_offsets: Vec<usize> = Vec::new(); // bb idx -> instruction offset

    let num_total_insts: usize; // total # of instructions
                                // collect bb inst offset
    {
        let mut offset: usize = 0;
        for bb in bbs.iter() {
            bb_inst_offsets.push(offset);
            offset += bb.instrs.len();
        }
        num_total_insts = offset;
    }

    // we don't know about function arguments' aliasing --
    // so we assume they alias with every allocation
    if let Some(fn_args) = &function.args {
        for fn_arg in fn_args.iter() {
            point_to_graph.insert(fn_arg.name.clone(), HashSet::new());
            // push in every single code location
            let locations: &mut HashSet<usize> = point_to_graph.get_mut(&fn_arg.name).unwrap();
            for loc in 0..num_total_insts {
                locations.insert(loc);
            }
        }
    }

    let mut wl: VecDeque<usize> = VecDeque::new();
    let mut in_wl: HashSet<usize> = HashSet::new();
    // initialize wl with all bbs
    for i in 0..bbs.len() {
        wl.push_back(i);
        in_wl.insert(i);
    }

    // perform forward analysis
    // processing wl, pushing back onto wl on change
    while let Some(bb_idx) = wl.pop_front() {
        let bb: &BasicBlock;
        unsafe {
            bb = bbs.get_unchecked_mut(bb_idx);
        }
        let inst_offset = bb_inst_offsets.get(bb_idx).unwrap().clone();
        let point_to_graph_changed: bool =
            build_point_to_graph(bb, inst_offset, &mut point_to_graph, num_total_insts);
        if point_to_graph_changed {
            for child in bb.out_bb_indices.iter() {
                if in_wl.contains(child) == false {
                    wl.push_back(child.clone());
                    in_wl.insert(child.clone());
                }
            }
        }
    }

    // done building points-to graph, now perform optimizations
    changed |= dead_store_elimination(function, &point_to_graph);

    changed
}

pub fn pointer_analysis_pass(program: &mut Program) -> bool {
    let mut changed: bool = false;

    for function in program.functions.iter_mut() {
        changed |= pointer_analysis_pass_fn(function);
    }

    changed
}
