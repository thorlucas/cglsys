use cglsys::{evolve, premade::HondaD0L, tree3d::*};
use cgmath::Point3;

fn main() {
    let honda = HondaD0L {
        internode_scale_factor_1: 0.60,
        internode_scale_factor_2: 0.85,
        branch_yaw_1: 25.0,
        branch_yaw_2: -15.0,
        branch_pitch_1: 180.0,
        branch_pitch_2: 180.0,
        branch_differential_diameter_factor: 0.45,
        branch_diameter_conservation_factor: 0.50,
        root_length: 100.0,
        root_diameter: 10.0,
    };

    let tree = construct_tree_3d(
        honda,
        Tree3DNode {
            position: cgmath::Point3::<f32>::new(0.0, 0.0, 0.0),
            diameter: 10.0,
        },
        3,
    );

    for edge in tree.edges() {
        println!(
            "Edge: (pos: {:?}, dia: {}), (pos: {:?}, dia: {}) ",
            edge.start.position, edge.start.diameter, edge.end.position, edge.end.diameter
        );
    }
}
