use std::collections::{HashMap, HashSet};

#[derive(Debug, PartialEq, Eq, Hash)]
enum Status {
    AtPickup,
    InFlight,
    Delivered,
}

#[derive(Debug, PartialEq, Eq, Hash)]
enum Direction {
    Left,
    Right,
}

// Function to calculate station positions
fn calculate_positions<'a>(stations: &'a Vec<&'a str>) -> HashMap<&'a str, usize> {
    stations
        .iter()
        .enumerate()
        .map(|(i, station)| (*station, i))
        .collect()
}

// Function to build connection maps and distances
fn build_connections_and_distances<'a>(
    edges: &'a Vec<&'a str>,
) -> (
    HashMap<&'a str, HashSet<&'a str>>,
    HashMap<(&'a str, &'a str), usize>,
) {
    edges.iter().fold(
        (HashMap::new(), HashMap::new()),
        |(mut connections, mut distances), edge_str| {
            let parts: Vec<&str> = edge_str.split(",").collect();
            let src = parts[1];
            let dst = parts[2];
            let distance: usize = parts[3].parse().unwrap();

            connections
                .entry(src)
                .or_insert_with(HashSet::new)
                .insert(dst);
            connections
                .entry(dst)
                .or_insert_with(HashSet::new)
                .insert(src);

            distances.insert((src, dst), distance);
            distances.insert((dst, src), distance);

            (connections, distances)
        },
    )
}

// Function to extract train names
fn extract_train_names<'a>(trains: &'a Vec<&'a str>) -> Vec<&'a str> {
    trains
        .iter()
        .map(|train| train.split(",").next().unwrap())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_positions() {
        let stations = vec!["X", "Y", "Z"];
        let expected = HashMap::from([("X", 0), ("Y", 1), ("Z", 2)]);
        assert_eq!(calculate_positions(&stations), expected);
    }

    #[test]
    fn test_build_connections_and_distances() {
        let edges = vec!["E1,X,Y,5", "E2,Y,Z,10"];
        let (connections, distances) = build_connections_and_distances(&edges);

        let expected_connections = HashMap::from([
            ("X", HashSet::from(["Y"])),
            ("Y", HashSet::from(["X", "Z"])),
            ("Z", HashSet::from(["Y"])),
        ]);
        let expected_distances = HashMap::from([
            (("X", "Y"), 5),
            (("Y", "X"), 5),
            (("Y", "Z"), 10),
            (("Z", "Y"), 10),
        ]);

        assert_eq!(connections, expected_connections);
        assert_eq!(distances, expected_distances);
    }

    #[test]
    fn test_extract_train_names() {
        let trains = vec!["Q1,A,LEFT", "Q2,B,RIGHT", "Q3,C,LEFT"];
        let expected = vec!["Q1", "Q2", "Q3"];
        assert_eq!(extract_train_names(&trains), expected);
    }
}

fn create_delivery_status<'a>(deliveries: &'a [&'a str]) -> HashMap<&'a str, Status> {
    deliveries
        .iter()
        .map(|pkg| (*pkg, Status::AtPickup))
        .collect()
}

fn build_train_info<'a>(
    trains: &'a [&'a str],
) -> (
    HashMap<&'a str, &'a str>,
    HashMap<&'a str, usize>,
    HashMap<&'a str, Vec<&'a str>>,
) {
    trains.iter().fold(
        (HashMap::new(), HashMap::new(), HashMap::new()),
        |(mut stations, mut capacities, mut loads), train_str| {
            let parts: Vec<&str> = train_str.split(",").collect();
            let train = parts[0];
            let capacity: usize = parts[1].parse().unwrap();
            let station = parts[2];

            stations.insert(train, station);
            capacities.insert(train, capacity);
            loads.insert(train, Vec::new());

            (stations, capacities, loads)
        },
    )
}

