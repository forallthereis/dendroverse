use std::{collections::VecDeque, sync::{Arc, Mutex}};

#[cfg(feature = "engine-arboretum")]
use arboretum_td::{graph::MutableGraph};
use fxhash::FxHashSet;
use itertools::Itertools;





pub(super) struct DirectedTreeDecomposition<MemoType> {
    pub max_bag_size: usize,
    pub root_nid: usize,
    pub adj_list: Vec<DTDNodeConnectivity>,
    pub nodes: Vec<Arc<Mutex<DTDNode<MemoType>>>>,
}

#[allow(unused)]
impl<'a, MemoType> DirectedTreeDecomposition<MemoType> {

    /// Adds a new node to a directed tree decomposition and returns its node ID
    fn add_node(&'a mut self, parent_nid: Option<usize>, bag: Vec<usize>, memo: MemoType) -> usize {
        self.adj_list.push(DTDNodeConnectivity { parent_nid, children_nids: Vec::new() });
        self.nodes.push(
            Arc::new(
                Mutex::new(
                    DTDNode { bag, memo }
                )
            )
        );

        let child_nid = self.adj_list.len() - 1;

        if let Some(parent_nid_value) = parent_nid {
            unsafe { self.adj_list.get_unchecked_mut(parent_nid_value) }.children_nids.push(child_nid);
        }

        child_nid
    }

    /// Returns a borrowing iterator that iterates over all leaves in the directed tree decomposition
    #[inline(always)]
    pub(super) fn iter_leaves(&'a self) -> DTDLeafIter<'a, MemoType> {
        DTDLeafIter { dtd: self, nid: 0 }
    }

    /// Builds a new directed tree decomposition from the given Arboretum_TD tree decomposition
    fn dtd_from<ExternalTDtype>(td: ExternalTDtype, vid_map_option: Option<Vec<usize>>) -> Self
    where
        ExternalTDtype: ExternalTreeDecomposition,
        MemoType: Default,
    {
        let mut answer = DirectedTreeDecomposition { max_bag_size: td.max_bag_size(), root_nid: 0, adj_list: Vec::new(), nodes: Vec::new() };

        // Assign the central node of the given tree decomposition to be the root
        // This is a ~heuristic~ that, as we hope, leads to the balanced branches of the nice tree decomposition
        let td_root_nid = external_td_centre(&td);

        // Add the root
        answer.add_node(
            None,
            match &vid_map_option {
                Some(vid_map) => td.iter_bag(td_root_nid).map(|vid| * unsafe { vid_map.get_unchecked(vid) }).sorted().collect(),
                None => td.iter_bag(td_root_nid).sorted().collect(),
            },
            MemoType::default(),
        );

        let mut unmapped_td_nids = VecDeque::from_iter(
            td.iter_adj_nodes(td_root_nid).map(|nid| (nid, td_root_nid))
        );
        let mut nid_map = vec![0; td.node_count()];

        while let Some((td_nid, td_parent_nid)) = unmapped_td_nids.pop_front() {
            let dtd_parent_nid = unsafe { *nid_map.get_unchecked(td_parent_nid) };
            let dtd_nid = answer.add_node(
                Some(dtd_parent_nid),
                match &vid_map_option {
                    Some(vid_map) => td.iter_bag(td_nid).map(|vid| * unsafe { vid_map.get_unchecked(vid) }).sorted().collect(),
                    None => td.iter_bag(td_nid).sorted().collect(),
                },
                MemoType::default(),
            );
            unsafe { *nid_map.get_unchecked_mut(td_nid) = dtd_nid; }

            unmapped_td_nids.extend(
                td
                    .iter_adj_nodes(td_nid)
                    .filter_map(|nid| if nid != td_parent_nid { Some((nid, td_nid)) } else { None })
            );
        }

        answer
    }

    /// Builds a new nice directed tree decompostion from the given external tree decomposition
    fn nice_dtd_from<ExternalTDType>(td: ExternalTDType, vid_map_option: Option<Vec<usize>>) -> Self
    where
        ExternalTDType: ExternalTreeDecomposition,
        MemoType: Default,
    {
        let mut answer = DirectedTreeDecomposition { max_bag_size: td.max_bag_size(), root_nid: 0, adj_list: Vec::new(), nodes: Vec::new() };

        // Assign the central node of the given tree decomposition to be the root
        // This is a ~heuristic~ that, as we hope, leads to the balanced branches of the nice tree decomposition
        let td_root_nid = external_td_centre(&td);

        // Add the root
        answer.add_node(
            None,
            match &vid_map_option {
                Some(vid_map) => td.iter_bag(td_root_nid).map(|vid| * unsafe { vid_map.get_unchecked(vid) }).sorted().collect(),
                None => td.iter_bag(td_root_nid).sorted().collect(),
            },
            MemoType::default(),
        );

        let mut unexplored_td_parent_nids = VecDeque::from([(td_root_nid, td_root_nid)]);
        let mut unmapped_td_children_nids = VecDeque::new();
        let mut nid_map = vec![0; td.node_count()];

        while !unexplored_td_parent_nids.is_empty() {
            let (td_parent_nid, td_grandparent_nid) = unexplored_td_parent_nids.pop_front().unwrap();
            let mut dtd_parent_nid = unsafe { *nid_map.get_unchecked(td_parent_nid) };

            unmapped_td_children_nids.extend(
                td
                    .iter_adj_nodes(td_parent_nid)
                    .filter_map(|nid| if nid != td_grandparent_nid { Some(nid) } else { None })
            );

            while !unmapped_td_children_nids.is_empty() {
                let td_child_nid = unmapped_td_children_nids.pop_front().unwrap();

                // If there're no other td-children, add an forget-introduce node for the current td-child
                if unmapped_td_children_nids.is_empty() {
                    let dtd_child_nid = answer.add_node(
                        Some(dtd_parent_nid),
                        match &vid_map_option {
                            Some(vid_map) => td.iter_bag(td_child_nid).map(|vid| * unsafe { vid_map.get_unchecked(vid) }).sorted().collect(),
                            None => td.iter_bag(td_child_nid).sorted().collect(),
                        },
                        MemoType::default(),
                    );
                    unsafe { *nid_map.get_unchecked_mut(td_child_nid) = dtd_child_nid; }

                    unexplored_td_parent_nids.push_back((td_child_nid, td_parent_nid));
                // Otherwise, make the current dtd-parent a join node with an forget-introduce node for the current td-child
                } else {
                    let dtd_parent_bag = unsafe { answer.nodes.get_unchecked(dtd_parent_nid) }.lock().unwrap().bag.clone();

                    let dtd_join_branch1_nid = answer.add_node(
                        Some(dtd_parent_nid),
                        dtd_parent_bag.clone(),
                        MemoType::default(),
                    );
                    let dtd_join_branch2_nid = answer.add_node(
                        Some(dtd_parent_nid),
                        dtd_parent_bag,
                        MemoType::default(),
                    );
                    let dtd_child_nid = answer.add_node(
                        Some(dtd_join_branch1_nid),
                        match &vid_map_option {
                            Some(vid_map) => td.iter_bag(td_child_nid).map(|vid| * unsafe { vid_map.get_unchecked(vid) }).sorted().collect(),
                            None => td.iter_bag(td_child_nid).sorted().collect(),
                        },
                        MemoType::default(),
                    );
                    unsafe { *nid_map.get_unchecked_mut(td_child_nid) = dtd_child_nid; }

                    unexplored_td_parent_nids.push_back((td_child_nid, td_parent_nid));

                    dtd_parent_nid = dtd_join_branch2_nid;
                }
            }
        }

        answer
    }
}



