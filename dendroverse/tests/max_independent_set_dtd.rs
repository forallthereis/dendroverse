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

#[test]
#[cfg(feature = "integrate-petgraph")]
fn graph_1357() {

    let og_graph: MyGraph = Graph::from_edges([
        (0, 12),
        (0, 24),
        (0, 51),
        (1, 13),
        (1, 25),
        (1, 51),
        (2, 22),
        (2, 34),
        (2, 54),
        (3, 23),
        (3, 35),
        (3, 54),
        (4, 14),
        (4, 28),
        (4, 55),
        (5, 15),
        (5, 29),
        (5, 55),
        (6, 16),
        (6, 30),
        (6, 56),
        (7, 17),
        (7, 31),
        (7, 56),
        (8, 18),
        (8, 26),
        (8, 53),
        (9, 19),
        (9, 27),
        (9, 53),
        (10, 20),
        (10, 32),
        (10, 52),
        (11, 21),
        (11, 33),
        (11, 52),
        (12, 0),
        (12, 25),
        (12, 45),
        (13, 1),
        (13, 24),
        (13, 45),
        (14, 4),
        (14, 29),
        (14, 47),
        (15, 5),
        (15, 28),
        (15, 47),
        (16, 6),
        (16, 31),
        (16, 50),
        (17, 7),
        (17, 30),
        (17, 50),
        (18, 8),
        (18, 27),
        (18, 46),
        (19, 9),
        (19, 26),
        (19, 46),
        (20, 10),
        (20, 33),
        (20, 49),
        (21, 11),
        (21, 32),
        (21, 49),
        (22, 2),
        (22, 35),
        (22, 48),
        (23, 3),
        (23, 34),
        (23, 48),
        (24, 0),
        (24, 13),
        (24, 42),
        (25, 1),
        (25, 12),
        (25, 42),
        (26, 8),
        (26, 19),
        (26, 42),
        (27, 9),
        (27, 18),
        (27, 42),
        (28, 4),
        (28, 15),
        (28, 43),
        (29, 5),
        (29, 14),
        (29, 43),
        (30, 6),
        (30, 17),
        (30, 44),
        (31, 7),
        (31, 16),
        (31, 44),
        (32, 10),
        (32, 21),
        (32, 44),
        (33, 11),
        (33, 20),
        (33, 44),
        (34, 2),
        (34, 23),
        (34, 43),
        (35, 3),
        (35, 22),
        (35, 43),
        (36, 37),
        (36, 45),
        (36, 58),
        (37, 36),
        (37, 46),
        (37, 57),
        (38, 39),
        (38, 47),
        (38, 57),
        (39, 38),
        (39, 48),
        (39, 59),
        (40, 41),
        (40, 50),
        (40, 59),
        (41, 40),
        (41, 49),
        (41, 58),
        (42, 24),
        (42, 25),
        (42, 26),
        (42, 27),
        (43, 28),
        (43, 29),
        (43, 34),
        (43, 35),
        (44, 30),
        (44, 31),
        (44, 32),
        (44, 33),
        (45, 12),
        (45, 13),
        (45, 36),
        (45, 46),
        (46, 18),
        (46, 19),
        (46, 37),
        (46, 45),
        (47, 14),
        (47, 15),
        (47, 38),
        (47, 48),
        (48, 22),
        (48, 23),
        (48, 39),
        (48, 47),
        (49, 20),
        (49, 21),
        (49, 41),
        (49, 50),
        (50, 16),
        (50, 17),
        (50, 40),
        (50, 49),
        (51, 0),
        (51, 1),
        (51, 52),
        (51, 53),
        (52, 10),
        (52, 11),
        (52, 51),
        (52, 56),
        (53, 8),
        (53, 9),
        (53, 51),
        (53, 55),
        (54, 2),
        (54, 3),
        (54, 55),
        (54, 56),
        (55, 4),
        (55, 5),
        (55, 53),
        (55, 54),
        (56, 6),
        (56, 7),
        (56, 52),
        (56, 54),
        (57, 37),
        (57, 38),
        (57, 58),
        (57, 59),
        (58, 36),
        (58, 41),
        (58, 57),
        (58, 59),
        (59, 39),
        (59, 40),
        (59, 57),
        (59, 58),
    ]);

    let mut max_independent_set_instance: dendroverse::DendroverseInstance<MaxIndependentSetMemo, MyGraph> =
        dendroverse
            ::DendroverseInstance
            ::with_auto_generated_dtd(&og_graph, &og_graph)
            .unwrap();

    max_independent_set_instance.solve_using_dtd(15).unwrap();
    let answer = max_independent_set_instance.solution().unwrap();
    assert_eq!(answer, 25);
    println!("Optimal solution (concurrent): {:?}", answer);

    max_independent_set_instance.reset();

    max_independent_set_instance.solve_using_dtd(1).unwrap();
    let answer = max_independent_set_instance.solution().unwrap();
    assert_eq!(answer, 25);
    println!("Optimal solution (sequential): {:?}", answer);

}
