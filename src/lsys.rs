use crate::tree::*;
use cgmath::{prelude::*, Basis3, Matrix3, Point3, Vector3};

/// Context passed to the lsys which is used to draw add segments.
pub struct Context<T, S>
where
    S: Copy,
{
    // TODO: Maybe tree should be private? Add functions for interacting with it.
    pub tree: Tree<T>,
    pub state: S,
    state_stack: Vec<S>,
}

impl<T, S> Context<T, S>
where
    S: Copy,
{
    pub fn push(&mut self) {
        self.state_stack.push(self.state);
    }

    // TODO: Result?
    pub fn pop(&mut self) {
        self.state = self.state_stack.pop().expect("No state on stack");
    }
}

/// A determinstic L-system with both left and right contexts.
pub trait D2LSystem {
    type Alphabet;
    type Node;
    type State: Copy;

    fn axiom(&self) -> Vec<Self::Alphabet>;
    fn production_rules(
        &self,
        atom: &Self::Alphabet,
        left_context: &[Self::Alphabet],
        right_context: &[Self::Alphabet],
    ) -> Vec<Self::Alphabet>;
    fn process(&self, context: &mut Context<Self::Node, Self::State>, atom: &Self::Alphabet);
}

pub fn evolve<A, T, S, L>(lsys: &L, iterations: usize) -> Vec<A>
where
    L: D2LSystem<Alphabet = A, Node = T, State = S>,
    S: Copy,
    A: std::fmt::Debug,
{
    let mut cons = lsys.axiom();
    println!("Evolving: {:?}", cons);
    for i in 0..iterations {
        let mut res = vec![];
        println!("Iter: {}", i);
        println!("Cons: {:?}", cons);
        for a in 0..(cons.len()) {
            let left_context = &cons[..a];
            let atom = &cons[a];
            let right_context = &cons[(a + 1)..];

            println!("A: {}", a);
            println!("LC: {:?}", left_context);
            println!("Atom: {:?}", atom);
            println!("RC: {:?}", right_context);

            res.extend(lsys.production_rules(atom, left_context, right_context));
        }
        cons = res;
    }

    cons
}

pub fn construct_tree<A, T, S, L, F>(
    lsys: L,
    root_node: T,
    iterations: usize,
    initialize_state: F,
) -> Tree<T>
where
    L: D2LSystem<Alphabet = A, Node = T, State = S>,
    S: Copy,
    F: Fn(NodeHandle) -> S,
    A: std::fmt::Debug,
{
    let mut context = {
        let mut tree: Tree<T> = Tree::<T>::default();
        let root_node_handle = tree.add_node(root_node);
        let state = initialize_state(root_node_handle);

        Context {
            tree,
            state_stack: vec![],
            state,
        }
    };

    let cons = evolve(&lsys, iterations);

    cons.iter().for_each(|atom| {
        lsys.process(&mut context, atom);
    });

    return context.tree;
}
