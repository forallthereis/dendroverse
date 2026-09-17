//! # dendroverse.rs
//!
//! ```text
//! cargo add dendroverse
//! ```
//!
//! ## 🤔 What is `dendroverse.rs`?
//! This is a Rust crate that enables you to solve computational problems on graphs using dynamic programming
//! over [tree decompositions][td_wiki].
//!
//! ## ✅ What this crate does for you
//! * Generates optimal tree decompsitions (using [`arboretum_td`][arboretum] under the hood).
//! * Transforms tree decompositions into nice tree decompositions.
//! * Implements a **multi-threaded** dynamic-programming-based solution process, including the orchestration of jobs for multiple threads.
//! * Constructs an explicit solution using backtracking.
//!
//! ## ❎ What this crate doesn't do for you
//! * It doesn’t automatically generate a solution algorithm based on the problem description
//!   ⇒ You'll have to implement payloads for each tree decomposition node yourself.
//! * It doesn't understand how a partial solution should be extended during backtracking in order to construct an answer to the problem
//!   ⇒ You'll have to implement the iterative partial solution expansion yourself.
//!   The good news here is that the `dendroverse.rs` interface makes this step as easy as possible by facilitating backlinking.
//!
//! ## 🔨 Four steps to use `dendroverse.rs`
//!
//! #### 1. Design a dynamic programming algorithm
//! First and foremost, you should design an algorithm to solve your specific problem using tree decompositions.
//! Your algorithm can be based on either arbitrary directed tree decompositions or nice tree decompositions.
//! Either way, the important thing is to understand what should be done at each node of the tree decomposition.
//!
//! #### 2. Define a `MemoType` for your problem
//! In our generic code, we use the name `MemoType` to refer to the specific type of **memo** you use.
//! Each tree decomposition node contains a memo, which is a data structure used to store the partial solutions associated with the node.
//! Memos are sometimes also called **DP-tables**.
//! When using `dendroverse.rs`, it's your responsibility to implement a custom type (typically a struct) to be used as a memo.
//!
//! #### 3. Implement necessary traits
//! You'll have to implement the necessary payloads for the tree decomposition nodes yourself.
//! The exact traits you need to implement for your `MemoType` depend on the type of tree decomposition you want to use.
//!
//! #### 4. Create and solve a [`DendroverseInstance`]
//! When everything's set up, you can finally create and solve a [`DendroverseInstance`].
//! See its documentation for more details.
//!
//! ## 🌲 Relaxed notion of nice tree decompositions
//!
//! The definition of a nice tree decomposition varies from source to source.
//! Generally, the nodes of a nice tree decomposition are divided into four categories:
//! _leaf_, _introduce_, _forget_ and _join_ nodes.
//! Sometimes the root node or the leaf nodes are required to have empty bags.
//! However, these definitions were originally designed to facilitate the **theoretical description** of an algorithm rather than its practical implementation.
//!
//! For this reason, `dendroverse.rs` uses a more relaxed definition of a nice tree decomposition, which is intended to improve the empirical runtime.
//! Specifically, a **nice tree decomposition** is understood to be a directed tree decompsition whose nodes are divided into three categories:
//! * **Leaf nodes.**
//! Nodes with no children.
//! * **Forget-introduce nodes.**
//! Forget and introduce nodes are merged into one category.
//! Similar to the classical nice tree decompositions, these nodes have exactly one child.
//! However, several vertices of the original graph can be forgotten and introduced at once.
//! Hence, the bag of a forget-introduce node can be represented as
//! (\<the bag of the child node\> \\ \<the set of forgotten vertices\>) ∪ \<the set of introduced vertices\>.
//! * **Join node.**
//! Nodes with exactly two children.
//! The bags of the children must be exactly the same as the bag of the join node, just like in the classical nice tree decompositions.
//!
//! Note that a classical nice tree decomposition is a special case of a relaxed nice tree decomposition and, hence, can still be used.
//!
//! [td_wiki]: https://en.wikipedia.org/wiki/Tree_decomposition
//! [arboretum]: https://docs.rs/arboretum-td/latest/arboretum_td/index.html
use std::collections::VecDeque;

use arboretum_td::{graph::MutableGraph, solver::AtomSolver};

mod dtd;
mod solve;





