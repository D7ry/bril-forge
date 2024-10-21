# Week2 Writeup: Loops

All optimization passes are verified with bril programs in `benchmarks` to ensure correctness.

## Finding Loops

Finding loops involves finding the DFS backedge of the program's CFG. One can do it with:
1. performing a DFS and use start&end time to find back edges.
    - this takes `O(V+E)` time
2. a dom tree -- `edge(u,v)` is a back edge if `v` dominates `u`
    - this takes `O(V+E)` (DFS) + `O(V+E)`(reverse-post order traversal of vertices and edges) +
      `O(E)` (testing edges)

Given dom tree is useful for analysis passes other than loops, we can amortize dom tree generation
cost -- therefore we choose the dom tree approach.

### Dom Tree

Implementation source: [`dom.rs`](../src/passes/dom.rs)

To generate the dom-tree, we perform reverse post-order traversal of all the bbs in a CFG using a forward dataflow analysis, where:
- `meet()` is the set-intersection of parents' dominators
- `gen()` is the bb itself
- `kill()` is undefined

The property of reverse post-ordering allows us to complete dominator analysis in a single pass, as
all parent bb of a bb that can potentailly dominate the bb have been processed.

### Finding Back Edges Using Dom Tree

Finding back edges with dom tree is trivial, we simply iterate over all edges, and check the
dominator relation between source and dest.

### Finding The Rest of the BBs belonging to the loop

We used a simple worklist algorithm to iteratively add all bbs that could, by definition, "reach the
end node without going through the header":

```rust
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
```

## Optimizing Loop Structure: Creating Pre-header

LICM hoists invariants to a common section that's only run once -- we create a pre-header section
that dominates all of the loop, including the header, where we can safely hoist the code.

Following pointer adjustments are required for the pre-header:
1. all bbs, except for the src of the loop back-edge, now points to the pre-header
2. the pre-header points to the header

P.S. Implementing graph-like data structure insertion/pointer adjustment in `rust`, made me appreciate 
the versatility and beauty of C pointers...

## Loop Invariant Code Motion

With the pre-header set up, 



### Example Program

