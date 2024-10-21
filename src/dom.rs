// Dominator-tree construction and utilities

use crate::ast::*;
use std::collections::HashSet;

// dominance info of a basic block
pub struct BBDomContext {
    pub dominators: HashSet<usize>, // indices to BB that dominates this BB
}

// dominance context of a function's basic blocks
pub struct DomContext {
    pub bbs: Vec<BBDomContext>,
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
    if !bbs.is_empty() {
        visit(0, bbs, &mut ordering, &mut visited);
    }

    ordering
}

fn get_reverse_post_order_traversal_ordering(bbs: &Vec<BasicBlock>) -> Vec<usize> {
    let mut ret = get_post_order_traversal_ordering(bbs);
    ret.reverse();
    ret
}

fn get_set_intersection(sets: &Vec<HashSet<usize>>) -> HashSet<usize> {
    if sets.is_empty() {
        return HashSet::new();
    }
    let smallest_set = sets.iter().min_by_key(|set| set.len()).unwrap();

    let intersection_set = smallest_set
        .iter()
        .filter(|&&elem| sets.iter().all(|set| set.contains(&elem)))
        .cloned()
        .collect();

    intersection_set
}

// get the dominance context of the function's bbs using reverse post-order traversal.
pub fn get_dom_context(bbs: &Vec<BasicBlock>) -> DomContext {
    let mut ctx: DomContext = DomContext { bbs: Vec::new() };

    // initialize ctx with empty data
    for _i in 0..bbs.len() {
        ctx.bbs.push(BBDomContext {
            dominators: HashSet::new(),
        })
    }

    let reverse_post_ordering = get_reverse_post_order_traversal_ordering(&bbs);

    // visit the bbs in reverse post order, calculating dom context for each bb
    for bb_idx in reverse_post_ordering.iter() {
        let bb: &BasicBlock;
        unsafe {
            bb = bbs.get_unchecked(bb_idx.clone());
        }

        // in (bb) = and(out(parent) for all parent in parent(bb) + bb

        let mut all_parent_dominators: Vec<HashSet<usize>> = Vec::new();

        // take intersection of all parents' dominators

        for parent_bb_idx in bb.in_bb_indices.iter() {
            let parent_bb_ctx;
            unsafe {
                parent_bb_ctx = ctx.bbs.get_unchecked_mut(parent_bb_idx.clone());
            }

            all_parent_dominators.push(parent_bb_ctx.dominators.clone());
        }

        let parent_dominators_itersection: HashSet<usize> =
            get_set_intersection(&all_parent_dominators);

        let bb_dom_ctx: &mut BBDomContext;
        unsafe {
            bb_dom_ctx = ctx.bbs.get_unchecked_mut(bb_idx.clone());
        }

        // shared dominators of parents dominate the child
        bb_dom_ctx.dominators = parent_dominators_itersection;
        // bb dominates itself
        bb_dom_ctx.dominators.insert(bb_idx.clone());
    }

    ctx
}