/// # Instance of a problem to be solved
///
/// This struct is your 'access point' to the most important functionality of `dendroverse.rs`.
/// It defines an instance of a computational problem you'd like to solve using dynamic programming over tree decompositions.
///
/// ## Creating a new instance
///
/// Currently, you have the following options to create a new `DendroverseInstance`:
/// * [`DendroverseInstance::<MemoType, _>::with_auto_generated_nice_dtd(...)`][auto_nice_dtds]
/// Use this when you have an original graph and you want to solve your problem using dynamic programming over nice tree decompositions.
/// This function will generate an optimal nice tree decomposition for your graph automatically.
/// Note that if you want to use this option, your data must satisfy the following additional requirements:
///     * Your original graph must be of type that implements [`DendroverseOgGraphInterface`].
///     * Your `MemoType` must implement [`NiceDTDMemo`].
///
/// ## Solving an instance
///
/// Once a `DendroverseInstance` is successfully created, it can be solved using one of the following methods:
/// * [`instance.solve_using_nice_dtd(...)`][solve_nice_dtds]
/// This method solves the problem using dynamic programming over nice tree decompositions.
/// This method must only be used when the instance was created using one of the following functions:
///     * [`DendroverseInstance::<MemoType, _>::with_auto_generated_nice_dtd(...)`][auto_nice_dtds]
///
/// ## Retrieving an answer
///
/// Once a `DendroverseInstance` is solved, you usually want to see the answer to the solved problem.
/// It can be retrieved by calling [`instance.answer()`][answer].
/// Note that this method is only available when your `MemoType` implements [`BacktrackableMemo`].
///
/// [auto_nice_dtds]: DendroverseInstance::with_auto_generated_nice_dtd
/// [solve_nice_dtds]: DendroverseInstance::solve_using_nice_dtd
/// [answer]: DendroverseInstance::answer
pub struct DendroverseInstance<'a, MemoType, AdditionalDataType>
where
    MemoType: Default + Send + Sync,
    AdditionalDataType: Sync,
{
    dtds: Vec<dtd::DirectedTreeDecomposition<MemoType>>,
    additional_data: &'a AdditionalDataType,
    is_instance_solved: bool,
}

impl<'a, MemoType, AdditionalDataType> DendroverseInstance<'a, MemoType, AdditionalDataType>
where
    MemoType: Default + Send + Sync,
    AdditionalDataType: Sync,
{

    /// Retrieves an answer to the solved problem
    ///
    /// Returns `Some(answer)` if the `DendroverseInstance` is solved and the instance is feasible, `None` otherwise.
    #[inline]
    pub fn answer(&self) -> Option<MemoType::AnswerType>
    where
        MemoType: BacktrackableMemo,
    {
        if self.is_instance_solved {
            backtrack_answer_from_root_nodes(&self.dtds)
        } else {
            None
        }
    }

    /// Creates a new `DendroverseInstance` with automatically generated nice tree decompositions for the given original graph.
    ///
    /// Here, the arguments are as follows:
    /// * `og_graph`
    /// An immutable reference to the original graph.
    /// * `additional_data`
    /// Data that can be immutably accessed from a node payload function during the solution process to check the feasibility of
    /// the newly constructed partial solutions.
    ///
    /// Note that a separate nice tree decomposition will be generated for each connected component of `og_graph`.
    /// If multi-thread dynamic programming is used, the nodes from all the tree decompositions will be processed concurrently.
    /// If single-thread dynamic programming is used, the decompositions will be processed sequentially.
    /// During backtracking, the nice tree decompositions are always processed one after another in a single thread.
    pub fn with_auto_generated_nice_dtd<OgGraphType>(
        og_graph: &'a OgGraphType,
        additional_data: &'a AdditionalDataType,
    ) -> anyhow::Result<Self>
    where
        OgGraphType: DendroverseOgGraphInterface,
        MemoType: Default + NiceDTDMemo<AdditionalDataType>,
    {

        let mut og_graph_arboretum = arboretum_td::graph::HashMapGraph::with_capacity(og_graph.vertices_count());
        for vid in 0..og_graph.vertices_count() {
            og_graph_arboretum.add_vertex(vid);
        }
        for (vid1, vid2) in og_graph.iter_edges() {
            og_graph_arboretum.add_edge(vid1, vid2);
        }

        let mut dtds = Vec::new();
        let ccs = og_graph_arboretum.connected_components();

        if ccs.len() == 1 {
            let td_generator = arboretum_td::exact::TamakiPid::with_graph(&og_graph_arboretum);
            dtds.push(
                match td_generator.compute() {
                    arboretum_td::solver::ComputationResult::Bounds(_) => return Err(anyhow::anyhow!("The automatic generation of the nice tree decomposition failed.")),
                    arboretum_td::solver::ComputationResult::ComputedTreeDecomposition(td) => dtd::DirectedTreeDecomposition::nice_dtd_from(td),
                }
            );

            return Ok(DendroverseInstance { dtds, additional_data, is_instance_solved: false })
        }

        for cc_vids in ccs {
            let cc = og_graph_arboretum.vertex_induced_subgraph(&cc_vids);
            let cc_td_generator = arboretum_td::exact::TamakiPid::with_graph(&cc);
            dtds.push(
                match cc_td_generator.compute() {
                    arboretum_td::solver::ComputationResult::Bounds(_) => return Err(anyhow::anyhow!("The automatic generation of the nice tree decomposition failed.")),
                    arboretum_td::solver::ComputationResult::ComputedTreeDecomposition(td) => dtd::DirectedTreeDecomposition::nice_dtd_from(td),
                }
            );
        }

        Ok(DendroverseInstance { dtds, additional_data, is_instance_solved: false })

    }

    /// Solves a `DendroverseInstance` using dynamic programming over nice tree decompositions
    ///
    /// Here, `threads_count` is the number of worker threads that will be spawned to execute the dynamic-programming-based solution process.
    /// We recommend setting this argument to the number of logical cores available in your system minus one.
    /// Setting this argument to `0` or `1` will result in a single-thread dynamic programming.
    #[inline(always)]
    pub fn solve_using_nice_dtd(&mut self, threads_count: usize) -> anyhow::Result<()>
    where
        MemoType: NiceDTDMemo<AdditionalDataType>,
    {
        let result = solve::solve_using_nice_dtd(&mut self.dtds, self.additional_data, threads_count);
        self.is_instance_solved = true;
        result
    }

}



