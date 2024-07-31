#[cfg(test)]
mod test_output {
    use crate::*;

    // Input:

    // stations: [
    //   'A', 'B', 'C',
    //   'D', 'E', 'F',
    //   'G'
    // ],
    // edges: [
    //   'E1,A,B,14',
    //   'E2,B,C,35',
    //   'E3,C,D,48',
    //   'E4,D,E,32',
    //   'E5,E,F,63',
    //   'E6,F,G,41'
    // ],
    // deliveries: [ 'K1,48,D,A', 'K2,38,E,F' ],
    // trains: [ 'Q1,65,E', 'Q2,57,G', 'Q3,46,F' ]

    // Output:

    // 'W=0, T=Q1, N1=E, P1=[], N2=D, P2=[], L=[]',
    // 'W=32, T=Q1, N1=D, P1=[K1], N2=C, P2=[], L=[K1]',
    // 'W=80, T=Q1, N1=C, P1=[], N2=B, P2=[], L=[K1]',
    // 'W=115, T=Q1, N1=B, P1=[], N2=A, P2=[K1], L=[]',
    // 'W=0, T=Q3, N1=F, P1=[], N2=E, P2=[], L=[]',
    // 'W=63, T=Q3, N1=E, P1=[K2], N2=F, P2=[K2], L=[]'

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

        // Configuration

        let mut graph = Graph::new();
        graph.init_key(&station_collection);
        graph.init_value(&edge_storage);

        let mut distance_map = DistanceMap::new();
        distance_map.init_key_value(graph.clone(), edge_storage.clone());

        let mut timeline = Timeline::new();
        for tr in train_collection.iter_mut() {
            timeline.insert(tr.name().to_string(), 0);
        }

        let mut train_movement = TrainMovement::new();

        let mut package_tracker = PackageTracker::new();

        for pkg in package_collection.iter() {
            package_tracker.add_package(pkg.clone(), PackageStatus::AwaitingPickup);
        }

        let mut package_name = PackageName::new();
        for pkg in package_collection.iter() {
            package_name.add_name(pkg.name().to_string());
        }

        let mut loggerize = Logger::new();

        // tracer!(&graph);
        println!();
        println!("Input: ");
        println!();
        println!("{}", &station_collection);
        println!("{}", &edge_storage.clone());
        println!("{}", &package_collection);
        println!("{}", &train_collection);
        println!();
        println!("Output: ");
        println!();

