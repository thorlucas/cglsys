mod lsys;
mod tree;
pub mod tree3d;

pub use lsys::*;

#[cfg(test)]
mod tests {
    use super::tree::*;
    use super::*;

    #[derive(Debug, PartialEq, Clone, Copy)]
    struct SimpleNode(usize);

    #[derive(Debug, Clone, Copy)]
    struct SimpleState(NodeHandle, usize);

    #[derive(Debug, Clone, Copy, PartialEq)]
    enum SimpleAlphabet {
        A(usize),
        B(usize, usize),
        F,
    }

    struct SimpleLSystem {
        axiom: Vec<SimpleAlphabet>,
    }

    impl SimpleLSystem {
        pub fn new(axiom: Vec<SimpleAlphabet>) -> Self {
            SimpleLSystem { axiom }
        }
    }

    impl D2LSystem<SimpleAlphabet, SimpleNode, SimpleState> for SimpleLSystem {
        fn axiom(&self) -> Vec<SimpleAlphabet> {
            self.axiom.clone()
        }

        fn rules(
            &self,
            atom: &SimpleAlphabet,
            left_context: &[SimpleAlphabet],
            right_context: &[SimpleAlphabet],
        ) -> Vec<SimpleAlphabet> {
            match (left_context, atom, right_context) {
                (
                    &[.., SimpleAlphabet::A(a)],
                    SimpleAlphabet::A(x),
                    &[SimpleAlphabet::B(b, c)],
                    ..,
                ) if a + b + c < 10 => vec![
                    SimpleAlphabet::B(a + b, a + c),
                    SimpleAlphabet::A(x + a + b + c),
                ],
                (&[..], SimpleAlphabet::B(x, y), &[..]) => vec![SimpleAlphabet::A(x + y)],
                (&[..], a, &[..]) => vec![*a],
            }
        }

        fn process(&self, context: &mut Context<SimpleNode, SimpleState>, atom: &SimpleAlphabet) {
            match *atom {
                SimpleAlphabet::A(x) => context.state.1 += x,
                SimpleAlphabet::F => {
                    let h = context.tree.add_node(SimpleNode(context.state.1));
                    context.tree.add_edge(context.state.0, h);
                }
                _ => (),
            }
        }
    }

    #[test]
    fn tree_can_add_node() {
        let mut tree = Tree::<SimpleNode>::default();
        let data = SimpleNode(10);

        let handle = tree.add_node(data);
        assert_eq!(*tree.get(handle), data);
    }

    #[test]
    fn tree_can_add_edge() {
        let mut tree = Tree::<SimpleNode>::default();
        let a = SimpleNode(10);
        let b = SimpleNode(20);

        let ha = tree.add_node(a);
        let hb = tree.add_node(b);

        tree.add_edge(ha, hb);

        let e = tree.edges().next().expect("No edges");
        assert_eq!(*e.start, a);
        assert_eq!(*e.end, b);
    }

    #[test]
    fn can_evolve_lsystem() {
        let test_case = move |iterations, axiom, expected| {
            assert_eq!(evolve(&SimpleLSystem::new(axiom), iterations), expected);
        };

        test_case(0, vec![SimpleAlphabet::F], vec![SimpleAlphabet::F]);

        test_case(1, vec![], vec![]);

        test_case(1, vec![SimpleAlphabet::A(5)], vec![SimpleAlphabet::A(5)]);

        test_case(1, vec![SimpleAlphabet::B(3, 4)], vec![SimpleAlphabet::A(7)]);

        test_case(
            1,
            vec![
                SimpleAlphabet::A(1),
                SimpleAlphabet::A(2),
                SimpleAlphabet::B(3, 4),
            ],
            vec![
                SimpleAlphabet::A(1),
                SimpleAlphabet::B(4, 5),
                SimpleAlphabet::A(10),
                SimpleAlphabet::A(7),
            ],
        );

        test_case(
            2,
            vec![
                SimpleAlphabet::A(1),
                SimpleAlphabet::A(2),
                SimpleAlphabet::B(3, 4),
            ],
            vec![
                SimpleAlphabet::A(1),
                SimpleAlphabet::A(9),
                SimpleAlphabet::A(10),
                SimpleAlphabet::A(7),
            ],
        );
    }

    #[test]
    fn can_construct_tree() {
        let lsys = SimpleLSystem::new(vec![SimpleAlphabet::A(5), SimpleAlphabet::F]);
        let res = construct_tree(lsys, SimpleNode(0), 0, |handle| SimpleState(handle, 0));

        let e = res.edges().next().unwrap();
        assert_eq!(*e.start, SimpleNode(0));
        assert_eq!(*e.end, SimpleNode(5));
    }
}
