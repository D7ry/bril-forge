use crate::ast;
use crate::dom;
use ast::*;
use dom::*;

use std::collections::HashSet;

struct Loop {
    header_idx: usize,
    back_node_idx: usize, // node that back-edges back to the header
    nodes: Vec<usize>,    // all nodes execept for header and back node
}

// create a pre-header block and inserts it to the basic blocks right before the block with
// `header_idx`.
// All bbs that originally flow to the header, flows to the pre-header, execept for the end of the
// loop with `back_node_idx`, which still flows to the original header.
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
                            OpcodeInstruction::Br { labels, ..} => {
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

// function-scope licm
fn licm_function(function: &mut Function) -> bool {
    let mut changed: bool = false;
    let mut bbs: Vec<BasicBlock> = function.get_basic_blocks();

    let dom_context: DomContext = get_dom_context(&bbs);

    let loops: Vec<Loop> = Vec::new();

    // find loops using back-edges, by itearting over all edges and check dom tree(technically can
    // also use DFS to figure this out, without dom tree)



    // for all loops, insert a pre-header -- useful for hoisting invariants!
    for l in loops.iter() {}

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