pub(super) struct DTDNodeConnectivity {
    pub parent_nid: Option<usize>,
    pub children_nids: Vec<usize>,
}



pub(super) struct DTDNode<MemoType> {
    pub bag: Vec<usize>,
    pub memo: MemoType,
}



pub(super) struct DTDLeafIter<'a, MemoType> {
    dtd: &'a DirectedTreeDecomposition<MemoType>,
    nid: usize,
}

impl<'a, MemoType> Iterator for DTDLeafIter<'a, MemoType> {
    type Item = (usize, Arc<Mutex<DTDNode<MemoType>>>);

    fn next(&mut self) -> Option<Self::Item> {
        while self.nid < self.dtd.adj_list.len() {
            self.nid += 1;
            if unsafe { self.dtd.adj_list.get_unchecked(self.nid - 1).children_nids.is_empty() } {
                return Some((self.nid - 1, Arc::clone( unsafe { self.dtd.nodes.get_unchecked(self.nid - 1) } )));
            }
        }
        None
    }
}



trait ExternalTreeDecomposition {
    fn iter_adj_nodes(&self, nid: usize) -> impl Iterator<Item = usize>;
    fn iter_bag(&self, nid: usize) -> impl Iterator<Item = usize>;
    fn iter_leaf_nids(&self) -> impl Iterator<Item = usize>;
    fn iter_neighbourhoods(&self) -> impl Iterator<Item = (usize, impl Iterator<Item = usize>)>;
    fn max_bag_size(&self) -> usize;
    fn node_count(&self) -> usize;
}

