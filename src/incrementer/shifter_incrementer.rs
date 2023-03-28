use std::{rc::Rc, cell::RefCell};
use crate::{shifter::Shifter, IndexedElement};
use super::Incrementer;
use bitvec::prelude::*;

// Purpose: with each iteration, evaluates a complete shifted state of the underlying shifter
pub struct ShifterIncrementer<T> {
    shifter: Box<dyn Shifter<T = T>>,
    index_mapping: Vec<usize>,
    is_started: bool,
    is_completed: bool,
    current_indexed_elements: Vec<IndexedElement<T>>,
    shifter_length: usize
}

impl<T> ShifterIncrementer<T> {
    pub fn new(shifter: Box<dyn Shifter<T = T>>, index_mapping: Vec<usize>) -> Self {
        let shifter_length = shifter.get_length();
        ShifterIncrementer {
            shifter: shifter,
            index_mapping: index_mapping,
            is_started: shifter_length == 0,
            is_completed: shifter_length == 0,
            current_indexed_elements: Vec::new(),
            shifter_length: shifter_length
        }
    }
}

impl<T> Incrementer for ShifterIncrementer<T> {
    type T = T;

    fn try_increment(&mut self) -> bool {
        if self.is_completed {
            return false;
        }
        if !self.is_started {
            self.is_started = true;
            let mut is_forward_required = true;
            while self.current_indexed_elements.len() != self.shifter_length {
                if is_forward_required && !self.shifter.try_forward() {
                    panic!("Unexpectedly failed to move forward when not at the end.");
                    //self.is_completed = true;
                    //return false;
                }
                if self.shifter.try_increment() {
                    let indexed_element = self.shifter.get_indexed_element();
                    self.current_indexed_elements.push(indexed_element);
                    is_forward_required = true;
                }
                else {
                    self.current_indexed_elements.pop();
                    if !self.shifter.try_backward() {
                        // failed to find any valid initial set of states
                        self.is_completed = true;
                        return false;
                    }
                    is_forward_required = false;
                }
            }
            return self.current_indexed_elements.len() != 0;
        }
        self.current_indexed_elements.pop();
        while self.current_indexed_elements.len() != self.shifter_length {
            if self.shifter.try_increment() {
                let indexed_element = self.shifter.get_indexed_element();
                self.current_indexed_elements.push(indexed_element);
                if self.current_indexed_elements.len() != self.shifter_length {
                    if !self.shifter.try_forward() {
                        panic!("Unexpectedly failed to move forward when not at the end.");
                    }
                }
            }
            else {
                if self.current_indexed_elements.len() == 0 {
                    self.is_completed = true;
                    return false;
                }
                self.current_indexed_elements.pop();
                if !self.shifter.try_backward() {
                    panic!("Unexpectedly failed to move backward when not at the beginning.");
                }
            }
        }
        return true;
    }
    fn get(&self) -> Vec<IndexedElement<Self::T>> {
        return self.current_indexed_elements
            .iter()
            .map(|indexed_element| { IndexedElement::new(indexed_element.element.clone(), self.index_mapping[indexed_element.index]) })
            .collect();
    }
    fn reset(&mut self) {
        self.shifter.reset();
        self.is_started = false;
        self.is_completed = false;
        self.current_indexed_elements.clear();

    }
    fn randomize(&mut self) {
        self.shifter.randomize();
    }
}

#[cfg(test)]
mod shifter_incrementer_tests {
    use std::{time::{Duration, Instant}, cell::RefCell, collections::BTreeSet};

    use crate::shifter::{segment_permutation_shifter::{SegmentPermutationShifter, Segment}, hyper_graph_cliche_shifter::{HyperGraphClicheShifter, StatefulHyperGraphNode}};

    use super::*;
    use bitvec::{bits, vec::BitVec};
    use rstest::rstest;
    use uuid::Uuid;

    fn init() {
        std::env::set_var("RUST_LOG", "trace");
        //pretty_env_logger::try_init();
    }

