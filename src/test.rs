#[cfg(test)]
mod test_output {
    use crate::*;

    // cargo test test_first -- --nocapture
    #[test]
    fn test_first() {
        let mut station_collection = StationCollection::new();
        station_collection.add_station("A".to_string());
        station_collection.add_station("B".to_string());
        station_collection.add_station("C".to_string());
        station_collection.add_station("D".to_string());
        station_collection.add_station("E".to_string());
        station_collection.add_station("F".to_string());
        station_collection.add_station("G".to_string());

        let mut edge_storage = EdgeStorage::new();
        edge_storage.push("E1".to_string(), "A".to_string(), "B".to_string(), 14);
        edge_storage.push("E2".to_string(), "B".to_string(), "C".to_string(), 35);
        edge_storage.push("E3".to_string(), "C".to_string(), "D".to_string(), 48);
        edge_storage.push("E4".to_string(), "D".to_string(), "E".to_string(), 32);
        edge_storage.push("E5".to_string(), "E".to_string(), "F".to_string(), 63);
        edge_storage.push("E6".to_string(), "F".to_string(), "G".to_string(), 41);

        let mut package_collection = PackageCollection::new();
        package_collection.add_package(
            "K1".to_string(),
            48,
            "D".to_string(),
            "A".to_string(),
            PackageStatus::AwaitingPickup,
        );

        package_collection.add_package(
            "K2".to_string(),
            38,
            "E".to_string(),
            "F".to_string(),
            PackageStatus::AwaitingPickup,
        );

        let mut package_collection_2 = PackageCollection::new();
        package_collection_2.add_package(
            "K1".to_string(),
            48,
            "D".to_string(),
            "A".to_string(),
            PackageStatus::AwaitingPickup,
        );

        package_collection_2.add_package(
            "K2".to_string(),
            38,
            "E".to_string(),
            "F".to_string(),
            PackageStatus::AwaitingPickup,
        );

        let mut train_collection = TrainCollection::new();
        train_collection.add_train(
            "Q1".to_string(),
            65,
            65,
            "E".to_string(),
            "E".to_string(),
            Vec::new(),
            0,
        );

        train_collection.add_train(
            "Q2".to_string(),
            57,
            57,
            "G".to_string(),
            "G".to_string(),
            Vec::new(),
            0,
        );

        train_collection.add_train(
            "Q3".to_string(),
            46,
            46,
            "F".to_string(),
            "F".to_string(),
            Vec::new(),
            0,
        );

        let mut graph = Graph::new();
        graph.init_key(&station_collection);
        graph.init_value(&edge_storage);

        let mut distance_map = DistanceMap::new();
        distance_map.init_key_value(graph.clone(), edge_storage);

        let mut timeline = Timeline::new();
        for tr in train_collection.iter_mut() {
            timeline.insert(tr.name().to_string(), 0);
        }

        let mut train_movement = TrainMovement::new();
        // let package_candidates = PackageCandidates::new();

        let mut package_tracker = PackageTracker::new();

        for pkg in package_collection.iter() {
            package_tracker.add_package(pkg.clone(), PackageStatus::AwaitingPickup);
        }

        let mut package_name = PackageName::new();
        for pkg in package_collection.iter() {
            package_name.add_name(pkg.name().to_string());
        }

        let loggerize = Logger::new();

        tracer!(&graph);
        tracer!(&train_collection);
        tracer!(&station_collection);

        start_searching(
            package_collection,
            package_collection_2,
            &mut train_collection,
            station_collection,
            graph,
            &mut train_movement,
            &mut distance_map,
            &mut timeline,
            package_tracker,
            loggerize,
        );
    }
}
