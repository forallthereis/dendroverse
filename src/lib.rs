//! <style>
//!     @import url('https://fonts.googleapis.com/css2?family=Fredoka:wght@500&display=swap');
//!
//!     .fredoka-title {
//!         font-family: "Fredoka", sans-serif;
//!         font-optical-sizing: auto;
//!         font-weight: 500;
//!         font-size: 6vw;
//!         font-style: normal;
//!         font-variation-settings: "wdth" 100;
//!         margin: 0;
//!         text-align: center;
//!     }
//!
//!     .gradient-title {
//!         background: #DF0000;
//!         background: linear-gradient(to bottom, #DF0000 0%, #850000 100%);
//!         -webkit-background-clip: text;
//!         -webkit-text-fill-color: transparent;
//!     }
//! </style>
//! <p class="fredoka-title">
//!     dendroverse<span class="gradient-title">.rs</span>
//! </p>
//! <br>
//!
//! ```text
//! cargo add dendroverse
//! ```
//!
//! ## 🤔 What is <span class="fredoka-title" style="font-size: 1.25rem;">dendroverse<span class="gradient-title">.rs</span></span>?
//! This is a Rust crate that enables you to solve computational problems on graphs using dynamic programming
//! over [tree decompositions][td_wiki].
//!
//! ## ✅ What this crate does for you
//! * Generates optimal tree decompsitions using the engine of your choice.
//! * Transforms tree decompositions into nice tree decompositions.
//! * Implements a **multi-threaded**, dynamic-programming-based solution process that includes job orchestration for multiple threads.
//! * Constructs a solution using backtracking.
//!
//! ## ❎ What this crate doesn't do for you
//! * It doesn’t automatically generate a solution algorithm based on the problem description
//!   ⇒ You'll have to implement payloads for each tree decomposition node yourself.
//! * It doesn't understand how a partial solution should be extended during backtracking in order to construct an answer to the problem
//!   ⇒ You'll have to implement the iterative partial solution expansion yourself.
//!   The good news here is that the <span class="fredoka-title" style="font-size: 1rem;">dendroverse<span class="gradient-title">.rs</span></span>
//!   interface makes this step as easy as possible by facilitating backlinking.
//!
//! ## 🔨 Four steps to use <span class="fredoka-title" style="font-size: 1.25rem;">dendroverse<span class="gradient-title">.rs</span></span>
//!
//! #### 1. Design a dynamic programming algorithm
//! First and foremost, you should design an algorithm to solve your specific problem using dynamic programming over tree decompositions.
//! Your algorithm can be based on either arbitrary directed tree decompositions or nice tree decompositions.
//! Either way, the important thing is to understand what should be done at each node of the tree decomposition.
//!
//! #### 2. Define a `MemoType` for your problem
//! In our generic code, we use the name `MemoType` to refer to the specific type of **memo** you use.
//! Each tree decomposition node contains a memo, which is a data structure used to store records associated with the node.
//! Memos are sometimes also called **DP-tables**.
//! When using <span class="fredoka-title" style="font-size: 1rem;">dendroverse<span class="gradient-title">.rs</span></span>,
//! it's your responsibility to implement a custom type (typically a struct) to be used as a memo.
//!
//! #### 3. Implement necessary traits
//! You'll have to implement the necessary payloads for the tree decomposition nodes yourself.
//! The exact traits you need to implement for your `MemoType` depend on the type of tree decomposition you want to use.
//! Specifically:
//! * Implement [`DTDMemo`] for your `MemoType` if your algorithm works with arbitrary tree decompositions.
//! * Implement [`NiceDTDMemo`] for your `MemoType` if your algorithm requires nice tree decompositions.
//! * Implement [`BacktrackableMemo`] for your `MemoType` if you want to retrieve solutions after having your problem instance solved.
//!
//! Furthermore, if you want to automatically generate tree decompositions for your graphs, make sure you implement
//! [`DendroverseOgGraphInterface`] for your graph type.
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
//! For this reason, <span class="fredoka-title" style="font-size: 1rem;">dendroverse<span class="gradient-title">.rs</span></span> uses a more
//! relaxed definition of a nice tree decomposition, which is intended to improve the empirical runtime.
//! Specifically, a **nice tree decomposition** is understood to be a directed tree decompsition whose nodes are divided into three categories:
//! * **Leaf nodes.**
//! Nodes with no children.
//! Their bags aren't necessarily empty.
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
//! ## 🧱 Additional feature flags
//!
//! #### Tree decomposition engines
//!
//! To generate tree decompositions automatically, you need to select the _tree decomposition engine_ (or engines) that you want
//! <span class="fredoka-title" style="font-size: 1rem;">dendroverse<span class="gradient-title">.rs</span></span> to use.
//! Each engine introduces a list of new dependencies, so to avoid adding unnecessary ones to your project, we've disabled them all by default.
//! So, don't forget to opt in!
//!
//! | Feature flag | Description |
//! |:------------:|-------------|
//! | `engine-arboretum` | Enables the [`arboretum_td`][arboretum] engine. This is the only supported engine that implements algorithms alongside heuristics to guarantee _the optimality of tree decompositions_, i.e. to construct tree decompositions with width = treewidth. |
//! | `engine-goatd` | Enables the [`goatd`][goatd] engine. It implements many heuristics and has proven effective and efficient in many use cases. |
//!
//! #### Graph library integrations
//!
//! If you use a popular graph library, you may find it tedious to implement [`DendroverseOgGraphInterface`] for it.
//! This is because Rust doesn't allow traits defined outside your crate to be implemented for types defined outside your crate.
//! Therefore, if you want to use a graph type from a third-party library with
//! <span class="fredoka-title" style="font-size: 1rem;">dendroverse<span class="gradient-title">.rs</span></span>, you'll have to
//! find a workaround, such as declaring a wrapper type, which is not ideal.
//! For this reason, we have integrated popular graph libraries for you.
//!
//! | Feature flag | Description |
//! |:------------:|-------------|
//! | `integrate-petgraph` | Integrates [`petgraph`][petgraph]. If your original graph is of type `petgraph::Graph<_, _, petgraph::Undirected, _>`, then you don't need to create a derivative type in your project and manually implement [`DendroverseOgGraphInterface`] for it. Instead, you can use your graph directly with <span class="fredoka-title" style="font-size: 1rem;">dendroverse<span class="gradient-title">.rs</span></span> after enabling this feature flag. |
//!
//! ## 🤓 Examples
//!
//! Examples of using
//! <span class="fredoka-title" style="font-size: 1rem;">dendroverse<span class="gradient-title">.rs</span></span>
//! can be found among our [integration tests][examples].
//! There, we've implemented two algorithms for finding a maximum independent set.
//! One uses arbitrary directed tree decompositions, the other relies on nice tree decompositions.
//!
//! ## 📑 Future plans
//!
//! The following features are planned for <span class="fredoka-title" style="font-size: 1rem;">dendroverse<span class="gradient-title">.rs</span></span> v0.4.0:
//! * **Custom tree decompositions.** Currently, only automatically generated tree decompositions can be used with
//! <span class="fredoka-title" style="font-size: 1rem;">dendroverse<span class="gradient-title">.rs</span></span>.
//! See the documentation for [`DendroverseInstance`] for more detail.
//! * **Solutions without backtracking.** When we solve optimisation problems on graphs, it's sometimes sufficient to retrieve an optimal objective
//! value from the tree decomposition's root node.
//! This doesn't require constructing a complete optimal solution and, hence, traversing the entire tree.
//! Currently, the traversal is unavoidable.
//!
//! [td_wiki]: https://en.wikipedia.org/wiki/Tree_decomposition
//! [arboretum]: https://docs.rs/arboretum-td/latest/arboretum_td/index.html
//! [goatd]: https://docs.rs/goatd/latest/goatd/index.html
//! [petgraph]: https://docs.rs/petgraph/latest/petgraph/
//! [examples]: https://github.com/forallthereis/dendroverse/tree/master/tests
#![cfg_attr(docsrs, feature(doc_cfg))]