fn create_timeline<'a>(train_names: &'a [&'a str]) -> HashMap<&'a str, usize> {
    train_names.iter().map(|train| (*train, 0)).collect()
}

// ... (Your existing code: enum definitions, functions)

#[cfg(test)]
mod tests_second {
    use super::*;

    #[test]
    fn test_create_delivery_status() {
        let deliveries = vec!["P1", "P2", "P3"];
        let expected = HashMap::from([
            ("P1", Status::AtPickup),
            ("P2", Status::AtPickup),
            ("P3", Status::AtPickup),
        ]);

        assert_eq!(create_delivery_status(&deliveries), expected);
    }

    #[test]
    fn test_build_train_info() {
        let trains = vec!["T1,3,A", "T2,2,B"];
        let (stations, capacities, loads) = build_train_info(&trains);

        let expected_stations = HashMap::from([("T1", "A"), ("T2", "B")]);
        let expected_capacities = HashMap::from([("T1", 3), ("T2", 2)]);
        let expected_loads: HashMap<&str, Vec<&str>> =
            HashMap::from([("T1", Vec::new()), ("T2", Vec::new())]);

        assert_eq!(stations, expected_stations);
        assert_eq!(capacities, expected_capacities);
        assert_eq!(loads, expected_loads);
    }

    #[test]
    fn test_create_timeline() {
        let train_names = vec!["T1", "T2", "T3"];
        let expected = HashMap::from([("T1", 0), ("T2", 0), ("T3", 0)]);
        assert_eq!(create_timeline(&train_names), expected);
    }

    // Additional Tests for Edge Cases
    #[test]
    fn test_empty_inputs() {
        let empty_vec: Vec<&str> = Vec::new();
        assert!(create_delivery_status(&empty_vec).is_empty());
        assert!(build_train_info(&empty_vec).0.is_empty());
        assert!(create_timeline(&empty_vec).is_empty());
    }

    #[test]
    #[should_panic] // Expect a panic due to invalid capacity format
    fn test_invalid_train_format() {
        let trains = vec!["T1,X,A"]; // 'X' is not a valid usize
        build_train_info(&trains); // This should panic
    }
}

// Assuming you're using HashMap for the data structures

// Function to load a package onto a train
fn load_package<'a>(
    train: &'a str,
    pkg: &'a str,
    train_loads: &mut HashMap<&'a str, Vec<&'a str>>,
    delivery_status: &mut HashMap<&'a str, Status>,
) {
    if train_loads.get(train).unwrap_or(&vec![]).contains(&pkg) {
        return;
    }

    train_loads.entry(train).or_default().push(pkg);
    delivery_status.insert(pkg, Status::InFlight);
}

// Function to unload a package from a train
fn unload_package<'a>(
    train: &'a str,
    pkg: &'a str,
    train_loads: &mut HashMap<&'a str, Vec<&'a str>>,
    deliveries: &mut Vec<&'a str>,
    delivery_status: &mut HashMap<&'a str, Status>,
) {
    if !train_loads.get(train).unwrap_or(&vec![]).contains(&pkg) {
        return;
    }

    let train_pkg_index = train_loads[train].iter().position(|&x| x == pkg).unwrap();
    train_loads.get_mut(train).unwrap().remove(train_pkg_index);

    let pkg_index = deliveries
        .iter()
        .position(|x| x.split(",").next().unwrap() == pkg)
        .unwrap();
    deliveries.remove(pkg_index);

    delivery_status.insert(pkg, Status::Delivered);
}

// Function to get package details
fn get_pkg_detail<'a>(pkg_name: &'a str, deliveries: &'a [&'a str]) -> HashMap<&'a str, String> {
    let pkg_option = deliveries.iter().find(|x| {
        let parts: Vec<&str> = x.split(",").collect();
        parts[0] == pkg_name
    });

    match pkg_option {
        Some(pkg) => {
            let parts: Vec<&str> = pkg.split(",").collect();
            let mut details = HashMap::new();
            details.insert("name", parts[0].to_string());
            details.insert("weight", parts[1].to_string());
            details.insert("from", parts[2].to_string());
            details.insert("to", parts[3].to_string());
            details
        }
        None => {
            let mut details = HashMap::new();
            details.insert("weight", "0".to_string()); // Default weight to 0 if not found
            details
        }
    }
}

// ... (other functions: get_pkg_detail, get_train_for_weight, get_diff)

// fn find_closest_trains<'a>(
//     deliveries: &'a [&'a str],
//     positions: &HashMap<&'a str, usize>,
//     train_names: &'a [&'a str],
//     train_stations: &HashMap<&'a str, &'a str>,
//     train_capacities: &HashMap<&'a str, usize>,
//     train_loads: &HashMap<&'a str, Vec<&'a str>>,
//     timeline: &HashMap<&'a str, usize>,
//     edges: &'a [&'a str],
// ) -> HashMap<&'a str, &'a str> {
//     deliveries.iter().fold(HashMap::new(), |mut acc, delivery| {
//         let details = get_pkg_detail(delivery, deliveries);
//         let pkg_pos = positions[details["from"].as_str()];

//         let (candidate, _) = train_names.iter().fold(
//             (
//                 get_train_for_weight(
//                     details["weight"].parse().unwrap(),
//                     train_names,
//                     train_capacities,
//                 ),
//                 edges.len(),
//             ),
//             |(candidate, distance), &train| {
//                 let train_pos = positions[train_stations[train]];
//                 let diff = get_diff(pkg_pos, train_pos);