pub trait BacktrackableMemo {

    type AnswerType;
    type BacktrackingHint;
    type PartialAnswerType: Default + TryInto<Self::AnswerType>;

    fn extend_partial_solution(
        &self,
        nid: usize,
        partial_solution: Option<Self::PartialAnswerType>,
        hint: Option<Self::BacktrackingHint>,
        children_nids: &Vec<usize>
    ) -> (Option<Self::PartialAnswerType>, Vec<Option<Self::BacktrackingHint>>);

}



fn backtrack_answer_from_root_nodes<MemoType>(dtds: &Vec<dtd::DirectedTreeDecomposition<MemoType>>) -> Option<MemoType::AnswerType>
where
    MemoType: BacktrackableMemo,
{

    let mut partial_solution = Some(MemoType::PartialAnswerType::default());

    for dtd in dtds.iter() {

        let mut node_queue: VecDeque<(usize, Option<MemoType::BacktrackingHint>)> = VecDeque::from([(dtd.root_nid, None)]);

        while !node_queue.is_empty() {

            let (nid, hint) = node_queue.pop_front().unwrap();
            let memo = unsafe{ &dtd.nodes.get_unchecked(nid).lock().unwrap().memo };
            let children_nids = unsafe { &dtd.adj_list.get_unchecked(nid).children_nids };
            let children_hints;

            (partial_solution, children_hints) = memo.extend_partial_solution(
                nid,
                partial_solution,
                hint,
                children_nids,
            );

            if partial_solution.is_none() {
                return None;
            }

            node_queue.extend(children_nids.iter().cloned().zip(children_hints.into_iter()));

        }

    }

    partial_solution.unwrap().try_into().ok()

}



pub trait DendroverseOgGraphInterface {
    fn vertices_count(&self) -> usize;
    fn iter_edges(&self) -> impl Iterator<Item = (usize, usize)>;
}



pub trait NiceDTDMemo<AdditionalDataType> {

    fn leaf_payload(
        &mut self,
        bag: &Vec<usize>,
        additional_data: &AdditionalDataType,
    );

    fn forget_introduce_payload(
        &mut self,
        child_bag: &Vec<usize>,
        child_nid: usize,
        child_memo: &Self,
        forgotten_vids: &Vec<usize>,
        introduced_vids: &Vec<usize>,
        additional_data: &AdditionalDataType,
    );

    fn join_payload(
        &mut self,
        children_bag: &Vec<usize>,
        child1_nid: usize,
        child1_memo: &Self,
        child2_nid: usize,
        child2_memo: &Self,
        additional_data: &AdditionalDataType,
    );

}