#[cfg(feature = "engine-arboretum")]
impl ExternalTreeDecomposition for arboretum_td::tree_decomposition::TreeDecomposition {
    #[inline(always)]
    fn iter_adj_nodes(&self, nid: usize) -> impl Iterator<Item = usize> {
        unsafe { self.bags.get_unchecked(nid) }.neighbors.iter().copied()
    }

    #[inline(always)]
    fn iter_bag(&self, nid: usize) -> impl Iterator<Item = usize> {
        unsafe { self.bags.get_unchecked(nid) }.vertex_set.iter().copied()
    }

    #[inline(always)]
    fn iter_leaf_nids(&self) -> impl Iterator<Item = usize> {
        self
            .bags
            .iter()
            .filter_map(|bag| if bag.neighbors.len() <= 1 { Some(bag.id) } else { None })
    }

    fn iter_neighbourhoods(&self) -> impl Iterator<Item = (usize, impl Iterator<Item = usize>)> {
        self
            .bags
            .iter()
            .map(|bag| (bag.id, bag.neighbors.iter().copied()))
    }

    #[inline(always)]
    fn max_bag_size(&self) -> usize {
        self.max_bag_size
    }

    #[inline(always)]
    fn node_count(&self) -> usize {
        self.bags.len()
    }
}

#[cfg(feature = "engine-goatd")]
impl ExternalTreeDecomposition for goatd::TreeDecomposition {
    #[inline(always)]
    fn iter_adj_nodes(&self, nid: usize) -> impl Iterator<Item = usize> {
        unsafe { self.adjacency().get_unchecked(nid) }.iter().copied()
    }

    #[inline(always)]
    fn iter_bag(&self, nid: usize) -> impl Iterator<Item = usize> {
        unsafe { self.bags().get_unchecked(nid) }.vertices().iter().map(|vid| *vid as usize)
    }

    #[inline(always)]
    fn iter_leaf_nids(&self) -> impl Iterator<Item = usize> {
        self
            .adjacency()
            .iter()
            .enumerate()
            .filter_map(|(nid, neighbours)| if neighbours.len() <= 1 { Some(nid) } else { None })
    }

    #[inline(always)]
    fn iter_neighbourhoods(&self) -> impl Iterator<Item = (usize, impl Iterator<Item = usize>)> {
        self
            .adjacency()
            .iter()
            .enumerate()
            .map(|(nid, neighbours)| (nid, neighbours.iter().copied()))
    }

    #[inline(always)]
    fn max_bag_size(&self) -> usize {
        self.treewidth() as usize + 1
    }

    #[inline(always)]
    fn node_count(&self) -> usize {
        self.bags().len()
    }
}



#[allow(unused_variables)]
pub(super) fn auto_generate_dtds<MemoType, OgGraphType>(
    og_graph: &OgGraphType,
    make_nice: bool,
    algo: crate::DTDGenerationAlgorithm,
) -> anyhow::Result<Vec<DirectedTreeDecomposition<MemoType>>>
where
    MemoType: Default,
    OgGraphType: crate::DendroverseOgGraphInterface,
{
    match algo {
        #[cfg(feature = "engine-arboretum")]
        crate::DTDGenerationAlgorithm::ArboretumAuto
        | crate::DTDGenerationAlgorithm::ArboretumExact
        | crate::DTDGenerationAlgorithm::ArboretumHeuristic => auto_generate_dtds_arboretum_td(og_graph, make_nice, algo),

        #[cfg(feature = "engine-goatd")]
        crate::DTDGenerationAlgorithm::GoatdPortfolio => auto_generate_dtds_goatd(og_graph, make_nice, algo),
    }
}



