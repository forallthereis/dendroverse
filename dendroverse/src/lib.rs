use std::collections::VecDeque;

use arboretum_td::{graph::MutableGraph, solver::AtomSolver};

mod dtd;
mod solve;





pub struct DendroverseInstance<'a, MemoType, AdditionalDataType>
where
    MemoType: Default + Send + Sync,
    AdditionalDataType: Sync,
{
    dtds: Vec<dtd::DirectedTreeDecomposition<MemoType>>,
    additional_data: &'a AdditionalDataType,
    threads_count: usize,
    is_instance_solved: bool,
}

impl<'a, MemoType, AdditionalDataType> DendroverseInstance<'a, MemoType, AdditionalDataType>
where
    MemoType: Default + Send + Sync,
    AdditionalDataType: Sync,
{

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

    pub fn with_auto_generated_nice_dtd<OgGraphType>(
        og_graph: &'a OgGraphType,
        additional_data: &'a AdditionalDataType,
        threads_count: usize,
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
        for cc_vids in og_graph_arboretum.connected_components() {
            let cc = og_graph_arboretum.vertex_induced_subgraph(&cc_vids);
            let cc_td_generator = arboretum_td::exact::TamakiPid::with_graph(&cc);
            dtds.push(
                match cc_td_generator.compute() {
                    arboretum_td::solver::ComputationResult::Bounds(_) => return Err(anyhow::anyhow!("The automatic generation of the nice tree decomposition failed.")),
                    arboretum_td::solver::ComputationResult::ComputedTreeDecomposition(td) => dtd::DirectedTreeDecomposition::nice_dtd_from(td),
                }
            );
        }

        Ok(DendroverseInstance { dtds, additional_data, threads_count, is_instance_solved: false })

    }

    #[inline(always)]
    pub fn solve_using_nice_dtd(&mut self) -> anyhow::Result<()>
    where
        MemoType: NiceDTDMemo<AdditionalDataType>,
    {
        let result = solve::solve_using_nice_dtd(&mut self.dtds, self.additional_data, self.threads_count);
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
