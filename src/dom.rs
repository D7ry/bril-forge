// Dominator-tree construction and utilities

use crate::ast::*;
use std::collections::HashSet;

// dominance info of a basic block
struct BBDomInfo {
    dominated: HashSet<usize>,  // indices to BB that are dominated by this BB
    dominators: HashSet<usize>, // indices to BB that dominates this BB
}

// dominance context of a function's basic blocks
struct DomContext {
    bbs: Vec<BBDomInfo>,
}

fn get_post_order_traversal_ordering(bbs: &Vec<BasicBlock>) -> Vec<usize> {
    fn visit(
        bb_idx: usize,
        bbs: &Vec<BasicBlock>,
        ordering: &mut Vec<usize>,
        visited: &mut HashSet<usize>,
    ) {
        assert!(!visited.contains(&bb_idx)); // mut have been not visited before

        let bb: &BasicBlock;
        unsafe {
            bb = bbs.get_unchecked(bb_idx);
        }

        // visit all children first
        for child_idx in bb.out_bb_indices.iter() {
            if !visited.contains(child_idx) {
                visit(child_idx.clone(), bbs, ordering, visited);
            }
        }

        // finally add self to ordering
        ordering.push(bb_idx);

        // mark as visited
        visited.insert(bb_idx);
    }

    let mut ordering: Vec<usize> = Vec::new();
    let mut visited: HashSet<usize> = HashSet::new();
    // note that the root bb dominates all other reacheable bbs, so we can simply go from root bb
    visit(0, bbs, &mut ordering, &mut visited);

    ordering
}

fn get_reverse_post_order_traversal_ordering(bbs: &Vec<BasicBlock>) -> Vec<usize> {
    let mut ret = get_post_order_traversal_ordering(bbs);
    ret.reverse();
    ret
}

// get the dominance context of the function's bbs using reverse post-order traversal.
pub fn get_dom_context(function: Function) -> DomContext {
    let mut ctx: DomContext = DomContext { bbs: Vec::new() };

    let bbs: Vec<BasicBlock> = function.get_basic_blocks();

    // initialize ctx with empty data
    for _i in 0..bbs.len() {
        ctx.bbs.push(BBDomInfo {
            dominated: HashSet::new(),
            dominators: HashSet::new(),
        })
    }

    let reverse_post_ordering = get_reverse_post_order_traversal_ordering(&bbs);

    // visit the bbs in reverse post order, calculating dom context for each bb
    for bb_idx in reverse_post_ordering.iter() {
        let bb: &BasicBlock;
        let bb_dom_ctx: &mut BBDomInfo;
        unsafe {
            bb = bbs.get_unchecked(bb_idx.clone());
            bb_dom_ctx = ctx.bbs.get_unchecked_mut(bb_idx.clone());
        }
        
        // in (bb) = and(out(parent) for all parent in parent(bb) + bb
        
        // take intersection of all parents' dominators
        
        // also register bb as dominated by parents


        // bb dominates itself
        bb_dom_ctx.dominated.insert(bb_idx.clone());
    }

    ctx
}
