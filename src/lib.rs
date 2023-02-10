use std::collections::HashMap;
use std::ops::{Index, IndexMut};

#[derive(Debug, Eq, PartialEq, PartialOrd, Ord, Clone, Copy, Hash)]
pub struct NodeId(usize);

#[derive(Debug, Eq, PartialEq, PartialOrd, Ord, Clone, Copy, Hash)]
pub struct EdgeId(usize);

#[derive(Default, Clone)]
pub struct Edge<EdgeUserData = ()> {
    pub user_data: EdgeUserData,
}

pub struct Graph<NodeUserData = (), EdgeUserData = ()> {
    next_id: usize,

    nodes_data: HashMap<NodeId, NodeUserData>,
    edges_data: HashMap<EdgeId, EdgeUserData>,

    edge_nodes: HashMap<EdgeId, (NodeId, NodeId)>,
}

impl<NodeUserData, EdgeUserData> Index<NodeId> for Graph<NodeUserData, EdgeUserData> {
    type Output = NodeUserData;
    fn index(&self, idx: NodeId) -> &NodeUserData {
        self.nodes_data.get(&idx).unwrap()
    }
}

impl<NodeUserData, EdgeUserData> IndexMut<NodeId> for Graph<NodeUserData, EdgeUserData> {
    fn index_mut(&mut self, idx: NodeId) -> &mut NodeUserData {
        self.nodes_data.get_mut(&idx).unwrap()
    }
}

impl<NodeUserData, EdgeUserData> Index<EdgeId> for Graph<NodeUserData, EdgeUserData> {
    type Output = EdgeUserData;
    fn index(&self, idx: EdgeId) -> &EdgeUserData {
        self.edges_data.get(&idx).unwrap()
    }
}

impl<NodeUserData, EdgeUserData> IndexMut<EdgeId> for Graph<NodeUserData, EdgeUserData> {
    fn index_mut(&mut self, idx: EdgeId) -> &mut EdgeUserData {
        self.edges_data.get_mut(&idx).unwrap()
    }
}

impl<NodeUserData, EdgeUserData> Default for Graph<NodeUserData, EdgeUserData> {
    fn default() -> Self {
        Graph {
            next_id: 0,
            nodes_data: HashMap::new(),
            edges_data: HashMap::new(),
            edge_nodes: HashMap::new(),
        }
    }
}

impl<NodeUserData, EdgeUserData> Graph<NodeUserData, EdgeUserData> {
    pub fn add_node(&mut self, node: NodeUserData) -> NodeId {
        let id = NodeId(self.next_id);
        self.next_id += 1;
        self.nodes_data.insert(id, node);
        id
    }
    pub fn remove_node(&mut self, id: NodeId) {
        let mut edges = Vec::new();
        for (e, (l, r)) in &self.edge_nodes {
            if l == &id || r == &id {
                edges.push(*e);
            }
        }
        for e in edges {
            self.edge_nodes.remove(&e);
            self.edges_data.remove(&e);
        }
        self.nodes_data.remove(&id);
    }
    pub fn add_edge(&mut self, l: NodeId, r: NodeId, edge: EdgeUserData) -> EdgeId {
        let id = EdgeId(self.next_id);
        self.next_id += 1;
        self.edges_data.insert(id, edge);
        self.edge_nodes.insert(id, (l, r));
        id
    }
    pub fn remove_edge(&mut self, id: EdgeId) {
        self.edge_nodes.remove(&id);
        self.edges_data.remove(&id);
    }
    pub fn visit_nodes<F: FnMut(NodeId, &NodeUserData)>(&self, mut f: F) {
        for (id, data) in &self.nodes_data {
            f(*id, data)
        }
    }
    pub fn visit_nodes_mut<F: FnMut(NodeId, &mut NodeUserData)>(&mut self, mut f: F) {
        for (id, data) in &mut self.nodes_data {
            f(*id, data)
        }
    }
    pub fn visit_edges<F: FnMut(EdgeId, &NodeUserData, &NodeUserData, &EdgeUserData)>(&self, mut f: F) {        
        for (id, data) in &self.edges_data {
            let (node1, node2) = self.edge_nodes.get(id).unwrap();
            let data1 = self.nodes_data.get(node1).unwrap();
            let data2 = self.nodes_data.get(node2).unwrap();
            f(*id, data1, data2, data)
        }
    }
    pub fn index_twice_mut(&mut self, id1: NodeId, id2: NodeId) -> (&mut NodeUserData, &mut NodeUserData) {
        unsafe {
            let self_mut = self as *mut _;
            (<Self as IndexMut<NodeId>>::index_mut(&mut *self_mut, id1),
             <Self as IndexMut<NodeId>>::index_mut(&mut *self_mut, id2))
        }
    }
    pub fn node_count(&self) -> usize {
        self.nodes_data.len()
    }
    pub fn neighbors_data<'a>(&'a self, id: NodeId) -> NeighborsData<'a, NodeUserData> {
        let mut neighbors = Vec::new();
        for (_, (l, r)) in &self.edge_nodes {
            if l == &id {
                neighbors.push((*l, self.nodes_data.get(l).unwrap()));
            } else if r == &id {
                neighbors.push((*r, self.nodes_data.get(r).unwrap()));
            }            
        }
        NeighborsData(neighbors)
    }
    pub fn neighbors<'a>(&'a self, id: NodeId) -> Neighbors {
        let mut neighbors = Vec::new();
        for (_, (l, r)) in &self.edge_nodes {
            if l == &id {
                neighbors.push(*l);
            } else if r == &id {
                neighbors.push(*r);
            }            
        }
        Neighbors(neighbors)
    }
}

pub struct Neighbors(Vec<NodeId>);

impl Neighbors {
    pub fn detach(self) -> NeighborsIter {
        NeighborsIter(false, 0, self.0)
    }
}

pub struct NeighborsIter(bool, usize, Vec<NodeId>);

impl NeighborsIter {
    pub fn next_node<T>(&mut self, _: T) -> Option<NodeId> {
        if self.0 {
            return None;
        }
        let idx = self.1;
        if idx == self.2.len() {
            self.0 = true;
            return None;
        }
        self.1 = 1 + idx;
        Some(self.2[idx])
    }
}

pub struct NeighborsData<'a, NodeUserData>(Vec<(NodeId, &'a NodeUserData)>);

impl<'a, NodeUserData> NeighborsData<'a, NodeUserData> {
    pub fn detach<'b>(&'b self) -> NeighborsDataIter<'a, 'b, NodeUserData> {
        NeighborsDataIter(false, 0, &self.0)
    }
}

pub struct NeighborsDataIter<'a, 'b, NodeUserData>(bool, usize, &'b Vec<(NodeId, &'a NodeUserData)>);

impl<'a, 'b, NodeUserData> NeighborsDataIter<'a, 'b, NodeUserData> {
    pub fn next_node<T>(&mut self, _: T) -> Option<NodeId> {
        if self.0 {
            return None;
        }
        let idx = self.1;
        if idx == self.2.len() {
            self.0 = true;
            return None;
        }
        self.1 = 1 + idx;
        Some(self.2[idx].0)
    }
}

impl<'a, 'b, NodeUserData> Iterator for NeighborsDataIter<'a, 'b, NodeUserData> {
    type Item = &'a NodeUserData;
    fn next(&mut self) -> Option<&'a NodeUserData> {
        if self.0 {
            return None;
        }
        let idx = self.1;
        if idx == self.2.len() {
            self.0 = true;
            return None;
        }
        self.1 = 1 + idx;
        Some(self.2[idx].1)
    }
}

