# Week3 Writeup: Loops

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

Implementation source: [`dom.rs`](../src/dom.rs)

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

Note that this algorithm also introduces BBs that may not be dominated by the header -- still, by
definition, they are a part of the natural loop(TODO: figure out why we want them to be included)

## Loop Normalization: Creating Pre-header

LICM hoists invariants to a common section that's only run once -- we create a pre-header section
that dominates all of the loop, including the header, where we can safely hoist the code.

Following pointer adjustments are required for the pre-header:
1. all bbs, except for the src of the loop back-edge, now points to the pre-header
2. the pre-header points to the header

P.S. Implementing graph-like data structure insertion/pointer adjustment in `rust`, made me appreciate 
the versatility and beauty of C pointers...

## Loop Invariant Code Motion

Implementation source: [`loop.rs`](../src/passes/loop.rs)

With the pre-header set up, for each loop, we iteratively hoist invariants until convergence:

```rust
for loop_ in loops.iter_mut() {
    while(licm_loop(loop_, &mut bbs)) {changed = true};
}
```

Note the following example uses relaxation(although relaxation is not negatively affecting the
following case) -- we need to additionally turn the loop into a do-while loop to ensure the
invariant is needed.

### Example Program

Before:
```
@main() {
    i: int = const 1;
.header:
    max: int = const 10;
    cond: bool = lt i max;
    br cond .body .exit;
.body:
    should_get_hoisted: int = const 15;
    should_get_hoisted_2: int = id should_get_hoisted;
    should_get_hoisted_3: int = add should_get_hoisted should_get_hoisted_2;
    print should_get_hoisted_3
    jmp .header;
.exit:
}
```

After Adding pre-header:
```
@main() {
    i: int = const 1;
.preheader:
.header:
    max: int = const 10;
    cond: bool = lt i max;
    br cond .body .exit;
.body:
    should_get_hoisted: int = const 15;
    should_get_hoisted_2: int = id should_get_hoisted;
    should_get_hoisted_3: int = add should_get_hoisted should_get_hoisted_2;
    print should_get_hoisted_3
    jmp .header;
.exit:
}
```

After one licm iteration:
```
@main() {
    i: int = const 1;
.preheader:
    should_get_hoisted: int = const 15;
.header:
    max: int = const 10;
    cond: bool = lt i max;
    br cond .body .exit;
.body:
    should_get_hoisted_2: int = id should_get_hoisted;
    should_get_hoisted_3: int = add should_get_hoisted should_get_hoisted_2;
    print should_get_hoisted_3
    jmp .header;
.exit:
}
```

After three licm iterations(til convergence):
```
@main() {
    i: int = const 1;
.preheader:
    should_get_hoisted: int = const 15;
    should_get_hoisted_2: int = id should_get_hoisted;
    should_get_hoisted_3: int = add should_get_hoisted should_get_hoisted_2;
.header:
    max: int = const 10;
    cond: bool = lt i max;
    br cond .body .exit;
.body:
    print should_get_hoisted_3
    jmp .header;
.exit:
}
```

One interesting observation is that loop invariants that are moved to the pre-header are more
subjective to constant propagation/DCE -- as observed in the following:

After a constant prop/DCE pass on the pre-header block:
```
@main() {
    i: int = const 1;
.preheader:
    should_get_hoisted_3: int = const 30;
.header:
    max: int = const 10;
    cond: bool = lt i max;
    br cond .body .exit;
.body:
    print should_get_hoisted_3
    jmp .header;
.exit:
}
```
