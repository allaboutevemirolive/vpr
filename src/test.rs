#[cfg(test)]
mod test_output {
    use crate::*;

    fn test_init() {
        let mut stations = Stations::new();
        stations.push(Station::new("A".to_string()));
        stations.push(Station::new("B".to_string()));
        stations.push(Station::new("C".to_string()));

        // Handle unsorted order
        stations.sort();

        let mut edges = Edges::new();
        edges.push(
            "E1".to_string(),
            stations.get_station_idx("A".to_string()).unwrap(),
            stations.get_station_idx("B".to_string()).unwrap(),
            30,
        );

        edges.push(
            "E2".to_string(),
            stations.get_station_idx("B".to_string()).unwrap(),
            stations.get_station_idx("C".to_string()).unwrap(),
            10,
        );

        let mut pkg_collection = PackageCollection::new();
        pkg_collection.add_package("K1".to_string(), PackageStatus::AwaitingPickup);

        let mut packages = Packages::new();
        packages.push_with(
            "K1".to_string(),
            5,
            stations.get_station_idx("A".to_string()).unwrap(),
            stations.get_station_idx("C".to_string()).unwrap(),
            PackageStatus::AwaitingPickup,
        );

        let mut trains = Trains::new();
        trains.push_with(
            "Q1".to_string(),
            6,
            stations.get_station_idx("B".to_string()).unwrap(),
            stations.get_station_idx("B".to_string()).unwrap(),
            Package::default(),
            0,
        );

        let mut graph = Graph::new();
        graph.init_key(stations.clone());
        graph.init_value(edges.clone(), stations.clone());

        dbg!(&graph);

        let mut dist_map = DistanceMap::new();
        dist_map.init_key_value(graph.clone(), edges, stations.clone());
        dist_map.sorted_entries();
    }
}
