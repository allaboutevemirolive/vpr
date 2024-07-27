#[cfg(test)]
mod test {
    use crate::*;

    #[test]
    fn test_insert_and_get_time() {
        let mut timeline = Timeline::new();

        timeline.insert("TrainA".to_string(), 100);
        timeline.insert("TrainB".to_string(), 200);

        assert_eq!(timeline.get_time("TrainA"), 100);
        assert_eq!(timeline.get_time("TrainB"), 200);
        assert_eq!(timeline.get_time("TrainC"), 0);
    }

    #[test]
    fn test_is_traveled_less() {
        let mut timeline = Timeline::new();

        timeline.insert("TrainA".to_string(), 100);
        timeline.insert("TrainB".to_string(), 200);

        assert!(timeline.is_traveled_less("TrainA", "TrainB"));
        assert!(!timeline.is_traveled_less("TrainB", "TrainA"));
        assert!(!timeline.is_traveled_less("TrainA", "TrainA"));
    }

    #[test]
    fn test_modify_time() {
        let mut timeline = Timeline::new();

        timeline.insert("TrainA".to_string(), 100);
        timeline.modify_time("TrainA", 150);

        assert_eq!(timeline.get_time("TrainA"), 150);

        timeline.modify_time("TrainB", 200);
        assert_eq!(timeline.get_time("TrainB"), 0);
    }

    #[test]
    fn test_trains_with_less_time() {
        let mut timeline = Timeline::new();

        timeline.insert("TrainA".to_string(), 100);
        timeline.insert("TrainB".to_string(), 200);
        timeline.insert("TrainC".to_string(), 50);

        let less_time_trains = timeline.trains_with_less_time("TrainB");
        let expected_trains = vec!["TrainA".to_string(), "TrainC".to_string()];

        assert_eq!(less_time_trains, expected_trains);
    }
}

#[cfg(test)]
mod tests_find_nearest {
    use crate::*;

    #[test]
    fn test_find_nearest_trains_basic() {
        let package_index = 5;
        let train_indices = vec![2, 5, 8];
        let result = find_nearest_trains(package_index, &train_indices);
        assert_eq!(result, vec![5]); // Train at index 5 is closest
    }

    #[test]
    fn test_find_nearest_trains_multiple_closest() {
        let package_index = 10;
        let train_indices = vec![8, 10, 12];
        let result = find_nearest_trains(package_index, &train_indices);
        assert_eq!(result, vec![10]); // Train at index 10 is the closest
    }

    #[test]
    fn test_find_nearest_trains_all_same_distance() {
        let package_index = 5;
        let train_indices = vec![3, 7];
        let result = find_nearest_trains(package_index, &train_indices);
        assert_eq!(result, vec![3, 7]); // Both are at the same minimal distance of 2
    }

    #[test]
    fn test_find_nearest_trains_single_train() {
        let package_index = 3;
        let train_indices = vec![5];
        let result = find_nearest_trains(package_index, &train_indices);
        assert_eq!(result, vec![5]); // Only one train, so return it
    }

    #[test]
    fn test_find_nearest_trains_no_trains() {
        let package_index = 10;
        let train_indices: Vec<usize> = Vec::new();
        let result = find_nearest_trains(package_index, &train_indices);
        assert_eq!(result, Vec::<usize>::new()); // No trains, so return empty vector
    }

    #[test]
    fn test_find_nearest_trains_package_index_at_edge() {
        let package_index = 0;
        let train_indices = vec![0, 10, 20];
        let result = find_nearest_trains(package_index, &train_indices);
        assert_eq!(result, vec![0]); // Train at index 0 is the closest
    }

    // #[test]
    // fn test_find_nearest_trains_large_package_index() {
    //     let package_index = 100;
    //     let train_indices = vec![90, 110, 120];
    //     let result = find_nearest_trains(package_index, &train_indices);
    //     assert_eq!(result, vec![110]); // Train at index 110 is closest
    // }
}