use std::collections::VecDeque;

#[cfg(feature = "integrate-petgraph")]
use petgraph::visit::EdgeRef;

mod dtd;
mod solve;





/// # Instance of a problem to be solved
///
/// <style>
///     @import url('https://fonts.googleapis.com/css2?family=Fredoka:wght@500&display=swap');
///
///     .fredoka-title {
///         font-family: "Fredoka", sans-serif;
///         font-optical-sizing: auto;
///         font-weight: 500;
///         font-size: 6vw;
///         font-style: normal;
///         font-variation-settings: "wdth" 100;
///         margin: 0;
///         text-align: center;
///     }
///
///     .gradient-title {
///         background: #DF0000;
///         background: linear-gradient(to bottom, #DF0000 0%, #850000 100%);
///         -webkit-background-clip: text;
///         -webkit-text-fill-color: transparent;
///     }
/// </style>
///
/// This struct is your 'access point' to the most important functionality of
/// <span class="fredoka-title" style="font-size: 1rem;">dendroverse<span class="gradient-title">.rs</span></span>.
/// It defines an instance of a computational problem you'd like to solve using dynamic programming over tree decompositions.
///
/// ## Creating a new instance
///
/// Currently, you have the following options to create a new `DendroverseInstance`:
/// * [`DendroverseInstance::<MemoType, _>::with_auto_generated_dtd(...)`][auto_dtds]
/// Use this when:
///     * Your dynamic programming algorithm can work over arbitrary directed tree decompositions.
///     * You don't have a tree decomposition for your original graph prepared in advance and want to generate it automatically.
///     * You implemented [`DendroverseOgGraphInterface`] for your original graph type (unless [it's already implemented by us][featflags]).
///     * You implemented `Default` and [`DTDMemo`] for your `MemoType`.
/// * [`DendroverseInstance::<MemoType, _>::with_auto_generated_nice_dtd(...)`][auto_nice_dtds]
/// Use this when:
///     * Your dynamic programming algorithm requires nice tree decompositions.
///     * You don't have a tree decomposition for your original graph prepared in advance and want to generate it automatically.
///     * You implemented [`DendroverseOgGraphInterface`] for your original graph type (unless [it's already implemented by us][featflags]).
///     * You implemented `Default` and [`NiceDTDMemo`] for your `MemoType`.
///
/// ## Solving an instance
///
/// Once a `DendroverseInstance` is successfully created, it can be solved using one of the following methods:
/// * [`instance.solve_using_dtd(...)`][solve_dtds]
/// This method solves the problem using dynamic programming over **arbitrary** directed tree decompositions.
/// This method is only available when your `MemoType` implements [`DTDMemo`].
/// * [`instance.solve_using_nice_dtd(...)`][solve_nice_dtds]
/// This method solves the problem using dynamic programming over **nice** tree decompositions.
/// This method must only be used when the instance was created using one of the following functions:
///     * [`DendroverseInstance::<MemoType, _>::with_auto_generated_nice_dtd(...)`][auto_nice_dtds]
///
/// ## Retrieving a solution
///
/// Once a `DendroverseInstance` is solved, you may want to see a solution to the solved problem.
/// A solution of a solved instance can be retrieved by calling [`instance.solution()`][soln].
/// Note that this method is only available when your `MemoType` implements [`BacktrackableMemo`].
///
/// [auto_dtds]: DendroverseInstance::with_auto_generated_dtd
/// [auto_nice_dtds]: DendroverseInstance::with_auto_generated_nice_dtd
/// [featflags]: crate#graph-library-integrations
/// [solve_dtds]: DendroverseInstance::solve_using_dtd
/// [solve_nice_dtds]: DendroverseInstance::solve_using_nice_dtd
/// [soln]: DendroverseInstance::solution
pub struct DendroverseInstance<'a, MemoType, AdditionalDataType>
where
    MemoType: Send + Sync,
    AdditionalDataType: Sync,
{
    dtds: Vec<dtd::DirectedTreeDecomposition<MemoType>>,
    additional_data: &'a AdditionalDataType,
    is_instance_solved: bool,
}

