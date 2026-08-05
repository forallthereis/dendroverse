mod dtd;
mod solver;





pub struct DendroverseInstance<'a, MemoType, AdditionalDataType>
where
    MemoType: Default + Send + Sync,
    AdditionalDataType: Sync,
{
    dtds: Vec<dtd::DirectedTreeDecomposition<MemoType>>,
    solution_algo: solver::SolutionAlgorithm<MemoType, AdditionalDataType>,
    additional_data: &'a AdditionalDataType,
    threads_count: usize,
}

impl<'a, MemoType, AdditionalDataType> DendroverseInstance<'a, MemoType, AdditionalDataType>
where
    MemoType: Default + Send + Sync,
    AdditionalDataType: Sync,
{
    pub fn solve(&mut self) -> anyhow::Result<()> {
        match &self.solution_algo {
            solver::SolutionAlgorithm::UsingNiceDTD(payloads) =>
                solver::solve_using_nice_dtd(&mut self.dtds, payloads, self.additional_data, self.threads_count),
        }
    }
}
