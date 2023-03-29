use std::{rc::Rc, cell::RefCell};

use bitvec::vec::BitVec;
use bitvec::prelude::*;

use crate::IndexedElement;

use super::{Shifter, index_shifter::IndexShifter};

#[derive(PartialEq)]
pub struct StatefulHyperGraphNode<T: PartialEq + std::fmt::Debug> {
    pub state: Rc<T>,
    neighbor_stateful_hyper_graph_nodes_per_hyper_graph_node_index: Vec<Vec<Rc<RefCell<StatefulHyperGraphNode<T>>>>>,
    is_hyper_graph_node_index_connected: BitVec
}

impl<T: PartialEq + std::fmt::Debug> StatefulHyperGraphNode<T> {
    pub fn new(state: Rc<T>, is_hyper_graph_node_index_connected: BitVec) -> Self {
        // passing in is_hyper_graph_node_index_connected sets which hyper graph nodes are already considered connected to
        //      very useful for when this hyper graph node will never be compared to another hyper graph node
        StatefulHyperGraphNode {
            state: state,
            neighbor_stateful_hyper_graph_nodes_per_hyper_graph_node_index: Vec::new(),
            is_hyper_graph_node_index_connected: is_hyper_graph_node_index_connected
        }
    }

    pub fn add_neighbor(&mut self, hyper_graph_node_index: usize, stateful_hyper_graph_node: Rc<RefCell<StatefulHyperGraphNode<T>>>) {
        while self.neighbor_stateful_hyper_graph_nodes_per_hyper_graph_node_index.len() <= hyper_graph_node_index {
            self.neighbor_stateful_hyper_graph_nodes_per_hyper_graph_node_index.push(Vec::new());
        }
        self.neighbor_stateful_hyper_graph_nodes_per_hyper_graph_node_index[hyper_graph_node_index].push(stateful_hyper_graph_node);
        self.is_hyper_graph_node_index_connected.set(hyper_graph_node_index, true);
    }

    pub fn is_connected_to_all_hyper_graph_nodes(&self) -> bool {
        debug!("state {:?} has {} connections", self.state, self.is_hyper_graph_node_index_connected.count_ones());
        return self.is_hyper_graph_node_index_connected.all();
    }
}

pub struct HyperGraphClicheShifter<T: PartialEq + std::fmt::Debug> {
    stateful_hyper_graph_nodes_per_hyper_graph_node_index: Vec<Vec<Rc<RefCell<StatefulHyperGraphNode<T>>>>>,
    is_independent_hyper_graph_node_per_hyper_graph_node_index: Rc<Vec<BitVec>>,
    hyper_graph_nodes_length: usize,
    current_hyper_graph_node_index: Option<usize>,
    current_hyper_graph_node_index_mapping: Vec<usize>,
    current_stateful_hyper_graph_node_per_hyper_graph_node_index: Vec<Rc<RefCell<StatefulHyperGraphNode<T>>>>,
    current_stateful_hyper_graph_node_index_per_hyper_graph_node_index: Vec<Option<usize>>,
    possible_states: Vec<Rc<T>>,
    focused_stateful_hyper_graph_node_index_and_hyper_graph_node_index_tuples: Option<Vec<(usize, usize)>>
}

