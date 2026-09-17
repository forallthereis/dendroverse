use dendroverse;
use fxhash::{FxBuildHasher, FxHashMap, FxHashSet};
use itertools::Itertools;
use petgraph::{Graph, Undirected, visit::EdgeRef};

use crate::MaxIndependentSetMemoAncestorEntryId::{NoAncestor, SingleAncestor, TwoAncestors};





struct MyGraph(Graph<(), (), Undirected, usize>);

impl dendroverse::DendroverseOgGraphInterface for MyGraph {
    fn iter_edges(&self) -> impl Iterator<Item = (usize, usize)> {
        self.0.edge_references().map(|e| (e.source().index(), e.target().index()))
    }

    fn vertices_count(&self) -> usize {
        self.0.node_count()
    }
}



#[derive(Clone, Copy, Debug)]
enum MaxIndependentSetMemoAncestorEntryId {
    NoAncestor,
    SingleAncestor(usize),
    TwoAncestors{
        child1_nid: usize,
        ancestor_entry1_id: usize,
        ancestor_entry2_id: usize,
    },
}

impl Default for MaxIndependentSetMemoAncestorEntryId {
    fn default() -> Self {
        Self::NoAncestor
    }
}



#[derive(Clone, Debug, Default)]
struct MaxIndependentSetMemoEntry {
    indep_set: Vec<usize>,
    obj_value: usize,
    ancestor_memo_entry_id: MaxIndependentSetMemoAncestorEntryId,
}



#[derive(Debug, Default)]
struct MaxIndependentSetMemo(Vec<MaxIndependentSetMemoEntry>);



impl dendroverse::NiceDTDMemo<MyGraph> for MaxIndependentSetMemo {

    fn leaf_payload(
        &mut self,
        bag: &Vec<usize>,
        additional_data: &MyGraph,
    )
    {

        for vs in bag.iter().powerset() {
            if is_independent(&vs, additional_data) {
                let obj_value = vs.len();
                self.0.push(MaxIndependentSetMemoEntry { indep_set: vs.into_iter().cloned().collect(), obj_value, ancestor_memo_entry_id: NoAncestor });
            }
        }

    }

    fn forget_introduce_payload(
        &mut self,
        _child_bag: &Vec<usize>,
        _child_nid: usize,
        child_memo: &Self,
        forgotten_vids: &Vec<usize>,
        introduced_vids: &Vec<usize>,
        additional_data: &MyGraph,
    )
    {

        let mut memo_entries_collector =
            FxHashMap
                ::<Vec<usize>, (usize, usize)>
                ::with_capacity_and_hasher(child_memo.0.len() * (1 << introduced_vids.len()), FxBuildHasher::new());

        for (child_memo_entry_id, child_memo_entry) in child_memo.0.iter().enumerate() {

            let vertex_set_wo_forgotten = subtract_sorted_vecs(&child_memo_entry.indep_set, forgotten_vids);

            for introduced_vertex_set in introduced_vids.iter().powerset() {

                let vertex_set =
                    vertex_set_wo_forgotten
                        .iter()
                        .cloned()
                        .merge(introduced_vertex_set.iter().map(|nid| **nid))
                        .collect::<Vec<usize>>();

                if !is_independent_with_added(&vertex_set_wo_forgotten, &introduced_vertex_set, additional_data) {
                    continue;
                }

                let obj_value = child_memo_entry.obj_value + introduced_vertex_set.len();

                match memo_entries_collector.get(&vertex_set) {

                    Some((collected_obj_value, _)) =>
                        if obj_value > *collected_obj_value {
                            memo_entries_collector.insert(vertex_set, (obj_value, child_memo_entry_id));
                        },

                    None => {memo_entries_collector.insert(vertex_set, (obj_value, child_memo_entry_id));},

                }

            }

        }

        self.0 =
            memo_entries_collector
                .into_iter()
                .map(|(vertex_set, (obj_value, child_memo_entry_id))| MaxIndependentSetMemoEntry { indep_set: vertex_set, obj_value, ancestor_memo_entry_id: SingleAncestor(child_memo_entry_id) })
                .collect();

    }

    fn join_payload(
        &mut self,
        _children_bag: &Vec<usize>,
        child1_nid: usize,
        child1_memo: &Self,
        _child2_nid: usize,
        child2_memo: &Self,
        _additional_data: &MyGraph,
    )
    {

        let mut unmatched_memo_entries_collector =
            FxHashMap
                ::<Vec<usize>, (usize, usize)>
                ::from_iter(child1_memo.0.iter().enumerate().map(|(i, e)| (e.indep_set.clone(), (e.obj_value, i))));

        for (child2_memo_entry_id, child2_memo_entry) in child2_memo.0.iter().enumerate() {

            let child2_vertex_set = &child2_memo_entry.indep_set;

            if !unmatched_memo_entries_collector.contains_key(child2_vertex_set) {
                continue;
            }

            let (child1_memo_entry_obj_value, child1_memo_entry_id) = unmatched_memo_entries_collector.remove(child2_vertex_set).unwrap();
            let memo_entry_obj_value = child1_memo_entry_obj_value + child2_memo_entry.obj_value - child2_vertex_set.len();
            let memo_entry_ancestor_entry_id = TwoAncestors{
                child1_nid,
                ancestor_entry1_id: child1_memo_entry_id,
                ancestor_entry2_id: child2_memo_entry_id,
            };

            self.0.push(
                MaxIndependentSetMemoEntry {
                    indep_set: child2_vertex_set.clone(),
                    obj_value: memo_entry_obj_value,
                    ancestor_memo_entry_id: memo_entry_ancestor_entry_id,
                }
            );

        }

    }

}



impl dendroverse::BacktrackableMemo for MaxIndependentSetMemo {

