use std::{collections::{HashSet, VecDeque}, sync::{Arc, Mutex}};

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

    /// Builds a new nice directed tree decompostion from the given Arboretum_TD tree decomposition
    pub(super) fn nice_dtd_from(td: arboretum_td::tree_decomposition::TreeDecomposition) -> Self
    where
        MemoType: Default,
    {

        let mut answer = DirectedTreeDecomposition { root_nid: 0, adj_list: Vec::new(), nodes: Vec::new() };

        // Assign the central node of the given tree decomposition to be the root
        // This is a ~heuristic~ that, as we hope, leads to the balanced branches of the nice tree decomposition
        let td_root = arboretum_td_centre(&td);

        // Add the root
        answer.add_node(
            None,
            unsafe { td.bags.get_unchecked(td_root) }.vertex_set.iter().cloned().sorted().collect(),
            MemoType::default(),
        );

        let mut unexplored_td_parent_nids = VecDeque::from([(td_root, td_root)]);
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
                    println!("Cloned parent bag for a new join node: {:?}", dtd_parent_bag);

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



/// Finds a centre of an Arboretum_TD tree decomposition
///
/// Centre = node with the minimum eccentricity.
/// Eccentricity = maximum distance to another node.
/// Distance = minimum path length.
fn arboretum_td_centre(td: &arboretum_td::tree_decomposition::TreeDecomposition) -> usize {

    let mut visited_nodes: HashSet<usize> = HashSet::from_iter(
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
