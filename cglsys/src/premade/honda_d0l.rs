use crate::{tree3d::*, Context, D2LSystem};
use cglsys_macro::production_rules;

pub struct HondaD0L {
    pub internode_scale_factor_1: f32,
    pub internode_scale_factor_2: f32,
    pub branch_yaw_1: f32,
    pub branch_yaw_2: f32,
    pub branch_pitch_1: f32,
    pub branch_pitch_2: f32,
    pub branch_differential_diameter_factor: f32,
    pub branch_diameter_conservation_factor: f32,

    pub root_diameter: f32,
    pub root_length: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HondaD0LAlphabet {
    F(f32),
    W(f32),
    R(f32, f32, f32),
    A(f32, f32),
    Push,
    Pop,
}

impl D2LSystem for HondaD0L {
    type Alphabet = HondaD0LAlphabet;
    type State = Tree3DState;
    type Node = Tree3DNode;

    production_rules! {
        A(s, w) =>
            W(w) F(s)
            Push
                R(0.0, self.branch_pitch_1, self.branch_yaw_1)
                A(s * self.internode_scale_factor_1, w * self.branch_differential_diameter_factor.powf(self.branch_diameter_conservation_factor))
            Pop
            Push
                R(0.0, self.branch_pitch_2, self.branch_yaw_2)
                A(s * self.internode_scale_factor_2, w * self.branch_differential_diameter_factor.powf(self.branch_diameter_conservation_factor))
            Pop,
    }

    fn axiom(&self) -> Vec<Self::Alphabet> {
        vec![Self::Alphabet::A(self.root_length, self.root_diameter)]
    }

    fn process(&self, context: &mut Context<Self::Node, Self::State>, atom: &Self::Alphabet) {
        use cgmath::Deg;

        match *atom {
            Self::Alphabet::F(s) => context.forward(s),
            Self::Alphabet::W(w) => context.diameter(w),
            Self::Alphabet::R(x, y, z) => {
                context.rotate(Deg(x).into(), Deg(y).into(), Deg(z).into())
            }
            Self::Alphabet::Push => context.push(),
            Self::Alphabet::Pop => context.pop(),
            _ => (),
        }
    }
}
