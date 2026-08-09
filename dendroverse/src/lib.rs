use std::sync::MutexGuard;

mod dtd;
mod solver;





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

    pub fn answer(&self) -> Option<MemoType::AnswerType>
    where
        MemoType: AnswerableMemo,
    {

        if self.is_instance_solved {

            let root_nodes: Vec<MutexGuard<dtd::DTDNode<MemoType>>> =
                self
                .dtds
                .iter()
                .map(|dtd| unsafe { dtd.nodes.get_unchecked(dtd.root_nid).lock().unwrap() })
                .collect();

            let root_nodes_memos: Vec<&MemoType> =
                root_nodes
                .iter()
                .map(|root_node| &root_node.memo)
                .collect();

            Some(MemoType::answer(root_nodes_memos))

        } else {

            None

        }

    }

    pub fn solve_using_nice_dtd(&mut self) -> anyhow::Result<()>
    where
        MemoType: NiceDTDMemo<AdditionalDataType>,
    {
        solver::solve_using_nice_dtd(&mut self.dtds, self.additional_data, self.threads_count)
    }

}



pub trait NiceDTDMemo<AdditionalDataType> {

    fn leaf_payload(
        &mut self,
        bag: &Vec<usize>,
        additional_data: &AdditionalDataType,
    );

    fn introduce_payload(
        &mut self,
        child_bag: &Vec<usize>,
        child_memo: &Self,
        introduced_vids: &Vec<usize>,
        additional_data: &AdditionalDataType,
    );

    fn forget_payload(
        &mut self,
        child_bag: &Vec<usize>,
        child_memo: &Self,
        forgotten_vids: &Vec<usize>,
        additional_data: &AdditionalDataType,
    );

    fn join_payload(
        &mut self,
        children_bag: &Vec<usize>,
        child1_memo: &Self,
        child2_memo: &Self,
        additional_data: &AdditionalDataType,
    );

}



pub trait AnswerableMemo {

    type AnswerType;

    fn answer(root_nodes_memos: Vec<&Self>) -> Self::AnswerType;

}
