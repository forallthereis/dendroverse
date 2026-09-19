use std::{collections::VecDeque, sync::{Arc, Mutex}};

use arboretum_td::{graph::MutableGraph, solver::AtomSolver};
use fxhash::FxHashSet;
use itertools::Itertools;





pub(super) struct DirectedTreeDecomposition<MemoType> {
    pub root_nid: usize,
    pub adj_list: Vec<DTDNodeConnectivity>,
    pub nodes: Vec<Arc<Mutex<DTDNode<MemoType>>>>,
}

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
    pub(super) fn dtd_from(td: arboretum_td::tree_decomposition::TreeDecomposition) -> Self
    where
        MemoType: Default,
    {

        let mut answer = DirectedTreeDecomposition { root_nid: 0, adj_list: Vec::new(), nodes: Vec::new() };

        // Assign the central node of the given tree decomposition to be the root
        // This is a ~heuristic~ that, as we hope, leads to the balanced branches of the nice tree decomposition
        let td_root_nid = arboretum_td_centre(&td);

        // Add the root
        answer.add_node(
            None,
            unsafe { td.bags.get_unchecked(td_root_nid) }.vertex_set.iter().cloned().sorted().collect(),
            MemoType::default(),
        );

        let mut unmapped_td_nids = VecDeque::from_iter(
            unsafe { td.bags.get_unchecked(td_root_nid) }.neighbors.iter().cloned().map(|nid| (nid, td_root_nid))
        );
        let mut nid_map = vec![0; td.bags.len()];

        while let Some((td_nid, td_parent_nid)) = unmapped_td_nids.pop_front() {

            let dtd_parent_nid = unsafe { *nid_map.get_unchecked(td_parent_nid) };
            let dtd_nid = answer.add_node(
                Some(dtd_parent_nid),
                unsafe { td.bags.get_unchecked(td_nid) }.vertex_set.iter().cloned().sorted().collect(),
                MemoType::default(),
            );
            unsafe { *nid_map.get_unchecked_mut(td_nid) = dtd_nid; }

            unmapped_td_nids.extend(
                unsafe { td.bags.get_unchecked(td_nid) }
                    .neighbors
                    .iter()
                    .filter_map(|nid| if *nid != td_parent_nid { Some((*nid, td_parent_nid)) } else { None })
            );

        }

        answer

    }

    /// Builds a new nice directed tree decompostion from the given Arboretum_TD tree decomposition
    pub(super) fn nice_dtd_from(td: arboretum_td::tree_decomposition::TreeDecomposition) -> Self
    where
        MemoType: Default,
    {

        let mut answer = DirectedTreeDecomposition { root_nid: 0, adj_list: Vec::new(), nodes: Vec::new() };

        // Assign the central node of the given tree decomposition to be the root
        // This is a ~heuristic~ that, as we hope, leads to the balanced branches of the nice tree decomposition
        let td_root_nid = arboretum_td_centre(&td);

        // Add the root
        answer.add_node(
            None,
            unsafe { td.bags.get_unchecked(td_root_nid) }.vertex_set.iter().cloned().sorted().collect(),
            MemoType::default(),
        );

        let mut unexplored_td_parent_nids = VecDeque::from([(td_root_nid, td_root_nid)]);
        let mut unmapped_td_children_nids = VecDeque::new();
        let mut nid_map = vec![0; td.bags.len()];

        while !unexplored_td_parent_nids.is_empty() {

            let (td_parent_nid, td_grandparent_nid) = unexplored_td_parent_nids.pop_front().unwrap();
            let mut dtd_parent_nid = unsafe { *nid_map.get_unchecked(td_parent_nid) };

            unmapped_td_children_nids.extend(
                unsafe { td.bags.get_unchecked(td_parent_nid) }
                    .neighbors
                    .iter()
                    .filter_map(|nid| if *nid != td_grandparent_nid { Some(*nid) } else { None })
            );

            while !unmapped_td_children_nids.is_empty() {

                let td_child_nid = unmapped_td_children_nids.pop_front().unwrap();

                // If there're no other td-children, add an forget-introduce node for the current td-child
                if unmapped_td_children_nids.is_empty() {

                    let dtd_child_nid = answer.add_node(
                        Some(dtd_parent_nid),
                        unsafe { td.bags.get_unchecked(td_child_nid) }.vertex_set.iter().cloned().sorted().collect(),
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
                        unsafe { td.bags.get_unchecked(td_child_nid) }.vertex_set.iter().cloned().sorted().collect(),
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



pub(super) fn auto_generate_dtds<MemoType, OgGraphType>(
    og_graph: &OgGraphType,
    make_nice: bool,
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

    if ccs.len() == 1 {
        let td_generator = arboretum_td::exact::TamakiPid::with_graph(&og_graph_arboretum);
        dtds.push(
            match td_generator.compute() {
                arboretum_td::solver::ComputationResult::Bounds(_) => return Err(anyhow::anyhow!("The automatic generation of the tree decomposition failed.")),
                arboretum_td::solver::ComputationResult::ComputedTreeDecomposition(td) =>
                    match make_nice {
                        true => DirectedTreeDecomposition::nice_dtd_from(td),
                        false => DirectedTreeDecomposition::dtd_from(td),
                    },
            }
        );

        return Ok(dtds)
    }

    for cc_vids in ccs {
        let cc = og_graph_arboretum.vertex_induced_subgraph(&cc_vids);
        let cc_td_generator = arboretum_td::exact::TamakiPid::with_graph(&cc);
        dtds.push(
            match cc_td_generator.compute() {
                arboretum_td::solver::ComputationResult::Bounds(_) => return Err(anyhow::anyhow!("The automatic generation of the tree decomposition failed.")),
                arboretum_td::solver::ComputationResult::ComputedTreeDecomposition(td) =>
                match make_nice {
                    true => DirectedTreeDecomposition::nice_dtd_from(td),
                    false => DirectedTreeDecomposition::dtd_from(td),
                },
            }
        );
    }

    Ok(dtds)

}



/// Finds a centre of an Arboretum_TD tree decomposition
///
/// Centre = node with the minimum eccentricity.
/// Eccentricity = maximum distance to another node.
/// Distance = minimum path length.
fn arboretum_td_centre(td: &arboretum_td::tree_decomposition::TreeDecomposition) -> usize {

    let mut visited_nodes: FxHashSet<usize> = FxHashSet::from_iter(
        td
            .bags
            .iter()
            .filter_map(|bag| if bag.neighbors.len() == 1 { Some(bag.id) } else { None })
    );
    let mut node_queue = VecDeque::from_iter(
        td
            .bags
            .iter()
            .filter_map(
                |bag| if !visited_nodes.contains(&bag.id) && bag.neighbors.iter().filter(|nid| !visited_nodes.contains(nid)).count() <= 1 { Some(bag.id) } else { None }
            )
    );

    let mut nid = 0;

    while !node_queue.is_empty() {

        nid = node_queue.pop_front().unwrap();
        let node_queue_front_nid = node_queue.front().copied();

        visited_nodes.insert(nid);

        let interesting_neighbours =
            unsafe { td.bags.get_unchecked(nid) }
                .neighbors
                .iter()
                .filter(|adj_nid| !visited_nodes.contains(*adj_nid) && node_queue_front_nid != Some(**adj_nid));

        for adj_nid in interesting_neighbours {
            if unsafe { td.bags.get_unchecked(*adj_nid) }.neighbors.iter().filter(|adj_adj_nid| !visited_nodes.contains(adj_adj_nid)).count() <= 1 {
                node_queue.push_back(*adj_nid);
            }
        }

    }

    nid

}