        start_searching(
            &mut package_collection,
            &mut train_collection,
            &station_collection,
            graph,
            &mut train_movement,
            &mut distance_map,
            &mut timeline,
            package_tracker,
            &mut loggerize,
        );
        println!();
    }

    // Input:

    // // example input
    // 3		// number of stations
    // A		// station name
    // B		// station name
    // C		// station name

    // 2		// number of edges
    // E1,A,B,30	// route from A to B that takes 30 minutes
    // E2,B,C,10	// route from B to C that takes 10 minutes

    // 1		// number of deliveries to be performed
    // K1,5,A,C	// package K1 with weight 5 located currently at station A that must be delivered to station C

    // 1		// number of trains
    // Q1,6,B		// train Q1 with capacity 6 located at station B

    // Output:

    // // Move Q1 to A via E1, takes 30 minutes.
    // W=0, T=Q1, N1=B, P1=[], N2=A, P2=[]
    // // Now move back to B. Takes 30 minutes.
    // W=30, T=Q1, N1=A, P1=[K1], N2=B, P2=[]
    // // Move to C and drop off - takes 10 minutes.
    // W=60, T=Q1, N1=B, P1=[], N2=C, P2=[K1]
    // // Takes 70 minutes total.

    // cargo test test_second -- --nocapture
    #[test]
    fn test_second() {
        let mut station_collection = StationCollection::new();
        station_collection.add_station("A".to_string());
        station_collection.add_station("B".to_string());
        station_collection.add_station("C".to_string());

        let mut edge_storage = EdgeStorage::new();
        edge_storage.push("E1".to_string(), "A".to_string(), "B".to_string(), 30);
        edge_storage.push("E2".to_string(), "B".to_string(), "C".to_string(), 10);

        let mut package_collection = PackageCollection::new();
        package_collection.add_package(
            "K1".to_string(),
            5,
            "A".to_string(),
            "C".to_string(),
            PackageStatus::AwaitingPickup,
        );

        let mut package_collection_2 = PackageCollection::new();
        package_collection_2.add_package(
            "K1".to_string(),
            5,
            "A".to_string(),
            "C".to_string(),
            PackageStatus::AwaitingPickup,
        );

        let mut train_collection = TrainCollection::new();
        train_collection.add_train(
            "Q1".to_string(),
            6,
            6,
            "B".to_string(),
            "B".to_string(),
            Vec::new(),
            0,
        );

        // Configuration

        let mut graph = Graph::new();
        graph.init_key(&station_collection);
        graph.init_value(&edge_storage);

        let mut distance_map = DistanceMap::new();
        distance_map.init_key_value(graph.clone(), edge_storage.clone());

        let mut timeline = Timeline::new();
        for tr in train_collection.iter_mut() {
            timeline.insert(tr.name().to_string(), 0);
        }

        let mut train_movement = TrainMovement::new();

        let mut package_tracker = PackageTracker::new();

        for pkg in package_collection.iter() {
            package_tracker.add_package(pkg.clone(), PackageStatus::AwaitingPickup);
        }

        let mut package_name = PackageName::new();
        for pkg in package_collection.iter() {
            package_name.add_name(pkg.name().to_string());
        }

        let mut loggerize = Logger::new();

        println!();
        println!("Input: ");
        println!();
        println!("{}", &station_collection);
        println!("{}", &edge_storage.clone());
        println!("{}", &package_collection);
        println!("{}", &train_collection);
        println!();
        println!("Output: ");
        println!();

        start_searching(
            &mut package_collection,
            &mut train_collection,
            &station_collection,
            graph,
            &mut train_movement,
            &mut distance_map,
            &mut timeline,
            package_tracker,
            &mut loggerize,
        );
        println!();
    }

    // Input:

    // stations: [ 'A', 'B', 'C', 'D' ],
    // edges: [ 'E1,A,B,48', 'E2,B,C,88', 'E3,C,D,3' ],
    // deliveries: [ 'K1,7,B,D', 'K2,20,B,A', 'K3,49,A,B', 'K4,7,B,C', 'K5,25,D,A' ],
    // trains: [ 'Q1,59,C', 'Q2,21,B', 'Q3,81,A' ]

    // Output:

    // 'W=0, T=Q2, N1=B, P1=[K1,K4], N2=C, P2=[K4], L=[K1]',
    // 'W=88, T=Q2, N1=C, P1=[], N2=D, P2=[K1], L=[]',
    // 'W=0, T=Q1, N1=C, P1=[], N2=B, P2=[], L=[]',
    // 'W=88, T=Q1, N1=B, P1=[K2], N2=A, P2=[K2], L=[]',
    // 'W=0, T=Q3, N1=A, P1=[K3], N2=B, P2=[K3], L=[]',
    // 'W=48, T=Q3, N1=B, P1=[], N2=C, P2=[], L=[]',
    // 'W=136, T=Q3, N1=C, P1=[], N2=D, P2=[], L=[]',
    // 'W=139, T=Q3, N1=D, P1=[K5], N2=C, P2=[], L=[K5]',
    // 'W=142, T=Q3, N1=C, P1=[], N2=B, P2=[], L=[K5]',
    // 'W=230, T=Q3, N1=B, P1=[], N2=A, P2=[K5], L=[]'

    // Not supported yet
    // cargo test test_third -- --nocapture
    #[test]
    fn test_third() {
        //
        let mut station_collection = StationCollection::new();
        station_collection.add_station("A".to_string());
        station_collection.add_station("B".to_string());
        station_collection.add_station("C".to_string());
        station_collection.add_station("D".to_string());

        let mut edge_storage = EdgeStorage::new();
        edge_storage.push("E1".to_string(), "A".to_string(), "B".to_string(), 48);
        edge_storage.push("E2".to_string(), "B".to_string(), "C".to_string(), 88);
        edge_storage.push("E3".to_string(), "C".to_string(), "D".to_string(), 3);

        let mut package_collection = PackageCollection::new();
        package_collection.add_package(
            "K1".to_string(),
            7,
            "B".to_string(),
            "D".to_string(),
            PackageStatus::AwaitingPickup,
        );
        package_collection.add_package(
            "K2".to_string(),
            20,
            "B".to_string(),
            "A".to_string(),
            PackageStatus::AwaitingPickup,
        );
        package_collection.add_package(
            "K3".to_string(),
            49,
            "A".to_string(),
            "B".to_string(),
            PackageStatus::AwaitingPickup,
        );
        package_collection.add_package(
            "K4".to_string(),
            7,
            "B".to_string(),
            "C".to_string(),
            PackageStatus::AwaitingPickup,
        );
        package_collection.add_package(
            "K5".to_string(),
            25,
            "D".to_string(),
            "A".to_string(),
            PackageStatus::AwaitingPickup,
        );

        // ---

        let mut package_collection_2 = PackageCollection::new();
        package_collection_2.add_package(
            "K1".to_string(),
            7,
            "B".to_string(),
            "D".to_string(),
            PackageStatus::AwaitingPickup,
        );
        package_collection_2.add_package(
            "K2".to_string(),
            20,
            "B".to_string(),
            "A".to_string(),
            PackageStatus::AwaitingPickup,
        );
        package_collection_2.add_package(
            "K3".to_string(),
            49,
            "A".to_string(),
            "B".to_string(),
            PackageStatus::AwaitingPickup,
        );
        package_collection_2.add_package(
            "K4".to_string(),
            7,
            "B".to_string(),
            "C".to_string(),
            PackageStatus::AwaitingPickup,
        );
        package_collection_2.add_package(
            "K5".to_string(),
            25,
            "D".to_string(),
            "A".to_string(),
            PackageStatus::AwaitingPickup,
        );

        let mut train_collection = TrainCollection::new();
        train_collection.add_train(
            "Q1".to_string(),
            59,
            59,
            "C".to_string(),
            "C".to_string(),
            Vec::new(),
            0,
        );
        train_collection.add_train(
            "Q2".to_string(),
            21,
            21,
            "B".to_string(),
            "B".to_string(),
            Vec::new(),
            0,
        );
        train_collection.add_train(
            "Q3".to_string(),
            81,
            81,
            "A".to_string(),
            "A".to_string(),
            Vec::new(),
            0,
        );

        // Configuration

        let mut graph = Graph::new();
        graph.init_key(&station_collection);
        graph.init_value(&edge_storage);

        let mut distance_map = DistanceMap::new();
        distance_map.init_key_value(graph.clone(), edge_storage.clone());

        let mut timeline = Timeline::new();
        for tr in train_collection.iter_mut() {
            timeline.insert(tr.name().to_string(), 0);
        }

        let mut train_movement = TrainMovement::new();

        let mut package_tracker = PackageTracker::new();

        for pkg in package_collection.iter() {
            package_tracker.add_package(pkg.clone(), PackageStatus::AwaitingPickup);
        }

        let mut package_name = PackageName::new();
        for pkg in package_collection.iter() {
            package_name.add_name(pkg.name().to_string());
        }

        let mut loggerize = Logger::new();

        // tracer!(&graph);
        // tracer!(&train_collection);
        // tracer!(&station_collection);

        println!();
        println!("Input: ");
        println!();
        println!("{}", &station_collection);
        println!("{}", &edge_storage.clone());
        println!("{}", &package_collection);
        println!("{}", &train_collection);
        println!();
        println!("Output: ");
        println!();

        start_searching(
            &mut package_collection,
            &mut train_collection,
            &station_collection,
            graph,
            &mut train_movement,
            &mut distance_map,
            &mut timeline,
            package_tracker,
            &mut loggerize,
        );
        println!();
    }

    // cargo test test_fourth -- --nocapture
    #[test]
    fn test_fourth() {
        let mut station_collection = StationCollection::new();
        station_collection.add_station("A".to_string());
        station_collection.add_station("B".to_string());
        station_collection.add_station("C".to_string());
        station_collection.add_station("D".to_string());
        station_collection.add_station("E".to_string());
        station_collection.add_station("F".to_string());

        let mut edge_storage = EdgeStorage::new();
        edge_storage.push("E1".to_string(), "A".to_string(), "B".to_string(), 45);
        edge_storage.push("E2".to_string(), "B".to_string(), "C".to_string(), 29);
        edge_storage.push("E3".to_string(), "C".to_string(), "D".to_string(), 88);
        edge_storage.push("E4".to_string(), "D".to_string(), "E".to_string(), 89);
        edge_storage.push("E5".to_string(), "E".to_string(), "F".to_string(), 50);

        let mut package_collection = PackageCollection::new();
        package_collection.add_package(
            "K1".to_string(),
            60,
            "B".to_string(),
            "E".to_string(),
            PackageStatus::AwaitingPickup,
        );

        package_collection.add_package(
            "K2".to_string(),
            25,
            "B".to_string(),
            "D".to_string(),
            PackageStatus::AwaitingPickup,
        );

        package_collection.add_package(
            "K3".to_string(),
            9,
            "F".to_string(),
            "C".to_string(),
            PackageStatus::AwaitingPickup,
        );

        let mut train_collection = TrainCollection::new();
        train_collection.add_train(
            "Q1".to_string(),
            68,
            68,
            "C".to_string(),
            "C".to_string(),
            Vec::new(),
            0,
        );

        train_collection.add_train(
            "Q2".to_string(),
            69,
            69,
            "B".to_string(),
            "B".to_string(),
            Vec::new(),
            0,
        );

        train_collection.add_train(
            "Q3".to_string(),
            79,
            79,
            "C".to_string(),
            "C".to_string(),
            Vec::new(),
            0,
        );

        // Configuration

        let mut graph = Graph::new();
        graph.init_key(&station_collection);
        graph.init_value(&edge_storage);

        let mut distance_map = DistanceMap::new();
        distance_map.init_key_value(graph.clone(), edge_storage.clone());

        let mut timeline = Timeline::new();
        for tr in train_collection.iter_mut() {
            timeline.insert(tr.name().to_string(), 0);
        }

        let mut train_movement = TrainMovement::new();

        let mut package_tracker = PackageTracker::new();

        for pkg in package_collection.iter() {
            package_tracker.add_package(pkg.clone(), PackageStatus::AwaitingPickup);
        }

        let mut package_name = PackageName::new();
        for pkg in package_collection.iter() {
            package_name.add_name(pkg.name().to_string());
        }

        let mut loggerize = Logger::new();

        // tracer!(&graph);
        println!();
        println!("Input: ");
        println!();
        println!("{}", &station_collection);
        println!("{}", &edge_storage.clone());
        println!("{}", &package_collection);
        println!("{}", &train_collection);
        println!();
        println!("Output: ");
        println!();

        start_searching(
            &mut package_collection,
            &mut train_collection,
            &station_collection,
            graph,
            &mut train_movement,
            &mut distance_map,
            &mut timeline,
            package_tracker,
            &mut loggerize,
        );
        println!();
    }
}