//                 if get_train_remaining_capacity(train, train_capacities, train_loads, deliveries)
//                     >= details["weight"].parse().unwrap()
//                     && (diff < distance
//                         || (diff == distance
//                             && timeline[train] < timeline[candidate.unwrap_or("")]))
//                 {
//                     (Some(train), diff)
//                 } else {
//                     (candidate, distance)
//                 }
//             },
//         );

//         acc.insert(details["name"].as_str(), candidate.unwrap());
//         acc
//     })
// }

// Corrected get_train_remaining_capacity function
fn get_train_remaining_capacity<'a>(
    train: &'a str,
    train_capacities: &HashMap<&'a str, usize>,
    train_loads: &HashMap<&'a str, Vec<&'a str>>, // Corrected type
    deliveries: &'a [&'a str],
) -> usize {
    let capacity = *train_capacities.get(train).unwrap_or(&0);
    let load_weight: usize = train_loads
        .get(train)
        .unwrap_or(&vec![])
        .iter()
        .map(|pkg| {
            let details = get_pkg_detail(pkg, deliveries);
            details["weight"].parse::<usize>().unwrap_or(0)
        })
        .sum();
    capacity.saturating_sub(load_weight)
}

// // Function to find a train with enough capacity
// fn get_train_for_weight<'a>(
//     weight: usize,
//     train_names: &'a [&'a str],
//     train_capacities: &HashMap<&'a str, usize>,
// ) -> Option<&'a str> {
//     train_names
//         .iter()
//         .find(|train| train_capacities.get(&train).cloned().unwrap_or(0) >= weight)
//         .cloned()
// }
// fn get_train_for_weight<'a>(
//     weight: usize,
//     train_names: &'a [&'a str],
//     train_capacities: &HashMap<&'a str, usize>,
// ) -> Option<&'a str> {
//     train_names
//         .iter()
//         .filter(|train| train_capacities.get(*train).copied().unwrap_or(0) >= weight) // Filter suitable trains
//         .min_by_key(|train| train_capacities.get(*train).copied().unwrap_or(usize::MAX)) // Find minimum capacity
//         .copied()
// }

// Function to get the positive difference between two numbers
fn get_diff(a: usize, b: usize) -> usize {
    if a > b {
        a - b
    } else {
        b - a
    }
}

// ... (Your existing code: enum definitions, functions)

#[cfg(test)]
mod tests_third {
    use super::*;

    #[test]
    fn test_load_package() {
        let mut train_loads: HashMap<&str, Vec<&str>> = HashMap::new();
        let mut delivery_status: HashMap<&str, Status> = HashMap::new();
        train_loads.insert("T1", vec![]);

        load_package("T1", "P1", &mut train_loads, &mut delivery_status);
        assert_eq!(train_loads["T1"], vec!["P1"]);
        assert_eq!(delivery_status["P1"], Status::InFlight);

        // Test duplicate loading
        load_package("T1", "P1", &mut train_loads, &mut delivery_status);
        assert_eq!(train_loads["T1"], vec!["P1"]); // Should not load again
    }

    #[test]
    fn test_unload_package() {
        let mut train_loads: HashMap<&str, Vec<&str>> = HashMap::new();
        let mut deliveries: Vec<&str> = vec!["P1,2,A,B"];
        let mut delivery_status: HashMap<&str, Status> = HashMap::new();
        train_loads.insert("T1", vec!["P1"]);

        unload_package(
            "T1",
            "P1",
            &mut train_loads,
            &mut deliveries,
            &mut delivery_status,
        );
        assert!(train_loads["T1"].is_empty());
        assert!(deliveries.is_empty());
        assert_eq!(delivery_status["P1"], Status::Delivered);

        // Test unloading a non-existent package
        unload_package(
            "T1",
            "P2",
            &mut train_loads,
            &mut deliveries,
            &mut delivery_status,
        ); // Should do nothing
    }

    #[test]
    fn test_get_pkg_detail() {
        let deliveries = vec!["P1,3,A,C", "P2,5,B,D"];
        let details = get_pkg_detail("P1", &deliveries);
        assert_eq!(details["name"], "P1");
        assert_eq!(details["weight"], "3");
        assert_eq!(details["from"], "A");
        assert_eq!(details["to"], "C");

        // Test for non-existent package
        let details = get_pkg_detail("P3", &deliveries);
        assert_eq!(details["weight"], "0"); // Default weight if not found
    }

    #[test]
    fn test_get_train_remaining_capacity() {
        let train_capacities = HashMap::from([("T1", 10)]);
        let train_loads = HashMap::from([("T1", vec!["P1", "P2"])]);
        let deliveries = vec!["P1,2,A,B", "P2,3,C,D"];
        assert_eq!(
            get_train_remaining_capacity("T1", &train_capacities, &train_loads, &deliveries),
            5
        );

        // Test with an empty train
        assert_eq!(
            get_train_remaining_capacity("T2", &train_capacities, &train_loads, &deliveries),
            0
        );
    }