impl<T: PartialEq + std::fmt::Debug> HyperGraphClicheShifter<T> {
    pub fn new(stateful_hyper_graph_nodes_per_hyper_graph_node_index: Vec<Vec<Rc<RefCell<StatefulHyperGraphNode<T>>>>>) -> Self {
        let mut is_independent_hyper_graph_node_per_hyper_graph_node_index: Vec<BitVec> = Vec::new();
        for _ in 0..stateful_hyper_graph_nodes_per_hyper_graph_node_index.len() {
            let is_independent_hyper_graph_node: BitVec = BitVec::repeat(false, stateful_hyper_graph_nodes_per_hyper_graph_node_index.len());
            is_independent_hyper_graph_node_per_hyper_graph_node_index.push(is_independent_hyper_graph_node);
        }
        return Self::new_with_islands(stateful_hyper_graph_nodes_per_hyper_graph_node_index, Rc::new(is_independent_hyper_graph_node_per_hyper_graph_node_index));
    }
    pub fn new_with_islands(stateful_hyper_graph_nodes_per_hyper_graph_node_index: Vec<Vec<Rc<RefCell<StatefulHyperGraphNode<T>>>>>, is_independent_hyper_graph_node_per_hyper_graph_node_index: Rc<Vec<BitVec>>) -> Self {
        let hyper_graph_nodes_length = stateful_hyper_graph_nodes_per_hyper_graph_node_index.len();
        let mut possible_states: Vec<Rc<T>> = Vec::new();
        for hyper_graph_node_index in 0..stateful_hyper_graph_nodes_per_hyper_graph_node_index.len() {
            for wrapped_stateful_hyper_graph_node in stateful_hyper_graph_nodes_per_hyper_graph_node_index[hyper_graph_node_index].iter() {
                let borrowed_stateful_hyper_graph_node = wrapped_stateful_hyper_graph_node.borrow();
                let mut is_state_in_possible_states = false;
                for possible_state in possible_states.iter() {
                    if possible_state == &borrowed_stateful_hyper_graph_node.state {
                        is_state_in_possible_states = true;
                        break;
                    }
                }
                if !is_state_in_possible_states {
                    let possible_state = borrowed_stateful_hyper_graph_node.state.clone();
                    possible_states.push(possible_state);
                }
            }
        }
        let mut current_hyper_graph_node_index_mapping: Vec<usize> = Vec::new();
        for hyper_graph_node_index in 0..stateful_hyper_graph_nodes_per_hyper_graph_node_index.len() {
            current_hyper_graph_node_index_mapping.push(hyper_graph_node_index);
        }
        HyperGraphClicheShifter {
            stateful_hyper_graph_nodes_per_hyper_graph_node_index: stateful_hyper_graph_nodes_per_hyper_graph_node_index,
            is_independent_hyper_graph_node_per_hyper_graph_node_index: is_independent_hyper_graph_node_per_hyper_graph_node_index,
            hyper_graph_nodes_length: hyper_graph_nodes_length,
            current_hyper_graph_node_index: None,
            current_hyper_graph_node_index_mapping: current_hyper_graph_node_index_mapping,
            current_stateful_hyper_graph_node_per_hyper_graph_node_index: Vec::new(),
            current_stateful_hyper_graph_node_index_per_hyper_graph_node_index: Vec::new(),
            possible_states: possible_states,
            focused_stateful_hyper_graph_node_index_and_hyper_graph_node_index_tuples: None
        }
    }

    pub fn focus_on_neighbors(&mut self, focused_stateful_hyper_graph_node_index_and_hyper_graph_node_index_tuples: Vec<(usize, usize)>) {

        // setup the current_hyper_graph_node_index_mapping to go over the same hyper graph node indexes as was provided for focusing
        self.current_hyper_graph_node_index_mapping.clear();
        for (_, hyper_graph_node_index) in focused_stateful_hyper_graph_node_index_and_hyper_graph_node_index_tuples.iter() {
            self.current_hyper_graph_node_index_mapping.push(*hyper_graph_node_index);
        }
        self.focused_stateful_hyper_graph_node_index_and_hyper_graph_node_index_tuples = Some(focused_stateful_hyper_graph_node_index_and_hyper_graph_node_index_tuples);
        for hyper_graph_node_index in 0..self.hyper_graph_nodes_length {
            if !self.current_hyper_graph_node_index_mapping.contains(&hyper_graph_node_index) {
                // TODO optimize by storing in a separate vector first, then concat together
                self.current_hyper_graph_node_index_mapping.push(hyper_graph_node_index);
            }
        }
    }

    pub fn unfocus_neighbors(&mut self) {
        self.focused_stateful_hyper_graph_node_index_and_hyper_graph_node_index_tuples = None;
        
        // reset the current_hyper_graph_node_index_mapping
        self.current_hyper_graph_node_index_mapping.clear();
        for hyper_graph_node_index in 0..self.hyper_graph_nodes_length {
            self.current_hyper_graph_node_index_mapping.push(hyper_graph_node_index);
        }
    }
}

impl<T: PartialEq + std::fmt::Debug> Shifter for HyperGraphClicheShifter<T> {
    type T = T;

