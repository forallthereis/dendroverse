use std::{collections::VecDeque, sync::{Arc, Condvar, Mutex, mpsc}, thread};





pub(super) enum SolutionAlgorithm<MemoType, AdditionalDataType> {
    UsingNiceDTD(NiceDTDPayloads<MemoType, AdditionalDataType>),
}



pub(super) struct NiceDTDPayloads<MemoType, AdditionalDataType> {
    leaf_payload: fn(&Vec<usize>, &mut MemoType, &AdditionalDataType),
    introduce_payload: fn(&mut MemoType, &Vec<usize>, &MemoType, &Vec<usize>, &AdditionalDataType),
    forget_payload: fn(&mut MemoType, &Vec<usize>, &MemoType, &Vec<usize>, &AdditionalDataType),
    join_payload: fn(&Vec<usize>, &mut MemoType, &MemoType, &MemoType, &AdditionalDataType),
}



enum NiceDTDJob<MemoType> {
    Leaf(NiceTDTLeafJobInfo<MemoType>),
    IntroduceForget(NiceDTDIntroduceForgetJobInfo<MemoType>),
    Join(NiceDTDJoinJobInfo<MemoType>),
    Terminate,
}



struct NiceTDTLeafJobInfo<MemoType> {
    nid: usize,
    node: Arc<Mutex<crate::dtd::DTDNode<MemoType>>>,
}



struct NiceDTDIntroduceForgetJobInfo<MemoType> {
    nid: usize,
    node: Arc<Mutex<crate::dtd::DTDNode<MemoType>>>,
    child_node: Arc<Mutex<crate::dtd::DTDNode<MemoType>>>,
    forgotten_vids: Vec<usize>,
    introduced_vids: Vec<usize>,
}



struct NiceDTDJoinJobInfo<MemoType> {
    nid: usize,
    node: Arc<Mutex<crate::dtd::DTDNode<MemoType>>>,
    child_node1: Arc<Mutex<crate::dtd::DTDNode<MemoType>>>,
    child_node2: Arc<Mutex<crate::dtd::DTDNode<MemoType>>>,
}



struct AvailableJobs<JobType> {
    jobs_queue: Mutex<VecDeque<JobType>>,
    not_empty_anymore: Condvar,
}



pub(super) fn solve_using_nice_dtd<MemoType, AdditionalDataType>(
    dtds: &mut Vec<crate::dtd::DirectedTreeDecomposition<MemoType>>,
    payloads: &NiceDTDPayloads<MemoType, AdditionalDataType>,
    additional_data: &AdditionalDataType,
    threads_count: usize,
) -> anyhow::Result<()>
where
    MemoType: Default + Send + Sync,
    AdditionalDataType: Sync,
{
    // Populate the initial available jobs queue with the leaves of the tree decompositions
    let available_jobs_queue_init: VecDeque<NiceDTDJob<MemoType>> =
        dtds
        .iter()
        .flat_map(|dtd| dtd.iter_leaves())
        .map(|(nid, node)| NiceDTDJob::Leaf(NiceTDTLeafJobInfo { nid, node }))
        .collect();
    let available_jobs = Arc::new(
        AvailableJobs {
            jobs_queue: Mutex::new(available_jobs_queue_init),
            not_empty_anymore: Condvar::new(),
        }
    );

    // Create communication channels for the reports about completed jobs
    let (completed_jobs_tx, completed_jobs_rx) = mpsc::sync_channel::<usize>(threads_count);

    // Spawn worker threads
    thread::scope(|s| {
        for _ in 0..threads_count {
            s.spawn(|| { nice_dtd_worker_thread(Arc::clone(&available_jobs), completed_jobs_tx.clone(), payloads, additional_data) });
        }
    });

    // Track the reports about completed jobs
    todo!();

    Ok(())
}



fn nice_dtd_worker_thread<MemoType, AdditionalDataType>(
    available_jobs: Arc<AvailableJobs<NiceDTDJob<MemoType>>>,
    completed_jobs_tx: mpsc::SyncSender<usize>,
    payloads: &NiceDTDPayloads<MemoType, AdditionalDataType>,
    additional_data: &AdditionalDataType,
)
where
    MemoType: Default,
{
    loop {
        let mut available_jobs_queue = available_jobs.jobs_queue.lock().unwrap();
        while available_jobs_queue.is_empty() {
            available_jobs_queue = available_jobs.not_empty_anymore.wait(available_jobs_queue).unwrap();
        }

        let job = available_jobs_queue.pop_front().unwrap();

        drop(available_jobs_queue);

        match job {

            NiceDTDJob::Leaf(job_info) => {
                let mut node = job_info.node.lock().unwrap();
                let mut memo = MemoType::default();

                (payloads.leaf_payload)(&node.bag, &mut memo, additional_data);

                node.memo = Some(memo);

                completed_jobs_tx.send(job_info.nid).unwrap();
            },

            NiceDTDJob::IntroduceForget(job_info) => {
                let mut node = job_info.node.lock().unwrap();
                let child_node = job_info.child_node.lock().unwrap();
                let mut intermediate_memo = MemoType::default();

                (payloads.forget_payload)(
                    &mut intermediate_memo,
                    &child_node.bag,
                    child_node.memo.as_ref().unwrap(),
                    &job_info.forgotten_vids,
                    additional_data,
                );

                let intermediate_bag: Vec<usize> =
                    child_node
                    .bag
                    .iter()
                    .filter(|vid| !job_info.forgotten_vids.contains(vid))
                    .cloned()
                    .collect();

                (payloads.introduce_payload)(
                    node.memo.as_mut().unwrap(),
                    &intermediate_bag,
                    &intermediate_memo,
                    &job_info.introduced_vids,
                    additional_data,
                );

                completed_jobs_tx.send(job_info.nid).unwrap();
            },

            NiceDTDJob::Terminate => break,

            _ => todo!("Implement all other node types."),
        }
    }
}
