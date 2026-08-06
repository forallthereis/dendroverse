use std::{collections::VecDeque, sync::{Arc, Condvar, Mutex, mpsc}, thread};





pub(super) enum SolutionAlgorithm<MemoType, AdditionalDataType> {
    UsingNiceDTD(NiceDTDPayloads<MemoType, AdditionalDataType>),
}



pub(super) struct NiceDTDPayloads<MemoType, AdditionalDataType> {
    leaf_payload: fn(&mut MemoType, &Vec<usize>, &AdditionalDataType),
    introduce_payload: fn(&mut MemoType, &Vec<usize>, &MemoType, &Vec<usize>, &AdditionalDataType),
    forget_payload: fn(&mut MemoType, &Vec<usize>, &MemoType, &Vec<usize>, &AdditionalDataType),
    join_payload: fn(&mut MemoType, &Vec<usize>, &MemoType, &MemoType, &AdditionalDataType),
}



enum NiceDTDJob<MemoType> {
    Leaf(NiceTDTLeafJobInfo<MemoType>),
    IntroduceForget(NiceDTDIntroduceForgetJobInfo<MemoType>),
    Join(NiceDTDJoinJobInfo<MemoType>),
    Terminate,
}



struct NiceTDTLeafJobInfo<MemoType> {
    fullnid: (usize, usize),
    node: Arc<Mutex<crate::dtd::DTDNode<MemoType>>>,
}



struct NiceDTDIntroduceForgetJobInfo<MemoType> {
    fullnid: (usize, usize),
    node: Arc<Mutex<crate::dtd::DTDNode<MemoType>>>,
    child_node: Arc<Mutex<crate::dtd::DTDNode<MemoType>>>,
    forgotten_vids: Vec<usize>,
    introduced_vids: Vec<usize>,
}



struct NiceDTDJoinJobInfo<MemoType> {
    fullnid: (usize, usize),
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
        .enumerate()
        .flat_map(|(dtdid, dtd)| dtd.iter_leaves().map(move |item| (dtdid, item)))
        .map(|(dtdid, (nid, node))| NiceDTDJob::Leaf(NiceTDTLeafJobInfo { fullnid: (dtdid, nid), node }))
        .collect();
    let available_jobs = Arc::new(
        AvailableJobs {
            jobs_queue: Mutex::new(available_jobs_queue_init),
            not_empty_anymore: Condvar::new(),
        }
    );

    // Create communication channels for the reports about completed jobs
    let (completed_jobs_tx, completed_jobs_rx) = mpsc::sync_channel::<(usize, usize)>(threads_count);
    let mut completed_dtds = 0usize;

    // Spawn worker threads
    thread::scope(|s| {
        for _ in 0..threads_count {
            s.spawn(|| { nice_dtd_worker_thread(Arc::clone(&available_jobs), completed_jobs_tx.clone(), payloads, additional_data) });
        }
    });

    // Track the reports about completed jobs
    loop {

        let (dtdid, nid) = completed_jobs_rx.recv()?;

        if nid == unsafe { dtds.get_unchecked(dtdid).root_nid } {
            completed_dtds += 1;
            if completed_dtds == dtds.len() {
                break;
            }
            continue;
        }

        let parent_nid = unsafe { dtds.get_unchecked(dtdid).adj_list.get_unchecked(nid).parent_nid.unwrap() };
        let parent_children_nids = unsafe { &dtds.get_unchecked(dtdid).adj_list.get_unchecked(parent_nid).children_nids };

        available_jobs.jobs_queue.lock().unwrap().push_back(

            if parent_children_nids.len() == 1 {

                let node = unsafe { Arc::clone( &dtds.get_unchecked(dtdid.clone()).nodes.get_unchecked(parent_nid.clone()) ) };
                let child_node = unsafe { Arc::clone( &dtds.get_unchecked(dtdid.clone()).nodes.get_unchecked(nid) ) };

                let forgotten_vids: Vec<usize>;
                let introduced_vids: Vec<usize>;

                {
                    let node_bag = &node.lock().unwrap().bag;
                    let child_bag = &child_node.lock().unwrap().bag;

                    forgotten_vids = node_bag.iter().filter(|vid| !child_bag.contains(vid)).cloned().collect();
                    introduced_vids = child_bag.iter().filter(|vid| !node_bag.contains(vid)).cloned().collect();
                }

                NiceDTDJob::IntroduceForget(
                    NiceDTDIntroduceForgetJobInfo {
                        fullnid: (dtdid, parent_nid),
                        node,
                        child_node,
                        forgotten_vids,
                        introduced_vids,
                    }
                )

            } else {

                let node = unsafe { Arc::clone( &dtds.get_unchecked(dtdid.clone()).nodes.get_unchecked(parent_nid.clone()) ) };
                let child_node1 = unsafe { Arc::clone( &dtds.get_unchecked(dtdid.clone()).nodes.get_unchecked(parent_children_nids.get_unchecked(0).clone()) ) };
                let child_node2 = unsafe { Arc::clone( &dtds.get_unchecked(dtdid.clone()).nodes.get_unchecked(parent_children_nids.get_unchecked(1).clone()) ) };

                NiceDTDJob::Join(
                    NiceDTDJoinJobInfo {
                        fullnid: (dtdid, parent_nid),
                        node,
                        child_node1,
                        child_node2,
                    }
                )

            }

        );

        available_jobs.not_empty_anymore.notify_one();

    }

    // Kill the worker threads
    for _ in 0..threads_count {
        available_jobs.jobs_queue.lock().unwrap().push_back(NiceDTDJob::Terminate);

        available_jobs.not_empty_anymore.notify_one();
    }

    Ok(())
}



fn nice_dtd_worker_thread<MemoType, AdditionalDataType>(
    available_jobs: Arc<AvailableJobs<NiceDTDJob<MemoType>>>,
    completed_jobs_tx: mpsc::SyncSender<(usize, usize)>,
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
                let crate::dtd::DTDNode::<MemoType> { bag: node_bag, memo: node_memo, .. } = &mut *node;

                (payloads.leaf_payload)(node_memo, node_bag, additional_data);

                completed_jobs_tx.send(job_info.fullnid).unwrap();
            },

            NiceDTDJob::IntroduceForget(job_info) => {
                let mut node = job_info.node.lock().unwrap();
                let child_node = job_info.child_node.lock().unwrap();
                let mut intermediate_memo = MemoType::default();

                (payloads.forget_payload)(
                    &mut intermediate_memo,
                    &child_node.bag,
                    &child_node.memo,
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
                    &mut node.memo,
                    &intermediate_bag,
                    &intermediate_memo,
                    &job_info.introduced_vids,
                    additional_data,
                );

                completed_jobs_tx.send(job_info.fullnid).unwrap();
            },

            NiceDTDJob::Join(job_info) => {
                let mut node = job_info.node.lock().unwrap();
                let child_node1 = job_info.child_node1.lock().unwrap();
                let child_node2 = job_info.child_node2.lock().unwrap();

                (payloads.join_payload)(
                    &mut node.memo,
                    &child_node1.bag,
                    &child_node1.memo,
                    &child_node2.memo,
                    additional_data,
                );

                completed_jobs_tx.send(job_info.fullnid).unwrap();
            },

            NiceDTDJob::Terminate => break,

        }
    }
}