    fn try_forward(&mut self) -> bool {
        if self.hyper_graph_nodes_length == 0 {
            return false;
        }
        if let Some(current_hyper_graph_node_index) = self.current_hyper_graph_node_index {
            if current_hyper_graph_node_index == self.hyper_graph_nodes_length {
                return false;
            }
            let next_hyper_graph_node_index = current_hyper_graph_node_index + 1;
            self.current_hyper_graph_node_index = Some(next_hyper_graph_node_index);
            if next_hyper_graph_node_index == self.hyper_graph_nodes_length {
                return false;
            }
            self.current_stateful_hyper_graph_node_index_per_hyper_graph_node_index.push(None);
            return true;
        }
        else {
            self.current_hyper_graph_node_index = Some(0);
            self.current_stateful_hyper_graph_node_index_per_hyper_graph_node_index.push(None);
            return true;
        }
    }
    fn try_backward(&mut self) -> bool {
        if let Some(current_hyper_graph_node_index) = self.current_hyper_graph_node_index {
            if current_hyper_graph_node_index != self.hyper_graph_nodes_length {
                self.current_stateful_hyper_graph_node_index_per_hyper_graph_node_index.pop();
                self.current_stateful_hyper_graph_node_per_hyper_graph_node_index.pop();
            }
            if current_hyper_graph_node_index == 0 {
                self.current_hyper_graph_node_index = None;
                return false;
            }
            self.current_hyper_graph_node_index = Some(current_hyper_graph_node_index - 1);
            return true;
        }
        else {
            return false;
        }
    }
    fn try_increment(&mut self) -> bool {
        // search the previous stateful_hyper_graph_node's neighbors for the next neighbor that is also a neighbor of all each previous stateful_hyper_graph_node
        if let Some(current_hyper_graph_node_index) = self.current_hyper_graph_node_index {
            let mapped_current_hyper_graph_node_index = self.current_hyper_graph_node_index_mapping[current_hyper_graph_node_index];
            
            // check to see if there are nodes to focus on first
            if let Some(focused_stateful_hyper_graph_node_index_and_hyper_graph_node_index_tuples) = &self.focused_stateful_hyper_graph_node_index_and_hyper_graph_node_index_tuples {

                // check to see that the current index is within the focused set of nodes
                if current_hyper_graph_node_index < focused_stateful_hyper_graph_node_index_and_hyper_graph_node_index_tuples.len() {

                    // if we've already incremented to the focused node, then return false
                    if let Some(current_stateful_hyper_graph_node_index) = self.current_stateful_hyper_graph_node_index_per_hyper_graph_node_index[current_hyper_graph_node_index] {
                        // this shift is focused on only this stateful hyper graph node and therefore has nowhere else to increment to
                        return false;
                    }

                    let (stateful_hyper_graph_node_index, hyper_graph_node_index) = &focused_stateful_hyper_graph_node_index_and_hyper_graph_node_index_tuples[current_hyper_graph_node_index];
                    let stateful_hyper_graph_node = self.stateful_hyper_graph_nodes_per_hyper_graph_node_index[*hyper_graph_node_index][*stateful_hyper_graph_node_index].clone();
                    self.current_stateful_hyper_graph_node_per_hyper_graph_node_index.push(stateful_hyper_graph_node);
                    self.current_stateful_hyper_graph_node_index_per_hyper_graph_node_index[current_hyper_graph_node_index] = Some(*stateful_hyper_graph_node_index);
                    return true;
                }
            }

            let initial_stateful_hyper_graph_node_index;
            if let Some(current_stateful_hyper_graph_node_index) = self.current_stateful_hyper_graph_node_index_per_hyper_graph_node_index[current_hyper_graph_node_index] {
                initial_stateful_hyper_graph_node_index = current_stateful_hyper_graph_node_index + 1;
            }
            else {
                initial_stateful_hyper_graph_node_index = 0;
            }

            // find a stateful hyper graph node in this hyper graph that is fully connected and is connected to all other previously traversed stateful hyper graph nodes
            for current_stateful_hyper_graph_node_index in initial_stateful_hyper_graph_node_index..self.stateful_hyper_graph_nodes_per_hyper_graph_node_index[mapped_current_hyper_graph_node_index].len() {
                let wrapped_current_stateful_hyper_graph_node = &self.stateful_hyper_graph_nodes_per_hyper_graph_node_index[mapped_current_hyper_graph_node_index][current_stateful_hyper_graph_node_index];
                let mut is_current_stateful_hyper_graph_node_valid = false;
                let borrowed_current_stateful_hyper_graph_node: &StatefulHyperGraphNode<T> = &wrapped_current_stateful_hyper_graph_node.borrow();
                if borrowed_current_stateful_hyper_graph_node.is_connected_to_all_hyper_graph_nodes() {
                    // TODO check to see that the previous stateful_hyper_graph_nodes permit the state of this hyper_graph_node
                    is_current_stateful_hyper_graph_node_valid = true;

                    let mut previous_hyper_graph_node_index = current_hyper_graph_node_index;
                    while previous_hyper_graph_node_index != 0 {
                        previous_hyper_graph_node_index -= 1;
                        let mapped_previous_hyper_graph_node_index = self.current_hyper_graph_node_index_mapping[previous_hyper_graph_node_index];
                        // if the previous hyper graph node is not independent from all other nodes, check to see that the current stateful hyper graph node state is connected to the previously traversed hyper graph node state
                        if !self.is_independent_hyper_graph_node_per_hyper_graph_node_index[mapped_previous_hyper_graph_node_index][mapped_current_hyper_graph_node_index] {
                            // loop over the stateful_hyper_graph_node neighbors, searching for the current stateful_hyper_graph_node state
                            let mut previous_neighbor_stateful_hyper_graph_node_exists_with_same_state = false;
                            let borrowed_previous_hyper_graph_node: &StatefulHyperGraphNode<T> = &self.current_stateful_hyper_graph_node_per_hyper_graph_node_index[previous_hyper_graph_node_index].borrow();
                            let borrowed_previous_hyper_graph_node_neighbors_length = borrowed_previous_hyper_graph_node.neighbor_stateful_hyper_graph_nodes_per_hyper_graph_node_index.len();
                            if mapped_current_hyper_graph_node_index < borrowed_previous_hyper_graph_node_neighbors_length {
                                for wrapped_previous_neighbor_stateful_hyper_graph_node in borrowed_previous_hyper_graph_node.neighbor_stateful_hyper_graph_nodes_per_hyper_graph_node_index[mapped_current_hyper_graph_node_index].iter() {
                                    let borrowed_previous_neighbor_stateful_hyper_graph_node: &StatefulHyperGraphNode<T> = &wrapped_previous_neighbor_stateful_hyper_graph_node.borrow();
                                    if borrowed_previous_neighbor_stateful_hyper_graph_node.state == borrowed_current_stateful_hyper_graph_node.state {
                                        previous_neighbor_stateful_hyper_graph_node_exists_with_same_state = true;
                                        break;
                                    }
                                }
                            }
                            if !previous_neighbor_stateful_hyper_graph_node_exists_with_same_state {
                                is_current_stateful_hyper_graph_node_valid = false;
                                break;
                            }
                        }
                    }
                }
                if is_current_stateful_hyper_graph_node_valid {
                    if current_hyper_graph_node_index == self.current_stateful_hyper_graph_node_per_hyper_graph_node_index.len() {
                        self.current_stateful_hyper_graph_node_per_hyper_graph_node_index.push(wrapped_current_stateful_hyper_graph_node.clone());
                    }
                    else {
                        self.current_stateful_hyper_graph_node_per_hyper_graph_node_index[current_hyper_graph_node_index] = wrapped_current_stateful_hyper_graph_node.clone();
                    }
                    self.current_stateful_hyper_graph_node_index_per_hyper_graph_node_index[current_hyper_graph_node_index] = Some(current_stateful_hyper_graph_node_index);
                    return true;
                }
            }
            return false;
        }
        else {
            // not moved forward yet
            return false;
        }
    }
    fn get_indexed_element(&self) -> IndexedElement<Self::T> {
        if let Some(current_hyper_graph_node_index) = self.current_hyper_graph_node_index {
            let mapped_current_hyper_graph_node_index = self.current_hyper_graph_node_index_mapping[current_hyper_graph_node_index];
            let borrowed_stateful_hyper_graph_node = self.current_stateful_hyper_graph_node_per_hyper_graph_node_index[current_hyper_graph_node_index].borrow();
            return IndexedElement::new(borrowed_stateful_hyper_graph_node.state.clone(), mapped_current_hyper_graph_node_index);
        }
        panic!("Unexpected attempt to get indexed element without moving forward and incrementing.");
    }
    fn get_length(&self) -> usize {
        return self.hyper_graph_nodes_length;
    }
    fn get_element_index_and_state_index(&self) -> (usize, usize) {
        if let Some(current_hyper_graph_node_index) = self.current_hyper_graph_node_index {
            let mapped_current_hyper_graph_node_index = self.current_hyper_graph_node_index_mapping[current_hyper_graph_node_index];
            let borrowed_stateful_hyper_graph_node = self.current_stateful_hyper_graph_node_per_hyper_graph_node_index[current_hyper_graph_node_index].borrow();
            let state = &borrowed_stateful_hyper_graph_node.state;
            for (possible_state_index, possible_state) in self.possible_states.iter().enumerate() {
                if possible_state == state {
                    return (mapped_current_hyper_graph_node_index, possible_state_index);
                }
            }
            panic!("Failed to find possible state when all possible states were previously collected.");
        }
        panic!("Unexpected attempt to get element index and state index when not moved forward and incremented.");
    }
    fn get_states(&self) -> Vec<Rc<Self::T>> {
        return self.possible_states.clone();
    }
    fn randomize(&mut self) {
        todo!();
    }
}