impl<'a, MemoType, AdditionalDataType> DendroverseInstance<'a, MemoType, AdditionalDataType>
where
    MemoType: Send + Sync,
    AdditionalDataType: Sync,
{
    /// Creates a new `DendroverseInstance` with automatically generated directed tree decompositions for the given original graph.
    ///
    /// Here, the arguments are as follows:
    /// * `og_graph`
    /// An immutable reference to the original graph.
    /// * `additional_data`
    /// Data that can be immutably accessed from a node payload function during the solution process to check the local feasibility of
    /// the newly constructed memo entries.
    /// * `algo`
    /// A selector of an algorithm to be used for the automatic generation of a tree decomposition.
    /// See the documentation for [`DTDGenerationAlgorithm`] for more detail.
    ///
    /// Note that a separate tree decomposition will be generated for each connected component of `og_graph`.
    /// If multi-threaded dynamic programming is later used to solve the instance, the nodes from all the tree decompositions will be processed concurrently.
    /// If single-threaded dynamic programming is used instead, the nice tree decompositions will be processed sequentially, one after another, in a single thread.
    /// During backtracking, the nice tree decompositions will always be processed sequentially, one after another, in a single thread.
    ///
    /// All memos are guaranteed to initially have values `MemoType::default()`.
    /// It is also guaranteed that the bags of all nodes are stored as sorted vectors.
    ///
    /// Note also that the generated tree decompositions will be directed, however, no further properties are guaranteed.
    ///
    /// This function returns an error if the automatic generation of a tree decomposition fails.
    #[inline(always)]
    pub fn with_auto_generated_dtd<OgGraphType>(
        og_graph: &'a OgGraphType,
        additional_data: &'a AdditionalDataType,
        algo: DTDGenerationAlgorithm,
    ) -> anyhow::Result<Self>
    where
        OgGraphType: DendroverseOgGraphInterface,
        MemoType: Default + DTDMemo<AdditionalDataType>,
    {
        Ok(DendroverseInstance { dtds: dtd::auto_generate_dtds(og_graph, false, algo)?, additional_data, is_instance_solved: false })
    }

    /// Creates a new `DendroverseInstance` with automatically generated nice tree decompositions for the given original graph.
    ///
    /// Here, the arguments are as follows:
    /// * `og_graph`
    /// An immutable reference to the original graph.
    /// * `additional_data`
    /// Data that can be immutably accessed from a node payload function during the solution process to check the local feasibility of
    /// the newly constructed memo entries.
    /// * `algo`
    /// A selector of an algorithm to be used for the automatic generation of a tree decomposition.
    /// See the documentation for [`DTDGenerationAlgorithm`] for more detail.
    ///
    /// Note that a separate nice tree decomposition will be generated for each connected component of `og_graph`.
    /// If multi-threaded dynamic programming is later used to solve the instance, the nodes from all the tree decompositions will be processed concurrently.
    /// If single-threaded dynamic programming is used instead, the nice tree decompositions will be processed sequentially, one after another, in a single thread.
    /// During backtracking, the nice tree decompositions will always be processed sequentially, one after another, in a single thread.
    ///
    /// All memos are guaranteed to initially have values `MemoType::default()`.
    /// It is also guaranteed that the bags of all nodes are stored as sorted vectors.
    ///
    /// This function returns an error if the automatic generation of a nice tree decomposition fails.
    #[inline(always)]
    pub fn with_auto_generated_nice_dtd<OgGraphType>(
        og_graph: &'a OgGraphType,
        additional_data: &'a AdditionalDataType,
        algo: DTDGenerationAlgorithm,
    ) -> anyhow::Result<Self>
    where
        OgGraphType: DendroverseOgGraphInterface,
        MemoType: Default + NiceDTDMemo<AdditionalDataType>,
    {
        Ok(DendroverseInstance { dtds: dtd::auto_generate_dtds(og_graph, true, algo)?, additional_data, is_instance_solved: false })
    }

    /// Solves the `DendroverseInstance` using dynamic programming over arbitrary directed tree decompositions.
    ///
    /// Here, `threads_count` is the number of worker threads that will be spawned to execute the dynamic-programming-based solution process.
    /// For large instances, we recommend setting this argument to the number of available logical cores in your system minus one.
    /// Setting this argument to `0` or `1` will result in a single-threaded dynamic programming.
    ///
    /// This function will return an error if one occurs while solving the instance.
    ///
    /// Calling this function after the instance was already solved will do nothing.
    /// If you want to solve the problem again using a different method or number of threads, consider resetting the instance with
    /// [`instance.reset()`][reset].
    ///
    /// [reset]: DendroverseInstance::reset
    pub fn solve_using_dtd(&mut self, threads_count: usize) -> anyhow::Result<()>
    where
        MemoType: DTDMemo<AdditionalDataType>,
    {
        if self.is_instance_solved {
            return Ok(());
        }

        let result =
            if threads_count >= 2 {
                solve::solve_using_dtd_multithread(&mut self.dtds, self.additional_data, threads_count)
            } else {
                solve::solve_using_dtd_singlethread(&mut self.dtds, self.additional_data)
            };

        self.is_instance_solved = true;

        result
    }

    /// Solves the `DendroverseInstance` using dynamic programming over nice tree decompositions.
    ///
    /// Here, `threads_count` is the number of worker threads that will be spawned to execute the dynamic-programming-based solution process.
    /// For large instances, we recommend setting this argument to the number of available logical cores in your system minus one.
    /// Setting this argument to `0` or `1` will result in a single-threaded dynamic programming.
    ///
    /// This function will return an error if one occurs while solving the instance.
    ///
    /// Calling this function after the instance was already solved will do nothing.
    /// If you want to solve the problem again using a different method or number of threads, consider resetting the instance with
    /// [`instance.reset()`][reset].
    ///
    /// [reset]: DendroverseInstance::reset
    #[inline]
    pub fn solve_using_nice_dtd(&mut self, threads_count: usize) -> anyhow::Result<()>
    where
        MemoType: NiceDTDMemo<AdditionalDataType>,
    {
        if self.is_instance_solved {
            return Ok(());
        }

        let result =
            if threads_count >= 2 {
                solve::solve_using_nice_dtd_multithread(&mut self.dtds, self.additional_data, threads_count)
            } else {
                solve::solve_using_nice_dtd_singlethread(&mut self.dtds, self.additional_data)
            };

        self.is_instance_solved = true;

        result
    }

    /// Retrieves a solution to the solved `DendroverseInstance`.
    ///
    /// Returns `Some(solution)` if the `DendroverseInstance` is solved and a solution exists, `None` otherwise.
    #[inline]
    pub fn solution(&self) -> Option<MemoType::SolutionType>
    where
        MemoType: BacktrackableMemo,
    {
        if self.is_instance_solved {
            backtrack_solution_from_root_nodes(&self.dtds)
        } else {
            None
        }
    }

    /// Resets the `DendroverseInstance`.
    ///
    /// Calling this function will clear all memos and restore their values to their defaults.
    /// This enables the instance to be solved again.
    ///
    /// Note that calling this method requires your `MemoType` to implement `Default`.
    /// If your `MemoType` doesn't implement `Default`, call [`instance.reset_unchecked()`][reset_unch] instead.
    /// In this case, it'll be your responsibility to manually clear the contents of the memos at the beginning of each payload.
    ///
    /// [reset_unch]: DendroverseInstance::reset_unchecked
    pub fn reset(&mut self)
    where
        MemoType: Default,
    {
        for dtd in self.dtds.iter_mut() {
            for node in dtd.nodes.iter() {
                node.lock().unwrap().memo = MemoType::default();
            }
        }
        self.is_instance_solved = false;
    }

    /// Blindly resets the `DendroverseInstance`.
    ///
    /// Calling this function will simply mark the instance as unsolved, however, the memos of the tree decomposition nodes will keep all the data.
    /// If you call this function aiming to solve the instance again, it'll be your responsibility to manually clear the contents of the memos
    /// at the beginning of each payload.
    pub fn reset_unchecked(&mut self) {
        self.is_instance_solved = false;
    }

    /// Computes the size of the largest bag in all available tree decompositions.
    ///
    /// In other words, this value is the width of the tree decompositions plus one.
    pub fn max_bag_size(&self) -> usize {
        self
            .dtds
            .iter()
            .map(|dtd| dtd.max_bag_size)
            .max()
            .unwrap_or(0)
    }
}