    type AnswerType = FxHashSet<usize>;
    type BacktrackingHint = usize;
    type PartialAnswerType = FxHashSet<usize>;

    fn extend_partial_solution(
        &self,
        _nid: usize,
        partial_solution: Option<Self::PartialAnswerType>,
        hint: Option<Self::BacktrackingHint>,
        children_nids: &Vec<usize>
    ) -> (Option<Self::PartialAnswerType>, Vec<Option<Self::BacktrackingHint>>)
    {

        let mut partial_solution = partial_solution.unwrap();

        let memo_entry_id =
            match hint {

                Some(hinted_entry_id) => hinted_entry_id,

                None => self.0.iter().enumerate().max_by_key(|(_, memo_entry)| memo_entry.obj_value).unwrap().0,

            };

        let entry = &self.0[memo_entry_id];

        partial_solution.extend(entry.indep_set.iter().cloned());

        let new_hint =
            match entry.ancestor_memo_entry_id {

                NoAncestor => vec![],

                SingleAncestor(ancestor_entry_id) => vec![Some(ancestor_entry_id)],

                TwoAncestors { child1_nid, ancestor_entry1_id, ancestor_entry2_id } =>
                    if child1_nid == children_nids[0] {
                        vec![Some(ancestor_entry1_id), Some(ancestor_entry2_id)]
                    } else {
                        vec![Some(ancestor_entry2_id), Some(ancestor_entry1_id)]
                    },

            };

        (Some(partial_solution), new_hint)

    }

}



fn is_independent(vertex_set: &Vec<&usize>, og_graph: &MyGraph) -> bool {
    for i in 0..vertex_set.len() {
        for j in (i + 1)..vertex_set.len() {
            if og_graph.0.contains_edge((*vertex_set[i]).into(), (*vertex_set[j]).into()) {
                return false;
            }
        }
    }
    true
}



fn is_independent_with_added(
    indep_vertex_set: &Vec<usize>,
    added_vertex_set: &Vec<&usize>,
    og_graph: &MyGraph
) -> bool {
    for i in 0..added_vertex_set.len() {
        for j in (i + 1)..added_vertex_set.len() {
            if og_graph.0.contains_edge((*added_vertex_set[i]).into(), (*added_vertex_set[j]).into()) {
                return false;
            }
        }
    }
    for added_vid in added_vertex_set {
        for indep_vid in indep_vertex_set {
            if og_graph.0.contains_edge((**added_vid).into(), (*indep_vid).into()) {
                return false;
            }
        }
    }
    true
}



fn subtract_sorted_vecs(minuend: &Vec<usize>, subtrahend: &Vec<usize>) -> Vec<usize> {

    let mut answer = Vec::with_capacity(minuend.len());
    let mut minuend_iter = minuend.iter();
    let mut subtrahend_iter = subtrahend.iter();
    let mut subtrahend_item_option = subtrahend_iter.next();

    while let Some(minuend_item) = minuend_iter.next() {

        loop {

            match subtrahend_item_option {

                Some(subtrahend_item) =>
                    if *minuend_item < *subtrahend_item {
                        answer.push(*minuend_item);
                        break;
                    } else if *minuend_item == *subtrahend_item {
                        subtrahend_item_option = subtrahend_iter.next();
                        break;
                    } else {
                        subtrahend_item_option = subtrahend_iter.next();
                    },

                None => {
                    answer.push(*minuend_item);
                    break;
                },

            }

        }

    }

    answer

}



#[test]
fn disconnected_graph() {

    let og_graph = Graph::<(), (), Undirected, usize>::from_edges([
        (0, 1),
        (2, 3),
    ]);
    let og_graph = MyGraph(og_graph);

    let mut max_independent_set_instance =
        dendroverse
            ::DendroverseInstance
            ::<MaxIndependentSetMemo, _>
            ::with_auto_generated_nice_dtd(&og_graph, &og_graph)
            .unwrap();

    max_independent_set_instance.solve_using_nice_dtd(15).unwrap();
    let answer = max_independent_set_instance.answer().unwrap();
    assert_eq!(answer.len(), 2);
    println!("Optimal solution (concurrent): {:?}", answer);

    max_independent_set_instance.reset();

    max_independent_set_instance.solve_using_nice_dtd(1).unwrap();
    let answer = max_independent_set_instance.answer().unwrap();
    assert_eq!(answer.len(), 2);
    println!("Optimal solution (sequential): {:?}", answer);

}

#[test]
fn wikipedia_graph() {

    let og_graph = Graph::<(), (), Undirected, usize>::from_edges([
        (0, 2),
        (0, 3),
        (1, 2),
        (1, 3),
        (2, 4),
        (2, 5),
        (2, 8),
        (3, 4),
        (3, 7),
        (4, 6),
        (5, 8),
        (6, 8),
        (6, 9),
        (7, 8),
        (8, 9),
    ]);
    let og_graph = MyGraph(og_graph);

    let mut max_independent_set_instance =
        dendroverse
            ::DendroverseInstance
            ::<MaxIndependentSetMemo, _>
            ::with_auto_generated_nice_dtd(&og_graph, &og_graph)
            .unwrap();

    max_independent_set_instance.solve_using_nice_dtd(15).unwrap();
    let answer = max_independent_set_instance.answer().unwrap();
    assert_eq!(answer.len(), 6);
    println!("Optimal solution (concurrent): {:?}", answer);

    max_independent_set_instance.reset();

    max_independent_set_instance.solve_using_nice_dtd(1).unwrap();
    let answer = max_independent_set_instance.answer().unwrap();
    assert_eq!(answer.len(), 6);
    println!("Optimal solution (sequential): {:?}", answer);

}
