use std::{collections::VecDeque, sync::{Arc, Condvar, Mutex, mpsc}, thread};

use fxhash::{FxBuildHasher, FxHashSet};





enum NiceDTDJob<MemoType> {
    Leaf(NiceTDTLeafJobInfo<MemoType>),
    ForgetIntroduce(NiceDTDIntroduceForgetJobInfo<MemoType>),
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



pub(super) fn solve_using_nice_dtd_multithread<MemoType, AdditionalDataType>(
    dtds: &mut Vec<crate::dtd::DirectedTreeDecomposition<MemoType>>,
    additional_data: &AdditionalDataType,
    threads_count: usize,
) -> anyhow::Result<()>
where
    MemoType: Send + Sync + crate::NiceDTDMemo<AdditionalDataType>,
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

    // Create a tracker of join nodes at least one child of which was processed
    // Join nodes can only be processed once both their children are processed
    let mut one_child_processed_join_nids = FxHashSet::with_hasher(FxBuildHasher::new());

    // Create communication channels for the reports about completed jobs
    let (completed_jobs_tx, completed_jobs_rx) = mpsc::sync_channel::<(usize, usize)>(threads_count);
    let mut completed_dtds = 0usize;

    // Spawn worker threads
    thread::scope(|s| -> anyhow::Result<()> {

        for _ in 0..threads_count {
            s.spawn(|| { nice_dtd_worker_thread(Arc::clone(&available_jobs), completed_jobs_tx.clone(), additional_data) });
        }

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

            if parent_children_nids.len() == 2 && !one_child_processed_join_nids.contains(&parent_nid) {
                one_child_processed_join_nids.insert(parent_nid);
                continue;
            }

            available_jobs
                .jobs_queue
                .lock()
                .map_err(|_| anyhow::anyhow!("Orchestrator failed to send new jobs to the worker threads because one of the worker threads had panicked and deadlocked the shared jobs queue."))?
                .push_back(

                if parent_children_nids.len() == 1 {

                    let node = unsafe { Arc::clone( &dtds.get_unchecked(dtdid.clone()).nodes.get_unchecked(parent_nid.clone()) ) };
                    let child_node = unsafe { Arc::clone( &dtds.get_unchecked(dtdid.clone()).nodes.get_unchecked(nid) ) };

                    let forgotten_vids: Vec<usize>;
                    let introduced_vids: Vec<usize>;

                    {
                        const ERROR_MESSAGE: &str = "Orchestrator failed to read the bag of a nice tree decomposition node because one of the worker threads had panicked and deadlocked the node.";
                        let node_bag = &node.lock().map_err(|_| anyhow::anyhow!(ERROR_MESSAGE))?.bag;
                        let child_bag = &child_node.lock().map_err(|_| anyhow::anyhow!(ERROR_MESSAGE))?.bag;

                        forgotten_vids = child_bag.iter().filter(|vid| !node_bag.contains(vid)).cloned().collect();
                        introduced_vids = node_bag.iter().filter(|vid| !child_bag.contains(vid)).cloned().collect();
                    }

                    NiceDTDJob::ForgetIntroduce(
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
            available_jobs
                .jobs_queue
                .lock()
                .map_err(|_| anyhow::anyhow!("Orchestrator failed to terminate the worker threads because one of the worker threads had panicked and deadlocked the shared jobs queue."))?
                .push_back(NiceDTDJob::Terminate);

            available_jobs.not_empty_anymore.notify_one();
        }

        Ok(())

    })

}



fn nice_dtd_worker_thread<MemoType, AdditionalDataType>(
    available_jobs: Arc<AvailableJobs<NiceDTDJob<MemoType>>>,
    completed_jobs_tx: mpsc::SyncSender<(usize, usize)>,
    additional_data: &AdditionalDataType,
)
where
    MemoType: crate::NiceDTDMemo<AdditionalDataType>,
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

                node_memo.leaf_payload(node_bag, additional_data);

                completed_jobs_tx.send(job_info.fullnid).unwrap();
            },

