use crate::ast;
use crate::dom;
use ast::*;
use dom::*;

use std::collections::{HashMap, HashSet, VecDeque};

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
                OpcodeInstruction::Ptradd { args, dest, typ }
                | OpcodeInstruction::Id { args, dest, typ } => {
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
                    for i in 0..num_fn_insts {
                        // points to everything
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

fn var_alias(var1: &String, var2: &String, point_to_graph: &HashMap<String, HashSet<usize>>) -> bool{
    if point_to_graph.contains_key(var1) && point_to_graph.contains_key(var2) {
        let var1_pointed_to: &HashSet<usize> = point_to_graph.get(var1).unwrap();
        let var2_pointed_to: &HashSet<usize> = point_to_graph.get(var2).unwrap();

        let mut has_alias: bool = false;

        // check for point to graph overlap
        for v in var1_pointed_to.iter() {
            if var2_pointed_to.contains(v) {
                has_alias = true;
                break;
            }
        }

        has_alias
    } else {
        false
    }
}

fn dead_store_elimination_bb(
    bb: &mut BasicBlock,
    point_to_graph: &HashMap<String, HashSet<usize>>, // var name -> memory ids var could point to
) -> bool {
    let mut insts_to_delete: Vec<usize> = Vec::new();

    let mut unused_stores: HashMap<String, usize> = HashMap::new(); // <store dst, inst idx>
    // going through instructions in order
    for (inst_idx, inst) in bb.instrs.iter().enumerate() {
        match inst {
            Instruction::Opcode(inst) => match inst {
                OpcodeInstruction::Store { args } => {
                    // if any previous stores to the same location remains unused, remove
                    // everything.
                    assert!(args.len() == 2);
                    // store, location, value
                    let store_dst = args.get(0).unwrap();
                    if let Some(unused_store_inst_idx) = unused_stores.get(store_dst) {
                        insts_to_delete.push(unused_store_inst_idx.clone());
                    }
                    unused_stores.insert(store_dst.clone(), inst_idx);
                }
                OpcodeInstruction::Load { args, dest, typ } => {
                    // if anything loads from the location, it's used!
                    assert!(args.len() == 1);
                    // for all unused stores, check for aliasing with the src of this load,
                    // if they alias, the unused store should be flagged as used.
                    let load_src = args.first().unwrap();
                    let mut used_stores: Vec<String> = Vec::new();
                    for elem in unused_stores.iter() {
                        let store_dst = elem.0;
                        let _store_inst_idx = elem.1.clone();
                        if var_alias(store_dst, load_src, point_to_graph) {
                            used_stores.push(store_dst.clone());
                        }
                    }
                    for store in used_stores {
                        unused_stores.remove(&store);
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }

    let changed: bool = !insts_to_delete.is_empty();

    // pop insts in reverse order
    insts_to_delete.sort();
    insts_to_delete.reverse();

    for idx in insts_to_delete.iter() {
        bb.instrs.remove(*idx);
    }

    changed
}

fn dead_store_elimination(
    function: &mut Function,
    point_to_graph: &HashMap<String, HashSet<usize>>,
) -> bool {
    let mut changed: bool = false;
    let mut bbs = function.get_basic_blocks();
    for bb in bbs.iter_mut() {
        changed |= dead_store_elimination_bb(bb, point_to_graph);
    }
    if changed {
        function.update(bbs);
    }
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
