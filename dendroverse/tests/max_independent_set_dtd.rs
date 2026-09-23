// Run the tests using the following command:
//
//      cargo test --features integrate-petgraph -- --nocapture
//
// This test solves the problem of finding THE CARDINALITY of a maximum independent set in a graph
// using dynamic programming over arbitrary directed tree decompositions.
// Here, the original graph is given as an instance of petgraph::Graph<(), (), Undirected, usize>,
// aliased below as MyGraph.
// To use this original graph as input, we rely on the dendroverse.rs's feature flag integrate-petgraph.
// To understand the algorithm, consult with https://en.wikipedia.org/wiki/Tree_decomposition#Dynamic_programming
use std::cmp::Ordering;

use dendroverse;
use fxhash::FxHashMap;
use itertools::Itertools;
use petgraph::{Graph, Undirected};





#[allow(unused)]
type MyGraph = Graph<(), (), Undirected, usize>;



#[allow(unused)]
#[derive(Debug, Default)]
struct MaxIndependentSetMemo {
    a: FxHashMap<Vec<usize>, usize>,
    b: FxHashMap<(usize, Vec<usize>), usize>,
}

impl dendroverse::DTDMemo<MyGraph> for MaxIndependentSetMemo {

    fn payload(
        &mut self,
        bag: &Vec<usize>,
        children_bags: Vec<&Vec<usize>>,
        children_nids: Vec<usize>,
        children_memos: Vec<&Self>,
        additional_data: &MyGraph,
    )
    {

        let bag_refs = bag.iter().collect();

        // Populate B
        for child_i in 0..children_nids.len() {

            for (child_vertex_set, child_a_value) in children_memos[child_i].a.iter() {

                let vertex_set = intersect_sorted_vecs(&bag_refs, child_vertex_set);

                if let Some(exising_b_value) = self.b.get(&(children_nids[child_i], vertex_set.clone())) && *exising_b_value >= *child_a_value {}
                else {
                    self.b.insert((children_nids[child_i], vertex_set.clone()), *child_a_value);
                }

            }

        }

        // Populate A
        for vertex_set in bag.iter().powerset() {

            if is_independent(&vertex_set, additional_data) {

                self.a.insert(vertex_set.iter().map(|vid| **vid).collect(),
                    vertex_set.len()
                    +
                    (0..children_nids.len())
                        .map(
                            |child_i| {
                                let descendant_vertex_set = intersect_sorted_vecs(&vertex_set, children_bags[child_i]);
                                let descendant_vertex_set_len = descendant_vertex_set.len();
                                self.b[&(children_nids[child_i], descendant_vertex_set)] - descendant_vertex_set_len
                            }
                        ).sum::<usize>()
                );

            }

        }

    }

}

impl dendroverse::BacktrackableMemo for MaxIndependentSetMemo {

    type BacktrackingHint = ();
    type PartialSolutionType = usize;
    type SolutionType = usize;

    fn extend_partial_solution(
        &self,
        _nid: usize,
        partial_solution: Self::PartialSolutionType,
        hint: Option<Self::BacktrackingHint>,
        children_nids: &Vec<usize>
    ) -> (Option<Self::PartialSolutionType>, Vec<Option<Self::BacktrackingHint>>)
    {
        if hint.is_none() {
            let opt_value = self.a.iter().map(|(_, obj_value)| *obj_value).max().unwrap();
            (Some(partial_solution + opt_value), vec![Some(()); children_nids.len()])
        } else {
            (Some(partial_solution), vec![Some(()); children_nids.len()])
        }
    }

}





#[allow(unused)]
fn is_independent(vertex_set: &Vec<&usize>, og_graph: &MyGraph) -> bool {
    for i in 0..vertex_set.len() {
        for j in (i + 1)..vertex_set.len() {
            if og_graph.contains_edge((*vertex_set[i]).into(), (*vertex_set[j]).into()) {
                return false;
            }
        }
    }
    true
}

#[allow(unused)]
fn intersect_sorted_vecs(vec1: &Vec<&usize>, vec2: &Vec<usize>) -> Vec<usize> {

    let mut answer = Vec::with_capacity(vec1.len().max(vec2.len()));
    let mut vec1_iter = vec1.iter().map(|item| **item);
    let mut vec2_iter = vec2.iter().cloned();
    let mut vec2_item_option = vec2_iter.next();

    while let Some(vec1_item) = vec1_iter.next() {

        loop {

            if let Some(vec2_item) = vec2_item_option {

                match vec1_item.cmp(&vec2_item) {
                    Ordering::Less => break,
                    Ordering::Equal => {
                        answer.push(vec2_item);
                        vec2_item_option = vec2_iter.next();
                        break;
                    },
                    Ordering::Greater => {vec2_item_option = vec2_iter.next();},
                }

            } else {
                break;
            }

        }

    }

    answer

}





#[test]
#[cfg(feature = "integrate-petgraph")]
fn disconnected_graph() {

    let og_graph: MyGraph = Graph::from_edges([
        (0, 1),
        (2, 3),
    ]);

    let mut max_independent_set_instance: dendroverse::DendroverseInstance<MaxIndependentSetMemo, MyGraph> =
        dendroverse
            ::DendroverseInstance
            ::with_auto_generated_dtd(&og_graph, &og_graph)
            .unwrap();

    max_independent_set_instance.solve_using_dtd(15).unwrap();
    let answer = max_independent_set_instance.solution().unwrap();
    assert_eq!(answer, 2);
    println!("Optimal solution (concurrent): {:?}", answer);

    max_independent_set_instance.reset();

    max_independent_set_instance.solve_using_dtd(1).unwrap();
    let answer = max_independent_set_instance.solution().unwrap();
    assert_eq!(answer, 2);
    println!("Optimal solution (sequential): {:?}", answer);

}

#[test]
#[cfg(feature = "integrate-petgraph")]
fn wikipedia_graph() {

    let og_graph: MyGraph = Graph::from_edges([
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

    let mut max_independent_set_instance: dendroverse::DendroverseInstance<MaxIndependentSetMemo, MyGraph> =
        dendroverse
            ::DendroverseInstance
            ::with_auto_generated_dtd(&og_graph, &og_graph)
            .unwrap();

    max_independent_set_instance.solve_using_dtd(15).unwrap();
    let answer = max_independent_set_instance.solution().unwrap();
    assert_eq!(answer, 6);
    println!("Optimal solution (concurrent): {:?}", answer);

    max_independent_set_instance.reset();

    max_independent_set_instance.solve_using_dtd(1).unwrap();
    let answer = max_independent_set_instance.solution().unwrap();
    assert_eq!(answer, 6);
    println!("Optimal solution (sequential): {:?}", answer);

}
