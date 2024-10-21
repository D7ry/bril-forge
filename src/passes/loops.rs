use crate::ast;
use crate::dom;
use ast::*;
use dom::*;

use std::collections::{HashSet, VecDeque};

struct Loop {
    pub header_idx: usize,
    pub back_node_idx: usize, // node that back-edges back to the header
    pub nodes: Vec<usize>,    // all nodes execept for header and back node
}

// create a pre-header block and inserts it to the basic blocks right before the block with
// `header_idx`.
// All bbs that originally flow to the header, flows to the pre-header, execept for the end of the
// loop with `back_node_idx`, which still flows to the original header.
//
// the created pre-header takes over `header_idx`, and the old header uses `header_idx + 1`
// note that all external state indices to bbs that are originally >= `header_idx` should be incremented by one
fn create_and_insert_pre_header(
    bbs: &mut Vec<BasicBlock>,
    header_idx: usize,
    mut back_node_idx: usize,
) {
    let old_header: &mut BasicBlock = bbs.get_mut(header_idx).unwrap();
    let old_header_new_idx = header_idx + 1; // new idx of old header, post insertion
                                             //
    let mut pre_header: BasicBlock = BasicBlock {
        instrs: Vec::new(),
        in_bb_indices: old_header.in_bb_indices.clone(), // all bbs that points to header now points to pre-header
        out_bb_indices: HashSet::from([old_header_new_idx]), // pre-header points to old header -- inserting it
                                                             // right before header allows the code path to
                                                             // directly go to the old header
    };

    old_header.in_bb_indices.clear();
    old_header.in_bb_indices.insert(header_idx); // after insertion, pre-header takes old header's
                                                 // idx

    let mut old_header_label: Option<String> = None;
    let mut pre_header_label: Option<String> = None;
    // for consistency, pre-header takes header's label as well
    if let Some(inst) = old_header.instrs.first_mut() {
        match inst {
            Instruction::Label { label } => {
                pre_header.instrs.push(Instruction::Label {
                    label: label.clone(),
                }); // pre-header now uses old header's label
                pre_header_label = Some(label.clone());
                label.push_str("@old"); // push old label to the header so nobody can jump to it
                old_header_label = Some(label.clone());
            }
            _ => {}
        }
    }
    // pre-header conveniently takes header's idx, so out_bb_indices of all blocks that jumps
    // to the old header don't need to be changed
    bbs.insert(header_idx, pre_header);

    if header_idx < back_node_idx {
        back_node_idx += 1;
    }

    // mutate indices(won't be necessary if only we can have pointers)
    for (idx, bb) in bbs.iter_mut().enumerate() {
        let mut new_out_bb_indices: HashSet<usize> = HashSet::new();
        let mut new_in_bb_indices: HashSet<usize> = HashSet::new();

        for i in bb.out_bb_indices.iter() {
            let mut new_idx = i.clone();
            // bbs whose indices are bigger than pre-header needs to increment index by one due to insertion.
            if new_idx > header_idx {
                new_idx += 1;
            }
            // if new idx == header_idx, we don't do any change -- we intend any BBs that jumps to
            // the old header to now jump to the pre-header
            new_out_bb_indices.insert(new_idx);
        }

        for i in bb.in_bb_indices.iter() {
            let mut new_idx = i.clone();
            // bbs whose indices are bigger than pre-header needs to increment index by one due to insertion.
            // also, if new idx == header_idx, we increment it too, because out(old_header) does
            // not change
            if new_idx >= header_idx {
                new_idx += 1;
            }

            new_in_bb_indices.insert(new_idx);
        }
        bb.out_bb_indices = new_out_bb_indices;
        bb.in_bb_indices = new_in_bb_indices;

        if idx == back_node_idx {
            // special case: the back node should point to the old header
            assert!(bb.out_bb_indices.contains(&header_idx));
            bb.out_bb_indices.remove(&header_idx);
            bb.out_bb_indices.insert(old_header_new_idx);
            if let Some(old_header_label) = old_header_label.clone() {
                let pre_header_label = pre_header_label.clone().unwrap();
                // ensure the label points to the old header as well
                for inst in bb.instrs.iter_mut() {
                    // change the labels to be pointing to old header...
                    match inst {
                        Instruction::Opcode(inst) => match inst {
                            OpcodeInstruction::Br { labels, .. } => {
                                for label in labels.iter_mut() {
                                    if *label == pre_header_label {
                                        *label = old_header_label.clone();
                                    }
                                }
                            }
                            OpcodeInstruction::Jmp { labels } => {
                                for label in labels.iter_mut() {
                                    if *label == pre_header_label {
                                        *label = old_header_label.clone();
                                    }
                                }
                            }
                            _ => {}
                        },
                        _ => {}
                    }
                }
            }
        }
    }
}


fn licm_loop(loop_: &mut Loop, bbs: &mut Vec<BasicBlock>) -> bool {
    let mut changed: bool = false;


    changed
}

// function-scope licm
fn licm_function(function: &mut Function) -> bool {
    let mut changed: bool = false;
    let mut bbs: Vec<BasicBlock> = function.get_basic_blocks();

    let dom_context: DomContext = get_dom_context(&bbs);

    let mut loops: Vec<Loop> = Vec::new();

    // find loops using back-edges, by itearting over all edges and check dom tree(technically can
    // also use dfs to figure this out, without dom tree)
    for (bb_idx, bb) in bbs.iter().enumerate() {
        let bb_dom_context: &BBDomContext;
        unsafe {
            bb_dom_context = dom_context.bbs.get_unchecked(bb_idx);
        }
        for out_bb_idx in bb.out_bb_indices.iter() {
            // we have a back edge if the dest of an edge dominates src
            let is_back_edge: bool = bb_dom_context.dominators.contains(out_bb_idx);
            if is_back_edge {
                let new_loop: Loop = Loop {
                    header_idx: out_bb_idx.clone(),
                    back_node_idx: bb_idx,
                    nodes: Vec::new(),
                };
                loops.push(new_loop);
            }
        }
    }

    // add pre-header
    for loop_ in loops.iter_mut() {
        create_and_insert_pre_header(&mut bbs, loop_.header_idx, loop_.back_node_idx);
    }

    // populate loop nodes
    for loop_ in loops.iter_mut() {
        let starting_node = loop_.back_node_idx.clone();
        let mut work_list: Vec<usize> = vec![starting_node];
        let mut processed: HashSet<usize> = HashSet::new();
        processed.insert(starting_node);

        while let Some(node_bb_idx) = work_list.pop() {
            // add all predecessors of the current node, that are not the header node, to the wl as well as
            // the nodes list
            let bb;
            unsafe {
                bb = bbs.get_unchecked(node_bb_idx);
            }
            for parent_idx in bb.in_bb_indices.iter() {
                if parent_idx.clone() == loop_.header_idx || processed.contains(parent_idx) {
                    continue;
                }
                processed.insert(parent_idx.clone());
                loop_.nodes.push(parent_idx.clone());
                work_list.push(parent_idx.clone());
            }
        }
    }

    for loop_ in loops.iter_mut() {
        changed |= licm_loop(loop_, &mut bbs);
    }

    if changed {
        function.update(bbs);
    }
    changed
}

// performs licm, hoisting loop invariants
pub fn loop_invariant_code_motion_pass(program: &mut Program) -> bool {
    let mut changed: bool = false;

    for function in program.functions.iter_mut() {
        changed |= licm_function(function);
    }

    changed
}