    #[rstest]
    fn two_segment_permutation_shifters() {
        init();

        let mut shifter_incrementer = ShifterIncrementer::new(
            Box::new(SegmentPermutationShifter::new(
                vec![
                    Rc::new(Segment::new(1)),
                    Rc::new(Segment::new(1))
                ],
                (10, 100),
                4,
                true,
                1,
                false
            )),
            vec![0, 1]
        );

        assert!(shifter_incrementer.try_increment());
        {
            let indexed_elements = shifter_incrementer.get();
            assert_eq!(2, indexed_elements.len());
            assert_eq!(0, indexed_elements[0].index);
            assert_eq!(&(10, 100), indexed_elements[0].element.as_ref());
            assert_eq!(1, indexed_elements[1].index);
            assert_eq!(&(12, 100), indexed_elements[1].element.as_ref());
        }
        assert!(shifter_incrementer.try_increment());
        {
            let indexed_elements = shifter_incrementer.get();
            assert_eq!(2, indexed_elements.len());
            assert_eq!(0, indexed_elements[0].index);
            assert_eq!(&(10, 100), indexed_elements[0].element.as_ref());
            assert_eq!(1, indexed_elements[1].index);
            assert_eq!(&(13, 100), indexed_elements[1].element.as_ref());
        }
        assert!(shifter_incrementer.try_increment());
        {
            let indexed_elements = shifter_incrementer.get();
            assert_eq!(2, indexed_elements.len());
            assert_eq!(0, indexed_elements[0].index);
            assert_eq!(&(11, 100), indexed_elements[0].element.as_ref());
            assert_eq!(1, indexed_elements[1].index);
            assert_eq!(&(13, 100), indexed_elements[1].element.as_ref());
        }
        assert!(!shifter_incrementer.try_increment());
    }

    #[rstest]
    fn complex_hyper_graph_cliche_shifter_with_zero_valid_cliches() {
        init();

        let stateful_hyper_graph_nodes_per_hyper_graph_node_index: Vec<Vec<Rc<RefCell<StatefulHyperGraphNode<(u8, u8)>>>>> = vec![
            vec![
                Rc::new(RefCell::new(StatefulHyperGraphNode::new(Rc::new((10 as u8, 100 as u8)), bitvec![1, 0, 0]))),
                Rc::new(RefCell::new(StatefulHyperGraphNode::new(Rc::new((12 as u8, 100 as u8)), bitvec![1, 0, 0])))
            ],
            vec![
                Rc::new(RefCell::new(StatefulHyperGraphNode::new(Rc::new((20 as u8, 200 as u8)), bitvec![0, 1, 0]))),
                Rc::new(RefCell::new(StatefulHyperGraphNode::new(Rc::new((20 as u8, 202 as u8)), bitvec![0, 1, 0])))
            ],
            vec![
                Rc::new(RefCell::new(StatefulHyperGraphNode::new(Rc::new((30 as u8, 40 as u8)), bitvec![0, 0, 1]))),
                Rc::new(RefCell::new(StatefulHyperGraphNode::new(Rc::new((31 as u8, 41 as u8)), bitvec![0, 0, 1])))
            ]
        ];

        let neighbor_hyper_graph_node_index_and_hyper_graph_node_state_tuples: Vec<((usize, usize), (usize, usize))> = vec![
            ((0, 0), (1, 0)),
            ((0, 0), (2, 0)),
            ((2, 0), (1, 1)),
            ((2, 0), (0, 1)),
            ((1, 1), (2, 1)),
            ((0, 1), (1, 0)),
            ((1, 0), (2, 1))
        ];

        for ((from_hyper_graph_node_index, from_hyper_graph_node_state_index), (to_hyper_graph_node_index, to_hyper_graph_node_state_index)) in neighbor_hyper_graph_node_index_and_hyper_graph_node_state_tuples {
            {
                let to_stateful_hyper_graph_node = stateful_hyper_graph_nodes_per_hyper_graph_node_index[to_hyper_graph_node_index][to_hyper_graph_node_state_index].clone();
                stateful_hyper_graph_nodes_per_hyper_graph_node_index[from_hyper_graph_node_index][from_hyper_graph_node_state_index].borrow_mut().add_neighbor(to_hyper_graph_node_index, to_stateful_hyper_graph_node);
            }
            {
                let from_stateful_hyper_graph_node = stateful_hyper_graph_nodes_per_hyper_graph_node_index[from_hyper_graph_node_index][from_hyper_graph_node_state_index].clone();
                stateful_hyper_graph_nodes_per_hyper_graph_node_index[to_hyper_graph_node_index][to_hyper_graph_node_state_index].borrow_mut().add_neighbor(from_hyper_graph_node_index, from_stateful_hyper_graph_node);
            }
        }

        let shifter: HyperGraphClicheShifter<(u8, u8)> = HyperGraphClicheShifter::new(stateful_hyper_graph_nodes_per_hyper_graph_node_index);
        let mut incrementer = ShifterIncrementer::new(Box::new(shifter), vec![0, 1, 2]);
        assert!(!incrementer.try_increment());
    }

