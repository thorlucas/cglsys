use crate::{lsys::*, tree::*};
use cgmath::{prelude::*, Basis3, Matrix3, Point3, Vector3};

pub struct Tree3DNode {
    position: Point3<f32>,
    diameter: f32,
}

#[derive(Copy, Clone)]
pub struct Tree3DState {
    last_node: NodeHandle,
    next_diameter: f32,
    heading: Basis3<f32>,
}

impl Context<Tree3DNode, Tree3DState> {
    /// Adds a new node a distance in the current heading from the previous node.
    pub fn forward<F>(&mut self, length: f32) {
        let delta: Vector3<f32> = {
            let change_basis: Matrix3<f32> = self.state.heading.into();
            length * change_basis * Vector3::unit_y()
        };

        let start = self.tree.get(self.state.last_node).position;
        let end = start + delta;

        let new_node = Tree3DNode {
            position: end,
            diameter: self.state.next_diameter,
        };

        self.state.last_node = self.tree.add_node(new_node);
    }

    /// Sets the diameter which will be used by the proceeding nodes.
    pub fn diameter(&mut self, diameter: f32) {
        self.state.next_diameter = diameter;
    }

    /// Rotates the heading by the euler angles x, y, and z in radians.
    pub fn rotate<A>(&mut self, x: cgmath::Rad<f32>, y: cgmath::Rad<f32>, z: cgmath::Rad<f32>) {
        let euler = cgmath::Euler::new(x, y, z);
        let rot = cgmath::Basis3::from_quaternion(&cgmath::Quaternion::from(euler));
        self.state.heading = self.state.heading * rot;
    }
}

/// Constructs a `Tree<Tree3DNode>` from a `D2LSystem<A, Tree3DNode, Tree3DState>` with alphabet A.
pub fn construct_tree_3d<A, L>(
    lsys: L,
    root_node: Tree3DNode,
    iterations: usize,
) -> Tree<Tree3DNode>
where
    L: D2LSystem<A, Tree3DNode, Tree3DState>,
    A: std::fmt::Debug,
{
    let diameter = root_node.diameter;
    construct_tree(lsys, root_node, iterations, |handle| Tree3DState {
        last_node: handle,
        next_diameter: diameter,
        heading: Basis3::<f32>::one(),
    })
}
