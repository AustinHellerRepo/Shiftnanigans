use std::rc::Rc;

use crate::IndexedElement;

use super::{Shifter, index_shifter::IndexShifter};

#[derive(PartialEq)]
pub struct StatefulHyperGraphNode<T: PartialEq> {
    state: Rc<T>,
    neighbor_stateful_hyper_graph_nodes_per_hyper_graph_node_index: Vec<Vec<Rc<StatefulHyperGraphNode<T>>>>
}

pub struct HyperGraphClicheShifter<T: PartialEq> {
    stateful_hyper_graph_nodes_per_hyper_graph_node_index: Vec<Vec<Rc<StatefulHyperGraphNode<T>>>>,
    hyper_graph_nodes_length: usize,
    current_hyper_graph_node_index: Option<usize>,
    current_stateful_hyper_graph_node_per_hyper_graph_node_index: Vec<Rc<StatefulHyperGraphNode<T>>>,
    current_stateful_hyper_graph_node_index_per_hyper_graph_node_index: Vec<usize>,
    possible_states: Vec<Rc<T>>
}

impl<T: PartialEq> HyperGraphClicheShifter<T> {
    pub fn new(stateful_hyper_graph_nodes_per_hyper_graph_node_index: Vec<Vec<Rc<StatefulHyperGraphNode<T>>>>) -> Self {
        let hyper_graph_nodes_length = stateful_hyper_graph_nodes_per_hyper_graph_node_index.len();
        let mut possible_states: Vec<Rc<T>> = Vec::new();
        for hyper_graph_node_index in 0..stateful_hyper_graph_nodes_per_hyper_graph_node_index.len() {
            for stateful_hyper_graph_node in stateful_hyper_graph_nodes_per_hyper_graph_node_index[hyper_graph_node_index].iter() {
                let mut is_state_in_possible_states = false;
                for possible_state in possible_states.iter() {
                    if possible_state == &stateful_hyper_graph_node.state {
                        is_state_in_possible_states = true;
                        break;
                    }
                }
                if !is_state_in_possible_states {
                    possible_states.push(stateful_hyper_graph_node.state.clone());
                }
            }
        }
        HyperGraphClicheShifter {
            stateful_hyper_graph_nodes_per_hyper_graph_node_index: stateful_hyper_graph_nodes_per_hyper_graph_node_index,
            hyper_graph_nodes_length: hyper_graph_nodes_length,
            current_hyper_graph_node_index: None,
            current_stateful_hyper_graph_node_per_hyper_graph_node_index: Vec::new(),
            current_stateful_hyper_graph_node_index_per_hyper_graph_node_index: Vec::new(),
            possible_states: possible_states
        }
    }
}

impl<T: PartialEq> Shifter for HyperGraphClicheShifter<T> {
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
            return true;
        }
        else {
            self.current_hyper_graph_node_index = Some(0);
            return true;
        }
    }
    fn try_backward(&mut self) -> bool {
        if let Some(current_hyper_graph_node_index) = self.current_hyper_graph_node_index {
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
            let initial_stateful_hyper_graph_node_index;
            if current_hyper_graph_node_index != self.current_stateful_hyper_graph_node_per_hyper_graph_node_index.len() {
                // the first stateful_hyper_graph_node needs to be found
                initial_stateful_hyper_graph_node_index = 0;
            }
            else {
                // the next stateful_hyper_graph_node needs to be found
                initial_stateful_hyper_graph_node_index = self.current_stateful_hyper_graph_node_index_per_hyper_graph_node_index[current_hyper_graph_node_index] + 1;
            }

            for current_stateful_hyper_graph_node_index in initial_stateful_hyper_graph_node_index..self.stateful_hyper_graph_nodes_per_hyper_graph_node_index[current_hyper_graph_node_index].len() {
                let current_stateful_hyper_graph_node = &self.stateful_hyper_graph_nodes_per_hyper_graph_node_index[current_hyper_graph_node_index][current_stateful_hyper_graph_node_index];
                // TODO check to see that the previous stateful_hyper_graph_nodes permit the state of this hyper_graph_node
                let mut is_current_stateful_hyper_graph_node_valid = true;

                let mut previous_hyper_graph_node_index = current_hyper_graph_node_index;
                while previous_hyper_graph_node_index != 0 {
                    previous_hyper_graph_node_index -= 1;
                    // loop over the stateful_hyper_graph_node neighbors, searching for the current stateful_hyper_graph_node state
                    let mut previous_neighbor_stateful_hyper_graph_node_exists_with_same_state = false;
                    for previous_neighbor_stateful_hyper_graph_node in self.current_stateful_hyper_graph_node_per_hyper_graph_node_index[previous_hyper_graph_node_index].neighbor_stateful_hyper_graph_nodes_per_hyper_graph_node_index[current_hyper_graph_node_index].iter() {
                        if previous_neighbor_stateful_hyper_graph_node.state == current_stateful_hyper_graph_node.state {
                            previous_neighbor_stateful_hyper_graph_node_exists_with_same_state = true;
                            break;
                        }
                    }
                    if !previous_neighbor_stateful_hyper_graph_node_exists_with_same_state {
                        is_current_stateful_hyper_graph_node_valid = false;
                        break;
                    }
                }

                if is_current_stateful_hyper_graph_node_valid {
                    self.current_stateful_hyper_graph_node_per_hyper_graph_node_index.push(current_stateful_hyper_graph_node.clone());
                    self.current_stateful_hyper_graph_node_index_per_hyper_graph_node_index.push(current_stateful_hyper_graph_node_index);
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
            return IndexedElement::new(self.current_stateful_hyper_graph_node_per_hyper_graph_node_index[current_hyper_graph_node_index].state.clone(), current_hyper_graph_node_index);
        }
        panic!("Unexpected attempt to get indexed element without moving forward and incrementing.");
    }
    fn get_length(&self) -> usize {
        return self.hyper_graph_nodes_length;
    }
    fn get_element_index_and_state_index(&self) -> (usize, usize) {
        if let Some(current_hyper_graph_node_index) = self.current_hyper_graph_node_index {
            let state = &self.current_stateful_hyper_graph_node_per_hyper_graph_node_index[current_hyper_graph_node_index].state;
            for (possible_state_index, possible_state) in self.possible_states.iter().enumerate() {
                if possible_state == state {
                    return (current_hyper_graph_node_index, possible_state_index);
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