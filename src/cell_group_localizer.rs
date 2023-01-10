use std::{rc::Rc, cell::RefCell};

use crate::{CellGroup, incrementer::{round_robin_incrementer::RoundRobinIncrementer, Incrementer}};



pub struct CellGroupLocalizer {
    cell_groups: Vec<Rc<CellGroup>>,
    current_round_robin_incrementer: RefCell<RoundRobinIncrementer<(u8, u8)>>
}

impl CellGroupLocalizer {
    pub fn new(cell_groups: Vec<Rc<CellGroup>>) -> Self {
        let mut incrementers: Vec<Rc<RefCell<dyn Incrementer<T = (u8, u8)>>>> = Vec::new();

        let round_robin_incrementer: RoundRobinIncrementer<(u8, u8)> = RoundRobinIncrementer::new(incrementers);
        CellGroupLocalizer {
            cell_groups: cell_groups,
            current_round_robin_incrementer: RefCell::new(round_robin_incrementer)
        }
    }
}