#[cfg(test)]
mod test_output {
    use crate::*;

    // cargo test test_init -- --nocapture
    #[test]
    fn test_init() {
        let output_steps: String = String::from(
            "W=0, T=Q1, N1=B, P1=[], N2=A, P2=[]\nW=30, T=Q1, N1=A, P1=[K1], N2=B, P2=[]\nW=60, T=Q1, N1=B, P1=[], N2=C, P2=[K1]\n",
        );

        let mut stations = Stations::new();
        stations.push(Station::new("A".to_string()));
        stations.push(Station::new("B".to_string()));
        stations.push(Station::new("C".to_string()));

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

        let mut dist_map = DistanceMap::new();
        dist_map.init_key_value(graph.clone(), edges, stations.clone());
        dist_map.sorted_entries();

        tracer!(&dist_map);

        // let mut trains_tracker = TrainsTracker::new();

        let mut timeline = Timeline::new();

        for (_, tr) in trains.enumerate_trains() {
            timeline.insert(tr.name().to_string(), 0);
        }

        let mut pkg_map = PackageTrainMapping::new();

        find_train_candidates(&packages, &trains, &mut timeline, &mut pkg_map);
        traced!(&pkg_map);

        tracer!(&timeline);

        let train_movement = TrainMovement::new(&stations);
        let logger = Logger::new();
        let pkg_cands = PackageCandidates::new();

        let where_to_go = "".to_string();
        let next_station = "".to_string();

        let num = 0;

        start_searching(
            num,
            packages,
            pkg_map,
            pkg_collection,
            trains,
            stations,
            where_to_go,
            graph,
            next_station,
            pkg_cands,
            logger,
            train_movement,
            dist_map,
            &mut timeline,
        );
    }

    // cargo test test_second -- --nocapture
    #[test]
    fn test_second() {
        let mut stations = Stations::new();
        stations.push(Station::new("A".to_string()));
        stations.push(Station::new("B".to_string()));
        stations.push(Station::new("C".to_string()));
        stations.push(Station::new("D".to_string()));
        stations.push(Station::new("E".to_string()));
        stations.push(Station::new("F".to_string()));
        stations.push(Station::new("G".to_string()));

        stations.sort();

        let mut edges = Edges::new();
        edges.push(
            "E1".to_string(),
            stations.get_station_idx("A".to_string()).unwrap(),
            stations.get_station_idx("B".to_string()).unwrap(),
            14,
        );

        edges.push(
            "E2".to_string(),
            stations.get_station_idx("B".to_string()).unwrap(),
            stations.get_station_idx("C".to_string()).unwrap(),
            35,
        );

        edges.push(
            "E3".to_string(),
            stations.get_station_idx("C".to_string()).unwrap(),
            stations.get_station_idx("D".to_string()).unwrap(),
            48,
        );

        edges.push(
            "E4".to_string(),
            stations.get_station_idx("D".to_string()).unwrap(),
            stations.get_station_idx("E".to_string()).unwrap(),
            32,
        );

        edges.push(
            "E5".to_string(),
            stations.get_station_idx("E".to_string()).unwrap(),
            stations.get_station_idx("F".to_string()).unwrap(),
            63,
        );

        edges.push(
            "E6".to_string(),
            stations.get_station_idx("F".to_string()).unwrap(),
            stations.get_station_idx("G".to_string()).unwrap(),
            41,
        );

        let mut pkg_collection = PackageCollection::new();
        pkg_collection.add_package("K1".to_string(), PackageStatus::AwaitingPickup);
        pkg_collection.add_package("K2".to_string(), PackageStatus::AwaitingPickup);

        let mut packages = Packages::new();
        packages.push_with(
            "K1".to_string(),
            48,
            stations.get_station_idx("D".to_string()).unwrap(),
            stations.get_station_idx("A".to_string()).unwrap(),
            PackageStatus::AwaitingPickup,
        );

        packages.push_with(
            "K2".to_string(),
            38,
            stations.get_station_idx("E".to_string()).unwrap(),
            stations.get_station_idx("F".to_string()).unwrap(),
            PackageStatus::AwaitingPickup,
        );

        let mut trains = Trains::new();
        trains.push_with(
            "Q1".to_string(),
            65,
            stations.get_station_idx("E".to_string()).unwrap(),
            stations.get_station_idx("E".to_string()).unwrap(),
            Package::default(),
            0,
        );

        trains.push_with(
            "Q2".to_string(),
            57,
            stations.get_station_idx("G".to_string()).unwrap(),
            stations.get_station_idx("G".to_string()).unwrap(),
            Package::default(),
            0,
        );

        trains.push_with(
            "Q3".to_string(),
            46,
            stations.get_station_idx("F".to_string()).unwrap(),
            stations.get_station_idx("F".to_string()).unwrap(),
            Package::default(),
            0,
        );

        let mut graph = Graph::new();
        graph.init_key(stations.clone());
        graph.init_value(edges.clone(), stations.clone());

        tracer!(&graph);

        let mut dist_map = DistanceMap::new();
        dist_map.init_key_value(graph.clone(), edges, stations.clone());
        dist_map.sorted_entries();

        tracer!(&dist_map);

        // let mut trains_tracker = TrainsTracker::new();

        let mut timeline = Timeline::new();

        for (_, tr) in trains.enumerate_trains() {
            timeline.insert(tr.name().to_string(), 0);
        }

        let mut pkg_map = PackageTrainMapping::new();

        find_train_candidates(&packages, &trains, &mut timeline, &mut pkg_map);
        traced!(&pkg_map);

        tracer!(&timeline);

        let train_movement = TrainMovement::new(&stations);
        let logger = Logger::new();
        let pkg_cands = PackageCandidates::new();

        let where_to_go = "".to_string();
        let next_station = "".to_string();

        let num = 0;

        start_searching(
            num,
            packages,
            pkg_map,
            pkg_collection,
            trains,
            stations,
            where_to_go,
            graph,
            next_station,
            pkg_cands,
            logger,
            train_movement,
            dist_map,
            &mut timeline,
        );
    }
}