    // #[test]
    // fn test_get_train_for_weight() {
    //     let train_capacities = HashMap::from([("T1", 5), ("T2", 10)]);
    //     let train_names = vec!["T1", "T2"];
    //     assert_eq!(
    //         get_train_for_weight(6, &train_names, &train_capacities),
    //         Some("T2")
    //     );

    //     // Test when no train has enough capacity
    //     assert_eq!(
    //         get_train_for_weight(11, &train_names, &train_capacities),
    //         None
    //     );
    // }

    //Test get_diff function
    #[test]
    fn test_get_diff() {
        assert_eq!(get_diff(5, 2), 3);
        assert_eq!(get_diff(1, 1), 0);
        assert_eq!(get_diff(2, 5), 3);
    }

    // use super::*;

    // #[test]
    // fn test_get_train_for_weight() {
    //     let train_capacities = HashMap::from([("T1", 5), ("T2", 10), ("T3", 8)]);
    //     let train_names = vec!["T1", "T2", "T3"];

    //     assert_eq!(
    //         get_train_for_weight(3, &train_names, &train_capacities),
    //         Some("T1")
    //     );

    //     // T2 is chosen because it has a smaller capacity than T3 (both can carry 7)
    //     assert_eq!(
    //         get_train_for_weight(7, &train_names, &train_capacities),
    //         Some("T2")
    //     );

    //     assert_eq!(
    //         get_train_for_weight(8, &train_names, &train_capacities),
    //         Some("T3")
    //     );
    // }

    // #[test]
    // fn test_get_train_for_weight_missing_capacity() {
    //     let train_capacities = HashMap::from([("T1", 5)]);
    //     let train_names = vec!["T1", "T2"]; // T2's capacity is missing

    //     // Even though T2 is in train_names, its capacity is not found, so it should not be selected
    //     assert_eq!(
    //         get_train_for_weight(3, &train_names, &train_capacities),
    //         Some("T1")
    //     );
    //     assert_eq!(
    //         get_train_for_weight(6, &train_names, &train_capacities),
    //         None
    //     );
    // }

    // #[test]
    // fn test_get_train_for_weight_zero_capacity() {
    //     let train_capacities = HashMap::from([("T1", 0)]);
    //     let train_names = vec!["T1"];

    //     // T1 has capacity 0, so it should not be selected
    //     assert_eq!(
    //         get_train_for_weight(1, &train_names, &train_capacities),
    //         None
    //     );
    // }

    //...(Add integration tests when you have a main loop in your main function)
}

// ... (other functions: get_pkg_detail, get_train_remaining_capacity,
//      get_train_for_weight, get_diff)

fn get_train_for_weight<'a>(
    weight: usize,
    train_names: &'a [&'a str],
    train_capacities: &HashMap<&'a str, usize>,
) -> Option<&'a str> {
    train_names
        .iter()
        .max_by_key(|train| {
            (
                train_capacities.get(*train).copied().unwrap_or(0) >= weight,
                train_capacities.get(*train).copied().unwrap_or(0),
            ) // Prioritize higher capacity
        })
        .copied()
}

// fn find_closest_trains<'a>(
//     deliveries: &'a [&'a str],
//     positions: &HashMap<&'a str, usize>,
//     train_names: &'a [&'a str],
//     train_stations: &HashMap<&'a str, &'a str>,
//     train_capacities: &HashMap<&'a str, usize>,
//     timeline: &HashMap<&'a str, usize>,
//     edges: &'a [&'a str], // You'll need to pass edges for its length
// ) -> HashMap<&'a str, &'a str> {
//     deliveries.iter().fold(HashMap::new(), |mut acc, delivery| {
//         let details = get_pkg_detail(delivery, deliveries);
//         let pkg_pos = positions[details["from"].as_str()];

//         let (candidate, _) = train_names.iter().fold(
//             (
//                 get_train_for_weight(
//                     details["weight"].parse().unwrap(),
//                     train_names,
//                     train_capacities,
//                 ),
//                 edges.len(), // Initialize distance with a large value
//             ),
//             |(candidate, distance), &train| {
//                 let train_pos = positions[train_stations[train]];
//                 let diff = get_diff(pkg_pos, train_pos);

//                 if get_train_remaining_capacity(train, train_capacities, train_stations, deliveries)
//                     >= details["weight"].parse().unwrap()
//                     && (diff < distance
//                         || (diff == distance
//                             && timeline[train] < timeline[candidate.unwrap_or("")]))
//                 {
//                     (Some(train), diff)
//                 } else {
//                     (candidate, distance)
//                 }
//             },
//         );

//         acc.insert(details["name"].as_str(), candidate.unwrap());
//         acc
//     })
// }
