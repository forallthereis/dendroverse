```text
cargo add dendroverse
```

## 🤔 What is `dendroverse.rs`?
This is a Rust crate that enables you to solve computational problems on graphs using dynamic programming
over [tree decompositions][td_wiki].

## ✅ What this crate does for you
* Generates optimal tree decompsitions (using [`arboretum_td`][arboretum] under the hood).
* Transforms tree decompositions into nice tree decompositions.
* Implements a **multi-threaded**, dynamic-programming-based solution process that includes job orchestration for multiple threads.
* Constructs a solution using backtracking.

## ❎ What this crate doesn't do for you
* It doesn’t automatically generate a solution algorithm based on the problem description
  ⇒ You'll have to implement payloads for each tree decomposition node yourself.
* It doesn't understand how a partial solution should be extended during backtracking in order to construct an answer to the problem
  ⇒ You'll have to implement the iterative partial solution expansion yourself.
  The good news here is that the `dendroverse.rs` interface makes this step as easy as possible by facilitating backlinking.

## 🔨 Four steps to use `dendroverse.rs`

#### 1. Design a dynamic programming algorithm
First and foremost, you should design an algorithm to solve your specific problem using dynamic programming over tree decompositions.
Your algorithm can be based on either arbitrary directed tree decompositions or nice tree decompositions.
Either way, the important thing is to understand what should be done at each node of the tree decomposition.

#### 2. Define a `MemoType` for your problem
In our generic code, we use the name `MemoType` to refer to the specific type of **memo** you use.
Each tree decomposition node contains a memo, which is a data structure used to store records associated with the node.
Memos are sometimes also called **DP-tables**.
When using `dendroverse.rs`, it's your responsibility to implement a custom type (typically a struct) to be used as a memo.

#### 3. Implement necessary traits
You'll have to implement the necessary payloads for the tree decomposition nodes yourself.
The exact traits you need to implement for your `MemoType` depend on the type of tree decomposition you want to use.
Specifically:
* Implement `DTDMemo` for your `MemoType` if your algorithm works with arbitrary tree decompositions.
* Implement `NiceDTDMemo` for your `MemoType` if your algorithm requires nice tree decompositions.
* Implement `BacktrackableMemo` for your `MemoType` if you want to retrieve solutions after having your problem instance solved.

#### 4. Create and solve a `DendroverseInstance`
When everything's set up, you can finally create and solve a `DendroverseInstance`.
See its documentation for more details.

## 🌲 Relaxed notion of nice tree decompositions

The definition of a nice tree decomposition varies from source to source.
Generally, the nodes of a nice tree decomposition are divided into four categories:
_leaf_, _introduce_, _forget_ and _join_ nodes.
Sometimes the root node or the leaf nodes are required to have empty bags.
However, these definitions were originally designed to facilitate the **theoretical description** of an algorithm rather than its practical implementation.

For this reason, `dendroverse.rs` uses a more relaxed definition of a nice tree decomposition, which is intended to improve the empirical runtime.
Specifically, a **nice tree decomposition** is understood to be a directed tree decompsition whose nodes are divided into three categories:
* **Leaf nodes.**
Nodes with no children.
* **Forget-introduce nodes.**
Forget and introduce nodes are merged into one category.
Similar to the classical nice tree decompositions, these nodes have exactly one child.
However, several vertices of the original graph can be forgotten and introduced at once.
Hence, the bag of a forget-introduce node can be represented as
(\<the bag of the child node\> \\ \<the set of forgotten vertices\>) ∪ \<the set of introduced vertices\>.
* **Join node.**
Nodes with exactly two children.
The bags of the children must be exactly the same as the bag of the join node, just like in the classical nice tree decompositions.

Note that a classical nice tree decomposition is a special case of a relaxed nice tree decomposition and, hence, can still be used.

## 🧱 Additional feature flags

The default functionality of `dendroverse.rs` can be extended with the following feature flags:

| Feature flag | Description |
|:------------:|-------------|
| `integrate-petgraph` | If you work with graphs in Rust, you likely use [`petgraph`][petgraph]. In this case, if your original graph is of type `petgraph::Graph<_, _, petgraph::Undirected, usize>`, then you don't need to create a derivative type in your project and manually implement `DendroverseOgGraphInterface` for it. Instead, you can use your graph directly with `dendroverse.rs` by enabling this feature flag. |

## 🤓 Examples

Examples of using `dendroverse.rs` can be found among our [integration tests][examples].
There, we've implemented two algorithms for finding a maximum independent set.
One uses arbitrary directed tree decompositions, the other relies on nice tree decompositions.

## 📑 Future plans

The following features are planned for `dendroverse.rs`:
* **Custom tree decompositions.** Currently, only automatically generated tree decompositions can be used with `dendroverse.rs`.
See the documentation for `DendroverseInstance` for more detail.
* **Solutions without backtracking.** When we solve optimisation problems on graphs, it's sometimes sufficient to retrieve an optimal objective
value from the tree decomposition's root node.
This doesn't require constructing a complete optimal solution and, hence, traversing the entire tree.
Currently, the traversal is unavoidable.

[td_wiki]: https://en.wikipedia.org/wiki/Tree_decomposition
[arboretum]: https://docs.rs/arboretum-td/latest/arboretum_td/index.html
[petgraph]: https://docs.rs/petgraph/latest/petgraph/
[examples]: https://github.com/forallthereis/dendroverse/tree/main/tests