#[allow(unreachable_patterns)]
#[cfg(feature = "engine-arboretum")]
fn auto_generate_dtds_arboretum_td<MemoType, OgGraphType>(
    og_graph: &OgGraphType,
    make_nice: bool,
    algo: crate::DTDGenerationAlgorithm,
) -> anyhow::Result<Vec<DirectedTreeDecomposition<MemoType>>>
where
    MemoType: Default,
    OgGraphType: crate::DendroverseOgGraphInterface,
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

    if ccs.len() <= 1 {
        let td_generator = match algo {
            crate::DTDGenerationAlgorithm::ArboretumAuto => arboretum_td::solver::Solver::auto(&og_graph_arboretum)
                .seed(Some(0))
                .safe_separator_limits(
                    arboretum_td::SafeSeparatorLimits::default().use_min_degree_for_minor_safe(true),
                ),
            crate::DTDGenerationAlgorithm::ArboretumExact => arboretum_td::solver::Solver::default_exact()
                .seed(Some(0))
                .safe_separator_limits(
                    arboretum_td::SafeSeparatorLimits::default().use_min_degree_for_minor_safe(true),
                ),
            crate::DTDGenerationAlgorithm::ArboretumHeuristic => arboretum_td::solver::Solver::default_heuristic()
                .safe_separator_limits(
                    arboretum_td::SafeSeparatorLimits::default().use_min_degree_for_minor_safe(true),
                )
                .seed(Some(0)),
            _ => unreachable!(),
        };

        let td = td_generator.solve(&og_graph_arboretum);

        dtds.push(
            match make_nice {
                true => DirectedTreeDecomposition::nice_dtd_from(td, None),
                false => DirectedTreeDecomposition::dtd_from(td, None),
            }
        );

        return Ok(dtds)
    }

    for cc_vids in ccs {
        let cc = og_graph_arboretum.vertex_induced_subgraph(&cc_vids);

        let cc_td_generator = match algo {
            crate::DTDGenerationAlgorithm::ArboretumAuto => arboretum_td::solver::Solver::auto(&cc)
                .seed(Some(0))
                .safe_separator_limits(
                    arboretum_td::SafeSeparatorLimits::default().use_min_degree_for_minor_safe(true),
                ),
            crate::DTDGenerationAlgorithm::ArboretumExact => arboretum_td::solver::Solver::default_exact()
                .seed(Some(0))
                .safe_separator_limits(
                    arboretum_td::SafeSeparatorLimits::default().use_min_degree_for_minor_safe(true),
                ),
            crate::DTDGenerationAlgorithm::ArboretumHeuristic => arboretum_td::solver::Solver::default_heuristic()
                .safe_separator_limits(
                    arboretum_td::SafeSeparatorLimits::default().use_min_degree_for_minor_safe(true),
                )
                .seed(Some(0)),
            _ => unreachable!(),
        };

        let cc_td = cc_td_generator.solve(&cc);

        dtds.push(
            match make_nice {
                true => DirectedTreeDecomposition::nice_dtd_from(cc_td, None),
                false => DirectedTreeDecomposition::dtd_from(cc_td, None),
            }
        );
    }

    Ok(dtds)
}