/// # Selector of an automatic tree decomposition engine
///
/// Note that you must enable [a tree decomposition engine][engine] in order to see the corresponding
/// variants in this enum.
///
/// [engine]: crate#tree-decomposition-engines
pub enum DTDGenerationAlgorithm {
    /// This variant runs the [`arboretum_td`][arboretum] engine.
    ///
    /// An algorithm will be chosen automatically depending on the number of vertices in the graph.
    /// If the graph is small (up to 100 vertices), an exact algorithm ([`arboretum_td`][arboretum]'s
    /// implementation of Tamaki's PID) will be used.
    /// In other cases, different heuristics will be used instead.
    /// All algorithms apply safe reduction rules to graphs with up to 50,000 vertices.
    /// Note that parts of these algorithms are implemented recursively, so using them on large graphs will cause stack overflow.
    ///
    /// [arboretum]: https://docs.rs/arboretum-td/latest/arboretum_td/
    #[cfg(feature = "engine-arboretum")]
    ArboretumAuto,
    /// This variant runs the [`arboretum_td`][arboretum] engine.
    ///
    /// An exact algorithm ([`arboretum_td`][arboretum]'s implementation of Tamaki's PID) will be used
    /// together with safe reduction rules.
    /// Note that parts of this algorithm are implemented recursively, so using them on large graphs will cause stack overflow.
    ///
    /// [arboretum]: https://docs.rs/arboretum-td/latest/arboretum_td/
    #[cfg(feature = "engine-arboretum")]
    ArboretumExact,
    /// This variant runs the [`arboretum_td`][arboretum] engine.
    ///
    /// A tabu-search-based heuristic will be used together with safe reduction rules.
    /// Note that parts of this algorithm are implemented recursively, so using them on large graphs will cause stack overflow.
    ///
    /// [arboretum]: https://docs.rs/arboretum-td/latest/arboretum_td/
    #[cfg(feature = "engine-arboretum")]
    ArboretumHeuristic,
    /// This variant runs the [`goatd`][goatd] engine.
    ///
    /// Several heuristics will be run one after another in a single thread, then the best result will be picked as final.
    ///
    /// [goatd]: https://docs.rs/goatd/latest/goatd/index.html
    #[cfg(feature = "engine-goatd")]
    GoatdPortfolio,
}