#[cfg(test)]
mod hyper_graph_cliche_shifter_tests {
    use std::{time::{Duration, Instant}, cell::RefCell, collections::BTreeMap};

    use super::*;
    use rstest::rstest;

    fn init() {
        std::env::set_var("RUST_LOG", "trace");
        //pretty_env_logger::try_init();
    }

    #[rstest]
    fn zero_hyper_graph_nodes() {
        init();

        let stateful_hyper_graph_nodes_per_hyper_graph_node_index: Vec<Vec<Rc<RefCell<StatefulHyperGraphNode<(u8, u8)>>>>> = Vec::new();

        let mut shifter = HyperGraphClicheShifter::new(stateful_hyper_graph_nodes_per_hyper_graph_node_index);
        for _ in 0..10 {
            assert!(!shifter.try_forward());
            assert!(!shifter.try_backward());
        }
    }

    #[rstest]
    fn one_hyper_graph_node_with_one_state() {
        init();

        let stateful_hyper_graph_nodes_per_hyper_graph_node_index: Vec<Vec<Rc<RefCell<StatefulHyperGraphNode<(u8, u8)>>>>> = vec![
            vec![
                Rc::new(RefCell::new(StatefulHyperGraphNode::new(Rc::new((10 as u8, 100 as u8)), bitvec![1])))
            ]
        ];

        let mut shifter: HyperGraphClicheShifter<(u8, u8)> = HyperGraphClicheShifter::new(stateful_hyper_graph_nodes_per_hyper_graph_node_index);
        for _ in 0..10 {
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(0, indexed_element.index);
                assert_eq!(&(10 as u8, 100 as u8), indexed_element.element.as_ref());
            }
            assert!(!shifter.try_forward());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(!shifter.try_backward());
        }
    }

    #[rstest]
    fn one_hyper_graph_node_with_two_states() {
        init();

        let stateful_hyper_graph_nodes_per_hyper_graph_node_index: Vec<Vec<Rc<RefCell<StatefulHyperGraphNode<(u8, u8)>>>>> = vec![
            vec![
                Rc::new(RefCell::new(StatefulHyperGraphNode::new(Rc::new((10 as u8, 100 as u8)), bitvec![1]))),
                Rc::new(RefCell::new(StatefulHyperGraphNode::new(Rc::new((12 as u8, 100 as u8)), bitvec![1])))
            ]
        ];

        let mut shifter: HyperGraphClicheShifter<(u8, u8)> = HyperGraphClicheShifter::new(stateful_hyper_graph_nodes_per_hyper_graph_node_index);
        for _ in 0..10 {
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(0, indexed_element.index);
                assert_eq!(&(10 as u8, 100 as u8), indexed_element.element.as_ref());
            }
            assert!(!shifter.try_forward());
            assert!(shifter.try_backward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(0, indexed_element.index);
                assert_eq!(&(12 as u8, 100 as u8), indexed_element.element.as_ref());
            }
            assert!(!shifter.try_forward());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(!shifter.try_backward());
        }
    }

    #[rstest]
    fn two_hyper_graph_nodes_with_one_state_not_neighbors() {
        init();

        let stateful_hyper_graph_nodes_per_hyper_graph_node_index: Vec<Vec<Rc<RefCell<StatefulHyperGraphNode<(u8, u8)>>>>> = vec![
            vec![
                Rc::new(RefCell::new(StatefulHyperGraphNode::new(Rc::new((10 as u8, 100 as u8)), bitvec![1, 0])))
            ],
            vec![
                Rc::new(RefCell::new(StatefulHyperGraphNode::new(Rc::new((20 as u8, 200 as u8)), bitvec![0, 1])))
            ]
        ];

        let mut shifter: HyperGraphClicheShifter<(u8, u8)> = HyperGraphClicheShifter::new(stateful_hyper_graph_nodes_per_hyper_graph_node_index);
        for _ in 0..10 {
            assert!(shifter.try_forward());
            assert!(!shifter.try_increment());
            assert!(!shifter.try_backward());
        }
    }

    #[rstest]
    fn two_hyper_graph_nodes_with_two_states_not_neighbors() {
        init();

        let stateful_hyper_graph_nodes_per_hyper_graph_node_index: Vec<Vec<Rc<RefCell<StatefulHyperGraphNode<(u8, u8)>>>>> = vec![
            vec![
                Rc::new(RefCell::new(StatefulHyperGraphNode::new(Rc::new((10 as u8, 100 as u8)), bitvec![1, 0]))),
                Rc::new(RefCell::new(StatefulHyperGraphNode::new(Rc::new((12 as u8, 100 as u8)), bitvec![1, 0])))
            ],
            vec![
                Rc::new(RefCell::new(StatefulHyperGraphNode::new(Rc::new((20 as u8, 200 as u8)), bitvec![0, 1])))
            ]
        ];

        let mut shifter: HyperGraphClicheShifter<(u8, u8)> = HyperGraphClicheShifter::new(stateful_hyper_graph_nodes_per_hyper_graph_node_index);
        for _ in 0..10 {
            assert!(shifter.try_forward());
            assert!(!shifter.try_increment());
            assert!(!shifter.try_backward());
        }
    }

    #[rstest]
    fn two_hyper_graph_nodes_with_one_state_both_neighbors() {
        init();

        let stateful_hyper_graph_nodes_per_hyper_graph_node_index: Vec<Vec<Rc<RefCell<StatefulHyperGraphNode<(u8, u8)>>>>> = vec![
            vec![
                Rc::new(RefCell::new(StatefulHyperGraphNode::new(Rc::new((10 as u8, 100 as u8)), bitvec![1, 0])))
            ],
            vec![
                Rc::new(RefCell::new(StatefulHyperGraphNode::new(Rc::new((20 as u8, 200 as u8)), bitvec![0, 1])))
            ]
        ];

        let neighbor_hyper_graph_node_index_and_hyper_graph_node_state_tuples: Vec<((usize, usize), (usize, usize))> = vec![
            ((0, 0), (1, 0))
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

        let mut shifter: HyperGraphClicheShifter<(u8, u8)> = HyperGraphClicheShifter::new(stateful_hyper_graph_nodes_per_hyper_graph_node_index);
        for _ in 0..10 {
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(0, indexed_element.index);
                assert_eq!(&(10 as u8, 100 as u8), indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(1, indexed_element.index);
                assert_eq!(&(20 as u8, 200 as u8), indexed_element.element.as_ref());
            }
            assert!(!shifter.try_forward());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(!shifter.try_backward());
        }
    }

    #[rstest]
    fn two_hyper_graph_nodes_with_two_separate_cliche_states_both_neighbors() {
        init();

        let stateful_hyper_graph_nodes_per_hyper_graph_node_index: Vec<Vec<Rc<RefCell<StatefulHyperGraphNode<(u8, u8)>>>>> = vec![
            vec![
                Rc::new(RefCell::new(StatefulHyperGraphNode::new(Rc::new((10 as u8, 100 as u8)), bitvec![1, 0]))),
                Rc::new(RefCell::new(StatefulHyperGraphNode::new(Rc::new((12 as u8, 100 as u8)), bitvec![1, 0])))
            ],
            vec![
                Rc::new(RefCell::new(StatefulHyperGraphNode::new(Rc::new((20 as u8, 200 as u8)), bitvec![0, 1]))),
                Rc::new(RefCell::new(StatefulHyperGraphNode::new(Rc::new((20 as u8, 202 as u8)), bitvec![0, 1])))
            ]
        ];

        let neighbor_hyper_graph_node_index_and_hyper_graph_node_state_tuples: Vec<((usize, usize), (usize, usize))> = vec![
            ((0, 0), (1, 0)),
            ((0, 1), (1, 1))
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

        let mut shifter: HyperGraphClicheShifter<(u8, u8)> = HyperGraphClicheShifter::new(stateful_hyper_graph_nodes_per_hyper_graph_node_index);
        for _ in 0..10 {
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(0, indexed_element.index);
                assert_eq!(&(10 as u8, 100 as u8), indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(1, indexed_element.index);
                assert_eq!(&(20 as u8, 200 as u8), indexed_element.element.as_ref());
            }
            assert!(!shifter.try_forward());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(0, indexed_element.index);
                assert_eq!(&(12 as u8, 100 as u8), indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(1, indexed_element.index);
                assert_eq!(&(20 as u8, 202 as u8), indexed_element.element.as_ref());
            }
            assert!(!shifter.try_forward());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(!shifter.try_backward());
        }
    }

    #[rstest]
    fn three_hyper_graph_nodes_with_almost_cliche_states() {
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

        let mut shifter: HyperGraphClicheShifter<(u8, u8)> = HyperGraphClicheShifter::new(stateful_hyper_graph_nodes_per_hyper_graph_node_index);
        for _ in 0..10 {
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(0, indexed_element.index);
                assert_eq!(&(10 as u8, 100 as u8), indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(1, indexed_element.index);
                assert_eq!(&(20 as u8, 200 as u8), indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(0, indexed_element.index);
                assert_eq!(&(12 as u8, 100 as u8), indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(1, indexed_element.index);
                assert_eq!(&(20 as u8, 200 as u8), indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(!shifter.try_backward());
        }
    }

    #[rstest]
    fn three_hyper_graph_nodes_with_step_cliche_states_unfocused() {
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

        let mut shifter: HyperGraphClicheShifter<(u8, u8)> = HyperGraphClicheShifter::new(stateful_hyper_graph_nodes_per_hyper_graph_node_index);
        for _ in 0..10 {
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(0, indexed_element.index);
                assert_eq!(&(14 as u8, 100 as u8), indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(1, indexed_element.index);
                assert_eq!(&(20 as u8, 202 as u8), indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(2, indexed_element.index);
                assert_eq!(&(30 as u8, 40 as u8), indexed_element.element.as_ref());
            }
            assert!(!shifter.try_forward());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(!shifter.try_backward());
        }
    }

    #[rstest]
    fn three_hyper_graph_nodes_with_step_cliche_states_focused_on_last_hyper_graph_node_index() {
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

        let mut shifter: HyperGraphClicheShifter<(u8, u8)> = HyperGraphClicheShifter::new(stateful_hyper_graph_nodes_per_hyper_graph_node_index.clone());
        shifter.focus_on_neighbors(vec![
            (0, 2),
            (1, 1)
        ]);
        for _ in 0..10 {
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(2, indexed_element.index);
                assert_eq!(&(30 as u8, 40 as u8), indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(1, indexed_element.index);
                assert_eq!(&(20 as u8, 202 as u8), indexed_element.element.as_ref());
            }
            assert!(shifter.try_forward());
            assert!(shifter.try_increment());
            {
                let indexed_element = shifter.get_indexed_element();
                assert_eq!(0, indexed_element.index);
                assert_eq!(&(14 as u8, 100 as u8), indexed_element.element.as_ref());
            }
            assert!(!shifter.try_forward());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(shifter.try_backward());
            assert!(!shifter.try_increment());
            assert!(!shifter.try_backward());
        }
    }
}