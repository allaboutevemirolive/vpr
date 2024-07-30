#[cfg(test)]
mod unit_test {
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
        assert_eq!(result, vec![5]);
    }

    #[test]
    fn test_find_nearest_trains_multiple_closest() {
        let package_index = 10;
        let train_indices = vec![8, 10, 12];
        let result = find_nearest_trains(package_index, &train_indices);
        assert_eq!(result, vec![10]);
    }

    #[test]
    fn test_find_nearest_trains_all_same_distance() {
        let package_index = 5;
        let train_indices = vec![3, 7];
        let result = find_nearest_trains(package_index, &train_indices);
        assert_eq!(result, vec![3, 7]);
    }

    #[test]
    fn test_find_nearest_trains_single_train() {
        let package_index = 3;
        let train_indices = vec![5];
        let result = find_nearest_trains(package_index, &train_indices);
        assert_eq!(result, vec![5]);
    }

    #[test]
    fn test_find_nearest_trains_no_trains() {
        let package_index = 10;
        let train_indices: Vec<usize> = Vec::new();
        let result = find_nearest_trains(package_index, &train_indices);
        assert_eq!(result, Vec::<usize>::new());
    }

    #[test]
    fn test_find_nearest_trains_package_index_at_edge() {
        let package_index = 0;
        let train_indices = vec![0, 10, 20];
        let result = find_nearest_trains(package_index, &train_indices);
        assert_eq!(result, vec![0]);
    }
}

// #[test]
// fn test_first_and_remove_first() {
//     use crate::*;
//     let mut collection = PackageCollection::new();
//     #[derive(Debug, Clone, PartialEq, Eq)]
//     pub struct Package {
//         name: String,
//     }

//     impl Package {
//         pub fn new(name: &str) -> Self {
//             Self {
//                 name: name.to_string(),
//             }
//         }
//     }

//     // Test with an empty collection
//     assert_eq!(collection.first(), None);
//     assert_eq!(collection.pick_first(), None);

//     // Add some packages
//     collection
//         .packages
//         .insert("pkg1".to_string(), Package::new("Package 1"));
//     collection
//         .packages
//         .insert("pkg2".to_string(), Package::new("Package 2"));

//     // Test first method
//     let first_pkg = collection.first();
//     assert!(first_pkg.is_some());
//     if let Some((key, package)) = first_pkg {
//         assert_eq!(key, "pkg1");
//         assert_eq!(package.name, "Package 1");
//     }

//     // Test pick_first method
//     let removed_pkg = collection.pick_first();
//     assert!(removed_pkg.is_some());
//     if let Some((key, package)) = removed_pkg {
//         assert_eq!(key, "pkg1");
//         assert_eq!(package.name, "Package 1");
//     }

//     // Test first method again after removal
//     let first_pkg_after_removal = collection.first();
//     assert!(first_pkg_after_removal.is_some());
//     if let Some((key, package)) = first_pkg_after_removal {
//         assert_eq!(key, "pkg2");
//         assert_eq!(package.name, "Package 2");
//     }

//     // Remove the last package
//     let removed_pkg = collection.pick_first();
//     assert!(removed_pkg.is_some());
//     if let Some((key, package)) = removed_pkg {
//         assert_eq!(key, "pkg2");
//         assert_eq!(package.name, "Package 2");
//     }

//     // Test with an empty collection again
//     assert_eq!(collection.first(), None);
//     assert_eq!(collection.pick_first(), None);
//     assert!(collection.is_empty());

//     println!("{:?}", collection);
// }