    #[rstest]
    fn complex_hyper_graph_cliche_shifter_with_one_valid_cliche() {
        init();

        let stateful_hyper_graph_nodes_per_hyper_graph_node_index: Vec<Vec<Rc<RefCell<StatefulHyperGraphNode<(u8, u8)>>>>> = vec![
            vec![
                Rc::new(RefCell::new(StatefulHyperGraphNode::new(Rc::new((10 as u8, 100 as u8)), bitvec![1, 0, 0]))),
                Rc::new(RefCell::new(StatefulHyperGraphNode::new(Rc::new((12 as u8, 100 as u8)), bitvec![1, 0, 0]))),
                Rc::new(RefCell::new(StatefulHyperGraphNode::new(Rc::new((14 as u8, 100 as u8)), bitvec![1, 0, 0])))
            ],
            vec![
                Rc::new(RefCell::new(StatefulHyperGraphNode::new(Rc::new((20 as u8, 200 as u8)), bitvec![0, 1, 0]))),
                Rc::new(RefCell::new(StatefulHyperGraphNode::new(Rc::new((20 as u8, 202 as u8)), bitvec![0, 1, 0])))
            ],
            vec![
                Rc::new(RefCell::new(StatefulHyperGraphNode::new(Rc::new((30 as u8, 40 as u8)), bitvec![0, 0, 1])))
            ]
        ];

        let neighbor_hyper_graph_node_index_and_hyper_graph_node_state_tuples: Vec<((usize, usize), (usize, usize))> = vec![
            ((0, 1), (1, 0)),
            ((0, 2), (1, 1)),
            ((1, 1), (2, 0)),
            ((0, 2), (2, 0))
        ];

        for ((from_hyper_graph_node_index, from_hyper_graph_node_state_index), (to_hyper_graph_node_index, to_hyper_graph_node_state_index)) in neighbor_hyper_graph_node_index_and_hyper_graph_node_state_tuples {
            {
                let to_stateful_hyper_graph_node = stateful_hyper_graph_nodes_per_hyper_graph_node_index[to_hyper_graph_node_index][to_hyper_graph_node_state_index].clone();
                stateful_hyper_graph_nodes_per_hyper_graph_node_index[from_hyper_graph_node_index][from_hyper_graph_node_state_index].borrow_mut().add_neighbor(to_hyper_graph_node_index, to_stateful_hyper_graph_node);
            }
            {
                let from_stateful_hyper_graph_node = stateful_hyper_graph_nodes_per_hyper_graph_node_index[from_hyper_graph_node_index][from_hyper_graph_node_state_index].clone();
                stateful_hyper_graph_nodes_per_hyper_graph_node_index[to_hyper_graph_node_index][to_hyper_graph_node_state_index].borrow_mut().add_neighbor(from_hyper_graph_node_index, from_stateful_hyper_graph_node);
            }
        }

        let shifter: HyperGraphClicheShifter<(u8, u8)> = HyperGraphClicheShifter::new(stateful_hyper_graph_nodes_per_hyper_graph_node_index);
        let mut incrementer = ShifterIncrementer::new(Box::new(shifter), vec![0, 1, 2]);
        assert!(incrementer.try_increment());
        {
            let indexed_elements = incrementer.get();
            assert_eq!(3, indexed_elements.len());
            assert_eq!(0, indexed_elements[0].index);
            assert_eq!(&(14 as u8, 100 as u8), indexed_elements[0].element.as_ref());
            assert_eq!(1, indexed_elements[1].index);
            assert_eq!(&(20 as u8, 202 as u8), indexed_elements[1].element.as_ref());
            assert_eq!(2, indexed_elements[2].index);
            assert_eq!(&(30 as u8, 40 as u8), indexed_elements[2].element.as_ref());
        }
        assert!(!incrementer.try_increment());
    }
}