/// # Trait for memos that support solution retrieval using backtracking
///
/// While dynamic programming traverses tree decompositions bottom-up to solve a given [`DendroverseInstance`], backtracking traverses the tree
/// decompositions of the solved instance top-down to retrieve the solution.
/// Each iterative step of this top-down traversal must be implemented by you for your `MemoType`.
///
/// Keep in mind that if you use the automatic generation of tree decompositions to create your [`DendroverseInstance`], then each connected
/// component of the original graph will have its own tree decomposition.
/// During backtracking, all tree decompositions will be processed consecutively, in a single thread.
pub trait BacktrackableMemo {
    /// The type to be used as a hint during backtracking.
    /// This is useful for backlinking.
    /// Specifically, when backlinking is used, each entry stored in a node's memo also records the IDs of the entries
    /// stored in its children's memos from which it was derived.
    /// These IDs can be passed as hints to the children's memos so that they know what entry to look up in their memos to further
    /// extend the existing partial solution.
    type BacktrackingHint;
    /// The type of a partial solution.
    /// Partial solutions are constructed step by step during backtracking and are typically extended after processing each node.
    /// At the end of the process, a partial solution is converted into a complete solution of type `Self::SolutionType`.
    type PartialSolutionType: Default + TryInto<Self::SolutionType>;
    /// The type of a complete solution that will be returned to the user.
    type SolutionType;