#[allow(unreachable_patterns)]
#[cfg(feature = "engine-goatd")]
fn auto_generate_dtds_goatd<MemoType, OgGraphType>(
    og_graph: &OgGraphType,
    make_nice: bool,
    algo: crate::DTDGenerationAlgorithm,
) -> anyhow::Result<Vec<DirectedTreeDecomposition<MemoType>>>
where
    MemoType: Default,
    OgGraphType: crate::DendroverseOgGraphInterface,
{
    let og_graph_goatd = goatd::Graph::new(
        og_graph.vertices_count() as u32,
        og_graph.iter_edges().map(|(vid1, vid2)| (vid1 as u32, vid2 as u32)),
    );

    let mut dtds = Vec::new();
    let ccs = connected_components_goatd(&og_graph_goatd);

    if ccs.len() <= 1 {
        let td = match algo {
            crate::DTDGenerationAlgorithm::GoatdPortfolio => {
                let weights = vec![1; og_graph_goatd.num_vertices() as usize];
                goatd::portfolio::decompose(&og_graph_goatd, &weights, 0, goatd::portfolio::PortfolioConfig::standard())?
            },
            _ => unreachable!(),
        };

        dtds.push(
            match make_nice {
                true => DirectedTreeDecomposition::nice_dtd_from(td, None),
                false => DirectedTreeDecomposition::dtd_from(td, None),
            }
        );

        return Ok(dtds)
    }

    for cc_vids in ccs {
        let cc = og_graph_goatd.induced_subgraph(&cc_vids).unwrap();

        let cc_td = match algo {
            crate::DTDGenerationAlgorithm::GoatdPortfolio => {
                let weights = vec![1; og_graph_goatd.num_vertices() as usize];
                goatd::portfolio::decompose(&cc, &weights, 0, goatd::portfolio::PortfolioConfig::standard())?
            },
            _ => unreachable!(),
        };

        dtds.push(
            match make_nice {
                true => DirectedTreeDecomposition::nice_dtd_from(cc_td, Some(cc_vids.into_iter().map(|vid| vid as usize).collect())),
                false => DirectedTreeDecomposition::dtd_from(cc_td, Some(cc_vids.into_iter().map(|vid| vid as usize).collect())),
            }
        );
    }

    Ok(dtds)
}



#[cfg(feature = "engine-goatd")]
fn connected_components_goatd(td: &goatd::Graph) -> Vec<Vec<u32>> {
    // let adjacency = goatd:: Adjacency::of(self);
    let vertex_count = td.num_vertices() as usize;

    let mut visited = vec![false; vertex_count];
    let mut components = Vec::new();

    for start in 0..vertex_count {
        if visited[start] {
            continue;
        }

        // Start a new component with an unvisited vertex.
        let mut component = Vec::new();
        let mut stack = vec![start as u32];
        visited[start] = true;

        // Iterative depth-first search.
        while let Some(vid) = stack.pop() {
            component.push(vid);

            for (_, adj_vid) in td.edges().iter().filter(|(vid1, _)| *vid1 == vid) {
                let adj_vid_usize = *adj_vid as usize;

                if !visited[adj_vid_usize] {
                    visited[adj_vid_usize] = true;
                    stack.push(*adj_vid);
                }
            }
        }

        // Keep the IDs deterministic and compatible with the
        // local vertex numbering used by induced_subgraph().
        // component.sort_unstable();
        components.push(component);
    }

    components
}



/// Finds a centre of an external tree decomposition
///
/// Centre = node with the minimum eccentricity.
/// Eccentricity = maximum distance to another node.
/// Distance = minimum path length.
fn external_td_centre<ExternalTDType>(td: &ExternalTDType) -> usize
where
    ExternalTDType: ExternalTreeDecomposition,
{
    let mut visited_nodes: FxHashSet<usize> = FxHashSet::from_iter(td.iter_leaf_nids());
    let mut node_queue = VecDeque::from_iter(
        td
            .iter_neighbourhoods()
            .filter_map(
                |(nid, adj_nids)| if !visited_nodes.contains(&nid) && adj_nids.filter(|adj_nid| !visited_nodes.contains(adj_nid)).count() <= 1 { Some(nid) } else { None }
            )
    );

    let mut nid = 0;

    while !node_queue.is_empty() {
        nid = node_queue.pop_front().unwrap();
        let node_queue_front_nid = node_queue.front().copied();

        visited_nodes.insert(nid);

        let interesting_neighbours =
            td
                .iter_adj_nodes(nid)
                .filter(|adj_nid| !visited_nodes.contains(adj_nid) && node_queue_front_nid != Some(*adj_nid));

        for adj_nid in interesting_neighbours {
            if td.iter_adj_nodes(adj_nid).filter(|adj_adj_nid| !visited_nodes.contains(adj_adj_nid)).count() <= 1 {
                node_queue.push_back(adj_nid);
            }
        }
    }

    nid
}