            NiceDTDJob::ForgetIntroduce(job_info) => {
                let mut node = job_info.node.lock().unwrap();
                let child_node = job_info.child_node.lock().unwrap();

                node.memo.forget_introduce_payload(
                    &child_node.bag,
                    job_info.fullnid.1,
                    &child_node.memo,
                    &job_info.forgotten_vids,
                    &job_info.introduced_vids,
                    additional_data,
                );

                completed_jobs_tx.send(job_info.fullnid).unwrap();
            },

            NiceDTDJob::Join(job_info) => {
                let mut node = job_info.node.lock().unwrap();
                let child_node1 = job_info.child_node1.lock().unwrap();
                let child_node2 = job_info.child_node2.lock().unwrap();

                node.memo.join_payload(
                    &child_node1.bag,
                    job_info.fullnid.1,
                    &child_node1.memo,
                    job_info.fullnid.1,
                    &child_node2.memo,
                    additional_data
                );

                completed_jobs_tx.send(job_info.fullnid).unwrap();
            },

            NiceDTDJob::Terminate => break,

        }

    }

}



pub(super) fn solve_using_nice_dtd_singlethread<MemoType, AdditionalDataType>(
    dtds: &mut Vec<crate::dtd::DirectedTreeDecomposition<MemoType>>,
    additional_data: &AdditionalDataType,
) -> anyhow::Result<()>
where
    MemoType: crate::NiceDTDMemo<AdditionalDataType>,
{

    for dtd in dtds {

        let mut node_queue = VecDeque::from_iter(dtd.iter_leaves());
        let mut one_child_processed_join_nids = FxHashSet::with_hasher(FxBuildHasher::new());

        while !node_queue.is_empty() {

            let (nid, node) = node_queue.pop_front().unwrap();
            let mut node = node.lock().unwrap();
            let children_nids = & unsafe { dtd.adj_list.get_unchecked(nid) }.children_nids;
            let parent_nid_option = unsafe { dtd.adj_list.get_unchecked(nid) }.parent_nid;
            let siblings_count =
                match parent_nid_option {
                    Some(parent_nid) => unsafe { dtd.adj_list.get_unchecked(parent_nid) }.children_nids.len(),
                    None => 0,
                };

            // Process the node, call payloads
            match children_nids.len() {

                0 => {

                    let crate::dtd::DTDNode::<MemoType> { bag: node_bag, memo: node_memo, .. } = &mut *node;

                    node_memo.leaf_payload(node_bag, additional_data);

                },

                1 => {

                    let child_nid = unsafe { *children_nids.get_unchecked(0) };
                    let child_node = unsafe { dtd.nodes.get_unchecked(child_nid) }.lock().unwrap();

                    let forgotten_vids = child_node.bag.iter().filter(|vid| !node.bag.contains(vid)).cloned().collect();
                    let introduced_vids = node.bag.iter().filter(|vid| !child_node.bag.contains(vid)).cloned().collect();

                    node.memo.forget_introduce_payload(
                        &child_node.bag,
                        child_nid,
                        &child_node.memo,
                        &forgotten_vids,
                        &introduced_vids,
                        additional_data,
                    );

                },

                2 => {

                    let child1_nid = unsafe { *children_nids.get_unchecked(0) };
                    let child1_node = unsafe { dtd.nodes.get_unchecked(child1_nid) }.lock().unwrap();
                    let child2_nid = unsafe { *children_nids.get_unchecked(1) };
                    let child2_node = unsafe { dtd.nodes.get_unchecked(child2_nid) }.lock().unwrap();

                    node.memo.join_payload(
                        &child1_node.bag,
                        child1_nid,
                        &child1_node.memo,
                        child2_nid,
                        &child2_node.memo,
                        additional_data
                    );

                },

                _ => return Err(anyhow::anyhow!("A node with more than two children was encountered in a nice tree decomposition.")),

            }

            if let Some(parent_nid) = parent_nid_option {

                if siblings_count <= 1 || one_child_processed_join_nids.contains(&parent_nid) {
                    node_queue.push_back((parent_nid, Arc::clone( unsafe { dtd.nodes.get_unchecked(parent_nid) } )));
                } else {
                    one_child_processed_join_nids.insert(parent_nid);
                }

            }

        }

    }

    Ok(())

}
