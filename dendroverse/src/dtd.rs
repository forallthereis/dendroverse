use std::sync::{Arc, Mutex};





pub(super) struct DirectedTreeDecomposition<MemoType> {
    pub root_nid: usize,
    pub adj_list: Vec<DTDNodeConnectivity>,
    pub nodes: Vec<Arc<Mutex<DTDNode<MemoType>>>>,
}

impl<'a, MemoType> DirectedTreeDecomposition<MemoType> {
    #[inline(always)]
    pub(super) fn iter_leaves(&'a self) -> DTDLeafIter<'a, MemoType> {
        DTDLeafIter { dtd: self, nid: 0 }
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