    /// Must extend a given partial solution by processing the memo.
    ///
    /// This function is called from [`instance.solution()`][soln] for every node of all available tree decompositions.
    /// The order in which the nodes are processed corresponds to the top-down traversal of the tree decomposition, i.e. a non-root
    /// node can only be processed if its parent is already processed.
    ///
    /// The arguments of this function have the following semantics:
    /// * `nid`
    /// The ID of the node that the memo (`self`) belongs to.
    /// This value can be used to interpret the message sent to the memo as a hint.
    /// * `partial_solution`
    /// A partial solution constructed so far.
    /// At the very start of the process (when this function is called for the memo of the root node of the first tree decomposition),
    /// the value of `partial_solution` is `Self::PartialSolutionType::default()`.
    /// * `hint`
    /// The hint passed to the memo by the parent's memo.
    /// `None` will always be passed as a hint to the root node of each available tree decomposition.
    /// This is because root nodes don't have any parents and each tree decomposition is processed independently during dynamic programming.
    /// Therefore, there's nothing to hint at in this case.
    /// * `children_nids`
    /// The IDs of all children nodes relative to the node that the memo belongs to.
    /// These may be useful for constructing hints for them.
    ///
    /// This function must return a 2-tuple with the following components:
    /// * `Option<Self::PartialSolutionType>`
    /// The extended partial solution.
    /// Value `None` must be returned if the instance turns out to have no solutions (e.g. if the optimisation problem instance is infeasible).
    /// This will immediately interrupt the backtracking and the upstream function [`instance.solution()`][soln] will also immediately return `None`.
    /// * `Vec<Option<Self::BacktrackingHint>>`
    /// Hints for the children nodes.
    /// Each _i_-th hint in the vector must correspond to the _i_-th child in `children_nids`.
    /// Since using backlinking is an option for you and not an obligation, you can always return a vector of `None` or nonsensical values.
    ///
    /// [soln]: DendroverseInstance::solution
    fn extend_partial_solution(
        &self,
        nid: usize,
        partial_solution: Self::PartialSolutionType,
        hint: Option<Self::BacktrackingHint>,
        children_nids: &Vec<usize>
    ) -> (Option<Self::PartialSolutionType>, Vec<Option<Self::BacktrackingHint>>);
}



