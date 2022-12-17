pub struct IndexIncrementer {
    maximum_exclusive_indexes: Vec<usize>,
    current_indexes: Vec<usize>,
    current_indexes_length: usize
}

impl IndexIncrementer {
    pub fn new(maximum_exclusive_indexes: Vec<usize>) -> Self {
        let mut current_indexes: Vec<usize> = Vec::new();
        let current_indexes_length: usize = maximum_exclusive_indexes.len();
        for (maximum_exclusive_index_index, maximum_exclusive_index) in maximum_exclusive_indexes.iter().enumerate() {
            current_indexes.push(0);
            if maximum_exclusive_index == &0 {
                panic!("Maximum exclusive index at {} must be greater than zero.", maximum_exclusive_index_index);
            }
        }
        IndexIncrementer {
            maximum_exclusive_indexes: maximum_exclusive_indexes,
            current_indexes: current_indexes,
            current_indexes_length: current_indexes_length
        }
    }
    pub fn try_increment(&mut self) -> bool {
        if self.current_indexes_length != 0 {
            let mut current_pointer_index = 0;
            while self.current_indexes[current_pointer_index] + 1 == self.maximum_exclusive_indexes[current_pointer_index] {
                self.current_indexes[current_pointer_index] = 0;
                current_pointer_index += 1;
                if current_pointer_index == self.current_indexes_length {
                    // we have cycled back around after reseting everything
                    return false;
                }
            }
            self.current_indexes[current_pointer_index] += 1;
            return true;
        }
        return false;
    }
    pub fn get(&self) -> Vec<usize> {
        self.current_indexes.clone()
    }
}

#[cfg(test)]
mod segment_container_tests {
    use super::*;
    use rstest::rstest;

    fn init() {
        std::env::set_var("RUST_LOG", "trace");
        //pretty_env_logger::try_init();
    }

    #[rstest]
    fn initialize() {
        init();

        let _ = IndexIncrementer::new(Vec::new());
    }

    #[rstest]
    #[should_panic]
    fn increment_zero() {
        init();

        let _ = IndexIncrementer::new(vec![0]);
    }

    #[rstest]
    #[case(1)]
    #[case(2)]
    #[case(3)]
    #[case(4)]
    #[case(5)]
    #[case(100)]
    fn increment_one(#[case] maximum_exclusive_index: usize) {
        init();

        let mut index_incrementer = IndexIncrementer::new(vec![maximum_exclusive_index]);
        for index in 0..(maximum_exclusive_index * 10) {
            assert_eq!(vec![index % maximum_exclusive_index], index_incrementer.get());
            let is_successful = index_incrementer.try_increment();
            if (index + 1) % maximum_exclusive_index == 0 {
                assert!(!is_successful);
            }
            else {
                assert!(is_successful);
            }
        }
        assert_eq!(vec![0], index_incrementer.get());
    }

    #[rstest]
    #[case(1, 1)]
    #[case(10, 10)]
    #[case(5, 17)]
    #[case(31, 19)]
    fn increment_two(#[case] first_maximum_exclusive_index: usize, #[case] second_maximum_exclusive_index: usize) {
        init();

        let mut index_incrementer = IndexIncrementer::new(vec![first_maximum_exclusive_index, second_maximum_exclusive_index]);
        for index in 0..(first_maximum_exclusive_index * second_maximum_exclusive_index) {
            assert_eq!(index % first_maximum_exclusive_index, index_incrementer.get()[0]);
            assert_eq!(index / first_maximum_exclusive_index, index_incrementer.get()[1]);
            let is_successful = index_incrementer.try_increment();
            if (index + 1) % (first_maximum_exclusive_index * second_maximum_exclusive_index) == 0 {
                assert!(!is_successful);
            }
            else {
                assert!(is_successful);
            }
        }
        assert_eq!(vec![0, 0], index_incrementer.get());
    }
}