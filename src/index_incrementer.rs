pub struct IndexIncrementer {
    maximum_exclusive_indexes: Vec<usize>,
    current_indexes: Vec<Option<usize>>,
    current_indexes_length: usize
}

impl IndexIncrementer {
    pub fn new(maximum_exclusive_indexes: Vec<usize>) -> Self {
        let mut current_indexes: Vec<Option<usize>> = Vec::new();
        let current_indexes_length: usize = maximum_exclusive_indexes.len();
        for (maximum_exclusive_index_index, maximum_exclusive_index) in maximum_exclusive_indexes.iter().enumerate() {
            if maximum_exclusive_index == &0 {
                current_indexes.push(None);
            }
            else {
                current_indexes.push(Some(0));
            }
        }
        IndexIncrementer {
            maximum_exclusive_indexes: maximum_exclusive_indexes,
            current_indexes: current_indexes,
            current_indexes_length: current_indexes_length
        }
    }
    pub fn from_vector_of_vector<T>(vector_of_vector: &Vec<Vec<T>>) -> Self {
        let mut maximum_exclusive_indexes: Vec<usize> = Vec::new();
        for inner_vector in vector_of_vector.iter() {
            maximum_exclusive_indexes.push(inner_vector.len());
        }
        IndexIncrementer::new(maximum_exclusive_indexes)
    }
    pub fn try_increment(&mut self) -> bool {
        if self.current_indexes_length != 0 {
            let mut current_pointer_index = 0;
            let mut current_index_option = self.current_indexes[current_pointer_index];
            let mut maximum_exclusive_index = self.maximum_exclusive_indexes[current_pointer_index];
            while maximum_exclusive_index == 0 || current_index_option.unwrap() + 1 == maximum_exclusive_index {
                if maximum_exclusive_index != 0 {
                    self.current_indexes[current_pointer_index] = Some(0);
                }
                current_pointer_index += 1;
                if current_pointer_index == self.current_indexes_length {
                    // we have reached the end
                    return false;
                }
                current_index_option = self.current_indexes[current_pointer_index];
                maximum_exclusive_index = self.maximum_exclusive_indexes[current_pointer_index];
            }
            self.current_indexes[current_pointer_index] = Some(current_index_option.unwrap() + 1);
            return true;
        }
        return false;
    }
    pub fn get(&self) -> Vec<Option<usize>> {
        self.current_indexes.clone()
    }
    pub fn reset(&mut self) {
        for (maximum_exclusive_index_index, maximum_exclusive_index) in self.maximum_exclusive_indexes.iter().enumerate() {
            if maximum_exclusive_index == &0 {
                self.current_indexes[maximum_exclusive_index_index] = None;
            }
            else {
                self.current_indexes[maximum_exclusive_index_index] = Some(0);
            }
        }
    }
}

#[cfg(test)]
mod index_incrementer_tests {
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
    fn increment_zero() {
        init();

        let mut index_incrementer = IndexIncrementer::new(vec![0]);
        for _ in 0..100 {
            let indexes = index_incrementer.get();
            assert_eq!(1, indexes.len());
            assert!(indexes[0].is_none());
            index_incrementer.try_increment();
        }
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
            assert_eq!(vec![Some(index % maximum_exclusive_index)], index_incrementer.get());
            let is_successful = index_incrementer.try_increment();
            if (index + 1) % maximum_exclusive_index == 0 {
                assert!(!is_successful);
            }
            else {
                assert!(is_successful);
            }
        }
        assert_eq!(vec![Some(0)], index_incrementer.get());
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
            assert_eq!(Some(index % first_maximum_exclusive_index), index_incrementer.get()[0]);
            assert_eq!(Some(index / first_maximum_exclusive_index), index_incrementer.get()[1]);
            let is_successful = index_incrementer.try_increment();
            if (index + 1) % (first_maximum_exclusive_index * second_maximum_exclusive_index) == 0 {
                assert!(!is_successful);
            }
            else {
                assert!(is_successful);
            }
        }
        assert_eq!(vec![Some(0), Some(0)], index_incrementer.get());
    }
}