fn backtrack_solution_from_root_nodes<MemoType>(dtds: &Vec<dtd::DirectedTreeDecomposition<MemoType>>) -> Option<MemoType::SolutionType>
where
    MemoType: BacktrackableMemo,
{
    let mut partial_solution = Some(MemoType::PartialSolutionType::default());

    for dtd in dtds.iter() {
        let mut node_queue: VecDeque<(usize, Option<MemoType::BacktrackingHint>)> = VecDeque::from([(dtd.root_nid, None)]);

        while !node_queue.is_empty() {
            let (nid, hint) = node_queue.pop_front().unwrap();
            let memo = unsafe{ &dtd.nodes.get_unchecked(nid).lock().unwrap().memo };
            let children_nids = unsafe { &dtd.adj_list.get_unchecked(nid).children_nids };
            let children_hints;

            (partial_solution, children_hints) = memo.extend_partial_solution(
                nid,
                partial_solution.unwrap(),
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



/// # Trait for original graphs supporting the automatic generation of tree decompositions
///
/// Implement this trait for your original graph type if you intend to generate tree decompositions for it
/// automatically.
pub trait DendroverseOgGraphInterface {
    /// Must return the number of vertices in the graph.
    fn vertices_count(&self) -> usize;

    /// Must return an iterator over all edges of the graph.
    ///
    /// Each edge must be represented by a 2-tuple `(usize, usize)` whose elements are the IDs of the vertices incident on it.
    /// Each edge must be undirected, the order of vertices is not important.
    fn iter_edges(&self) -> impl Iterator<Item = (usize, usize)>;
}

#[cfg(feature = "integrate-petgraph")]
impl<N, E, Ix> DendroverseOgGraphInterface for petgraph::Graph<N, E, petgraph::Undirected, Ix>
where
    Ix: petgraph::stable_graph::IndexType,
{
    fn vertices_count(&self) -> usize {
        self.node_count()
    }

    fn iter_edges(&self) -> impl Iterator<Item = (usize, usize)> {
        self.edge_references().map(|e| (e.source().index(), e.target().index()))
    }
}



/// # Trait for memos that support dynamic programming over arbitrary directed tree decompositions
///
/// Keep in mind that if you use the automatic generation of tree decompositions to create your [`DendroverseInstance`], then each connected
/// component of the original graph will have its own directed tree decomposition.
/// In this case, dynamic programming will be applied independently to each available tree decomposition.
///
/// Note also that you don't have to implement [`NiceDTDMemo`] in order to implement `DTDMemo` for your `MemoType`.
pub trait DTDMemo<AdditionalDataType> {
    /// Must populate the memo of a node.
    ///
    /// The memo to be populated is `self`.
    /// Here, the arguments are:
    /// * `is_root`
    /// Indicates whether the node owning the memo is the root of a directed tree decomposition.
    /// This is useful to filter out the entries that definitely don't represent any valid solutions.
    /// * `bag`
    /// The bag of the node owning the memo.
    /// * `children_bags`
    /// The bags of all child nodes.
    /// * `children_nids`
    /// The IDs of all child nodes.
    /// These values can be used for backlinking.
    /// * `children_memos`
    /// The memos of all child nodes.
    /// * `additional_data`
    /// The data that were passed to the [`DendroverseInstance`] constructor.
    /// These data can be used to validate the local feasibility of each new generated memo entry for `self`.
    ///
    /// Vectors `children_bags`, `children_nids` and `children_memos` are guaranteed to have the same length.
    /// The order of elements in them is consistent.
    fn payload(
        &mut self,
        is_root: bool,
        bag: &Vec<usize>,
        children_bags: Vec<&Vec<usize>>,
        children_nids: Vec<usize>,
        children_memos: Vec<&Self>,
        additional_data: &AdditionalDataType,
    );
}



/// # Trait for memos that support dynamic programming over nice tree decompositions
///
/// Keep in mind that if you use the automatic generation of tree decompositions to create your [`DendroverseInstance`], then each connected
/// component of the original graph will have its own nice tree decomposition.
/// In this case, dynamic programming will be applied independently to each available tree decomposition.
///
/// Note also that you don't have to implement [`DTDMemo`] in order to implement `NiceDTDMemo` for your `MemoType`.
pub trait NiceDTDMemo<AdditionalDataType> {
    /// Must populate the memo of a leaf node.
    ///
    /// The memo (`self`) is guaranteed to belong to a leaf node.
    /// Here, the arguments are:
    /// * `is_root`
    /// Indicates whether the node owning the memo is the root of a nice tree decomposition.
    /// This is useful to filter out the entries that definitely don't represent any valid solutions.
    /// * `bag`
    /// The bag of the node owning the memo.
    /// * `additional_data`
    /// The data that were passed to the [`DendroverseInstance`] constructor.
    /// These data can be used to validate the local feasibility of each new generated memo entry for `self`.
    fn leaf_payload(
        &mut self,
        is_root: bool,
        bag: &Vec<usize>,
        additional_data: &AdditionalDataType,
    );

    /// Must populate the memo of a forget-introduce node.
    ///
    /// The memo (`self`) is guaranteed to belong to a forget-introduce node.
    /// Here, the arguments are:
    /// * `is_root`
    /// Indicates whether the node owning the memo is the root of a nice tree decomposition.
    /// This is useful to filter out the entries that definitely don't represent any valid solutions.
    /// * `child_bag`
    /// The child node's bag.
    /// * `child_nid`
    /// The child node's ID.
    /// This value can be used for backlinking.
    /// * `child_memo`
    /// The child node's memo.
    /// * `forgotten_vids`
    /// The set of forgotten vertices.
    /// * `introduced_vids`
    /// The set of introduced vertices.
    /// * `additional_data`
    /// The data that were passed to the [`DendroverseInstance`] constructor.
    /// These data can be used to validate the local feasibility of each new generated memo entry for `self`.
    fn forget_introduce_payload(
        &mut self,
        is_root: bool,
        child_bag: &Vec<usize>,
        child_nid: usize,
        child_memo: &Self,
        forgotten_vids: &Vec<usize>,
        introduced_vids: &Vec<usize>,
        additional_data: &AdditionalDataType,
    );

    /// Must populate the memo of a join node.
    ///
    /// The memo (`self`) is guaranteed to belong to a join node.
    /// Here, the arguments are:
    /// * `is_root`
    /// Indicates whether the node owning the memo is the root of a nice tree decomposition.
    /// This is useful to filter out the entries that definitely don't represent any valid solutions.
    /// * `children_bag`
    /// The bag of the node owning the memo and the bag of both child nodes.
    /// * `child1_nid`
    /// The first child node's ID.
    /// This value can be used for backlinking.
    /// * `child1_memo`
    /// The first child node's memo.
    /// * `child2_nid`
    /// The second child node's ID.
    /// This value can be used for backlinking.
    /// * `child2_memo`
    /// The second child node's memo.
    /// * `additional_data`
    /// The data that were passed to the [`DendroverseInstance`] constructor.
    /// These data can be used to validate the local feasibility of each new generated memo entry for `self`.
    fn join_payload(
        &mut self,
        is_root: bool,
        children_bag: &Vec<usize>,
        child1_nid: usize,
        child1_memo: &Self,
        child2_nid: usize,
        child2_memo: &Self,
        additional_data: &AdditionalDataType,
    );
}
