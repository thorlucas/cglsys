pub struct Node<T> {
    children: Vec<NodeHandle>,
    data: T,
}

impl<'a, T> Node<T> {
    pub fn new(data: T) -> Self {
        Node {
            children: vec![],
            data,
        }
    }

    fn add_child(&mut self, child: NodeHandle) {
        self.children.push(child);
    }
}

#[derive(Debug, Clone, Copy)]
pub struct NodeHandle {
    node_id: usize,
}

impl NodeHandle {
    pub fn new(node_id: usize) -> Self {
        Self { node_id }
    }
}

pub struct Edge<'a, T> {
    pub start: &'a T,
    pub end: &'a T,
}

pub struct Tree<T> {
    nodes: Vec<Node<T>>,
}

impl<T> Default for Tree<T> {
    fn default() -> Tree<T> {
        Self { nodes: vec![] }
    }
}

impl<'a, T> Tree<T> {
    pub fn add_node(&mut self, data: T) -> NodeHandle {
        self.nodes.push(Node::new(data));
        NodeHandle::new(self.nodes.len() - 1)
    }

    pub fn add_edge(&mut self, start: NodeHandle, end: NodeHandle) {
        self.nodes
            .get_mut(start.node_id)
            .expect("Invalid handle")
            .add_child(end.clone());
    }

    pub fn get(&self, node: NodeHandle) -> &T {
        &self.nodes.get(node.node_id).expect("Invalid handle").data
    }

    pub fn edges(&'a self) -> impl Iterator<Item = Edge<'a, T>> {
        self.nodes.iter().flat_map(move |node| {
            node.children.iter().map(move |child_handle| Edge {
                start: &node.data,
                end: &self
                    .nodes
                    .get(child_handle.node_id)
                    .expect("Invalid handle")
                    .data,
            })
        })
    }
}
