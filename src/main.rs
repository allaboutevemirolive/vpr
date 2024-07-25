use std::collections::HashMap;
use std::fmt;

fn main() {
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

    dbg!(&dist_map);

    let mut package_map = find_train_candidates(&packages, &trains);

    dbg!(&package_map);

    let mut train_movement = TrainMovement::new();

    // let mut tracker = Tracker::new();

    let mut logger = Logger::new();

    let mut pkg_cands = PackageCandidates::new();

    let mut num_track = 0;

    let mut where_to_go = "".to_string();
    let mut next_station = "".to_string();

    // while num_track != 3 {
    while !packages.is_empty() {
        let mut pkg = packages.first().cloned().unwrap();
        let train = package_map.get_train_mut(&pkg).unwrap();

        if *pkg_collection.get_status(&pkg.name()).unwrap() == PackageStatus::AwaitingPickup {
            // Pick the closest train
            // let train = package_map.get_train_mut(&pkg).unwrap();

            // Check if the package pickup is where train already is

            dbg!(&pkg.from());
            dbg!(&trains.find_by_idx(pkg.from()));

            // For each package check if train same location a package
            if trains.find_by_idx(pkg.from()) {
                println!(
                    "package {:?} is at the same location train {}",
                    pkg,
                    stations.get_station_name(pkg.from()).unwrap()
                );

                if train.already_load_package(pkg.clone()) {
                    continue;
                }

                train.load_package(pkg.clone());
                pkg_collection.update_status(&pkg.name(), PackageStatus::InTransit);
            } else {
                // Move the train
                next_station = get_next(
                    &train,
                    &pkg,
                    &stations,
                    &graph,
                    where_to_go.clone(),
                    &pkg_collection,
                );
                dbg!(&next_station);

                let train_index = stations
                    .get_station_idx(
                        stations
                            .get_station_name(train.current_index())
                            .unwrap()
                            .to_string(),
                    )
                    .unwrap();

                dbg!(&train_index);
                dbg!(&train.current_index());
                dbg!(&pkg.from());
                dbg!(&pkg.to());

                // let remaining_space: u32 = train.remain_capacity() - pkg.weight();

                // dbg!(&remaining_space);

                // We handle the case if
                // - train index != with pkg origin
                // - if train index != pkg destionation
                while train.current_index() != pkg.from() || train.current_index() != pkg.to() {
                    for (_, pkg_) in packages.clone().enumerate_packages() {
                        dbg!(&pkg_);
                        let loadable = train.remain_capacity() >= pkg_.weight();
                        dbg!(&loadable);

                        let pkg_status = pkg_collection.get_status(pkg_.name()).unwrap();
                        dbg!(&pkg_status);
                        let waiting = *pkg_status == PackageStatus::AwaitingPickup;

                        let same_location = pkg.from() == train.current_index();
                        dbg!(&same_location);

                        if loadable && waiting && same_location {
                            pkg_cands.push(&pkg_);
                        }
                    }

                    dbg!("{} with {}", &pkg_cands.len(), &pkg_cands);
                    if !pkg_cands.is_empty() {
                        for (_, candidate) in pkg_cands.enumerate_cands() {
                            if !train.already_load_package(candidate.clone()) {
                                // candidate.clone().set_status(PackageStatus::InTransit);
                                train.load_package(candidate.clone());
                                pkg_collection
                                    .update_status(&candidate.name(), PackageStatus::InTransit);
                                // pkg.set_status(PackageStatus::InTransit);
                                dbg!(&pkg_collection);
                            }
                        }
                    }

                    let mut pick_packages: Vec<Package> = Vec::new();

                    for pick_pkg in train.packages.clone() {
                        if pick_pkg.from() == train_index && !logger.already_add(&pick_pkg) {
                            pick_packages.push(pick_pkg);
                        }
                    }
                    dbg!(&pick_packages);
                    train_movement.with_picked_pkgs(pick_packages.clone());
                    dbg!(&train_movement);

                    for pick_p in pick_packages {
                        logger.push(&pick_p);
                    }
                    dbg!(&logger);

                    let mut drop_packages: Vec<Package> = Vec::new();

                    dbg!(&next_station);
                    dbg!(&stations.get_station_idx(next_station.clone()).unwrap());

                    let next_station_idx = &stations.get_station_idx(next_station.clone()).unwrap();

                    for loader in train.packages.clone() {
                        if loader.to() == *next_station_idx
                            && *loader.status() != PackageStatus::Dummy
                        {
                            drop_packages.push(loader);
                        }
                    }

                    dbg!(&drop_packages);

                    train_movement.with_drop_pkgs(drop_packages.clone());

                    dbg!(&train_movement);
                    dbg!(&train);

                    for drop_pkg in drop_packages {
                        if train.already_load_package(drop_pkg.clone()) {
                            continue;
                        }

                        // Remove from carriage
                        train.remove_package(drop_pkg.clone());
                        // Remove from packages
                        packages.pop(drop_pkg.clone());
                        // Remove from tracker
                        pkg_collection.update_status(&drop_pkg.name(), PackageStatus::Delivered);
                        dbg!(&pkg_collection);
                    }

                    train_movement.with_time(train.time());
                    train_movement.with_train(train.name().to_string());
                    train_movement.with_from(train_index);
                    train_movement.with_to(*next_station_idx);
                    for pkg_train in train.packages.clone() {
                        if *pkg_train.status() != PackageStatus::Dummy {
                            train_movement.with_carriages(train.packages.clone());
                        }
                    }

                    dbg!(&train_movement);
                    dbg!(&train);

                    dbg!(&stations
                        .get_station_name(train.current_index())
                        .unwrap()
                        .to_string());
                    dbg!(stations
                        .get_station_name(*next_station_idx)
                        .unwrap()
                        .to_string());

                    let dist = dist_map.get_distance(
                        stations.get_station_name(train_index).unwrap().to_string(),
                        stations
                            .get_station_name(*next_station_idx)
                            .unwrap()
                            .to_string(),
                    );
                    dbg!(&dist);

                    // let grp = graph

                    let numerize = dist.parse::<u32>().unwrap();
                    dbg!(&numerize);

                    dbg!(&train);
                    train.accumulate_time(numerize);
                    train.update_current_idx(*next_station_idx);
                    dbg!(&train);

                    // We arrived

                    next_station = get_next(
                        &train,
                        &pkg,
                        &stations,
                        &graph,
                        where_to_go.clone(),
                        &pkg_collection,
                    );
                    dbg!(&next_station);
                    break; // TODO
                }
            }
        }
        // dbg!("Hello World");
        if *pkg_collection.get_status(&pkg.name()).unwrap() == PackageStatus::InTransit {
            dbg!(&pkg_collection.get_status(&pkg.name()).unwrap());

            // ===================

            // Move the train
            next_station = get_next(
                &train,
                &pkg,
                &stations,
                &graph,
                where_to_go.clone(),
                &pkg_collection,
            );
            dbg!(&next_station);

            let train_index = stations
                .get_station_idx(
                    stations
                        .get_station_name(train.current_index())
                        .unwrap()
                        .to_string(),
                )
                .unwrap();

            dbg!(&train_index);
            dbg!(&train.current_index());
            dbg!(&pkg.from());
            dbg!(&pkg.to());

            let remaining_space: u32 = train.remain_capacity();

            dbg!(&remaining_space);

            // We handle the case if
            // - train index != with pkg origin
            // - if train index != pkg destionation
            // while train.current_index() != pkg.from() || train.current_index() != pkg.to() {
            while !(*pkg_collection.get_status(&pkg.name()).unwrap() == PackageStatus::Delivered) {
                for (_, pkg_) in packages.clone().enumerate_packages() {
                    dbg!(&pkg_);
                    let loadable = train.remain_capacity() >= pkg_.weight();
                    dbg!(&loadable);

                    let pkg_status = pkg_collection.get_status(pkg_.name()).unwrap();
                    dbg!(&pkg_status);
                    let waiting = *pkg_status == PackageStatus::AwaitingPickup;

                    let same_location = pkg.from() == train.current_index();
                    dbg!(&same_location);

                    if loadable && waiting && same_location {
                        pkg_cands.push(&pkg_);
                    }
                }

                dbg!("{} with {}", &pkg_cands.len(), &pkg_cands);
                if !pkg_cands.is_empty() {
                    for (_, candidate) in pkg_cands.enumerate_cands() {
                        if !train.already_load_package(candidate.clone()) {
                            // candidate.clone().set_status(PackageStatus::InTransit);
                            train.load_package(candidate.clone());
                            pkg_collection
                                .update_status(&candidate.name(), PackageStatus::InTransit);
                            // pkg.set_status(PackageStatus::InTransit);
                            dbg!(&pkg_collection);
                        }
                    }
                }

                let mut pick_packages: Vec<Package> = Vec::new();

                for pick_pkg in train.packages.clone() {
                    if pick_pkg.from() == train_index && !logger.already_add(&pick_pkg) {
                        pick_packages.push(pick_pkg);
                    }
                }
                dbg!(&pick_packages);
                train_movement.with_picked_pkgs(pick_packages.clone());
                dbg!(&train_movement);

                for pick_p in pick_packages {
                    logger.push(&pick_p);
                }
                dbg!(&logger);

                let mut drop_packages: Vec<Package> = Vec::new();

                dbg!(&next_station);
                dbg!(&stations.get_station_idx(next_station.clone()).unwrap());

                let next_station_idx = &stations.get_station_idx(next_station.clone()).unwrap();

                for loader in train.packages.clone() {
                    if loader.to() == *next_station_idx && *loader.status() != PackageStatus::Dummy
                    {
                        drop_packages.push(loader);
                    }
                }

                dbg!(&drop_packages);

                train_movement.with_drop_pkgs(drop_packages.clone());

                dbg!(&train_movement);
                dbg!(&train);

                for drop_pkg in drop_packages {
                    // if train.already_load_package(drop_pkg.clone()) {
                    //     continue;
                    // }

                    // Remove from carriage
                    train.remove_package(drop_pkg.clone());
                    // Remove from packages
                    packages.pop(drop_pkg.clone());
                    // Remove from tracker
                    pkg_collection.update_status(&drop_pkg.name(), PackageStatus::Delivered);
                    dbg!(&pkg_collection);
                    dbg!(&packages);
                }

                train_movement.with_time(train.time());
                train_movement.with_train(train.name().to_string());
                train_movement.with_from(train_index);
                train_movement.with_to(*next_station_idx);
                for pkg_train in train.packages.clone() {
                    if *pkg_train.status() != PackageStatus::Dummy {
                        train_movement.with_carriages(train.packages.clone());
                    }
                }

                dbg!(&train_movement);
                dbg!(&train);

                dbg!(&stations
                    .get_station_name(train.current_index())
                    .unwrap()
                    .to_string());
                dbg!(stations
                    .get_station_name(*next_station_idx)
                    .unwrap()
                    .to_string());

                let dist = dist_map.get_distance(
                    stations.get_station_name(train_index).unwrap().to_string(),
                    stations
                        .get_station_name(*next_station_idx)
                        .unwrap()
                        .to_string(),
                );
                dbg!(&dist);

                // let grp = graph

                let numerize = dist.parse::<u32>().unwrap();
                dbg!(&numerize);

                dbg!(&train);
                train.accumulate_time(numerize);
                train.update_current_idx(*next_station_idx);
                dbg!(&train);

                // We arrived

                next_station = get_next(
                    &train,
                    &pkg,
                    &stations,
                    &graph,
                    where_to_go.clone(),
                    &pkg_collection,
                );
                dbg!(&next_station);
                // break; // TODO
            }
        }

        // break;
        // num_track += 1;
    }
}

fn get_next<'a>(
    train: &Train,
    pkg: &Package,
    stations: &Stations,
    graph: &'a Graph,
    mut where_to_go: String,
    pkg_collection: &PackageCollection,
) -> String {
    let (next_station, alt_station) = graph
        .get_neighbors(stations.get_station_name(train.current_index()).unwrap())
        .unwrap()
        .split_first()
        .unwrap();

    let alt_station = alt_station.join("");

    let next_station_idx = stations.get_station_idx(next_station.to_string()).unwrap();
    let alt_station_idx = stations.get_station_idx(alt_station.to_string());

    let pkg_status = pkg_collection.get_status(&pkg.name()).unwrap();

    let train_pos_name = stations.get_station_name(train.current_index()).unwrap();

    let train_pos_idx = stations
        .get_station_idx(train_pos_name.to_string())
        .unwrap();

    let pkg_pos_name = stations.get_station_name(pkg.from()).unwrap();

    let pkg_pos_idx = stations.get_station_idx(pkg_pos_name.to_string()).unwrap();

    dbg!(&train_pos_name);
    dbg!(&train_pos_idx);
    dbg!(&pkg_pos_name);
    dbg!(&pkg_pos_idx);

    if *pkg_status == PackageStatus::AwaitingPickup {
        if next_station_idx == pkg.from() || alt_station.is_empty() {
            return next_station.to_string();
        }
    }

    if *pkg_status == PackageStatus::InTransit {
        if alt_station_idx.is_some() {
            if alt_station_idx.unwrap() == pkg.to() {
                return alt_station.to_string();
            }
        }
    }

    if pkg_pos_idx > train_pos_idx {
        return alt_station;
    }

    next_station.to_string()
}

/// We use this struct to track the route
/// First, we store the current train's index to `from`
/// Then we store package's location's index to `to`
///
/// After train reach to package's location index, we update this struct
///
/// We store package's location (same as curr train's location) to from
/// We store package's destination to `to`
#[derive(Debug)]
pub struct Tracker {
    from: u32,
    to: u32,
}

impl Tracker {
    pub fn new() -> Self {
        Self { from: 0, to: 0 }
    }

    pub fn set_from(&mut self, from_: u32) {
        self.from = from_
    }

    pub fn from(&self) -> u32 {
        self.from
    }

    pub fn set_to(&mut self, to_: u32) {
        self.to = to_
    }

    pub fn to(&self) -> u32 {
        self.to
    }

    pub fn subtract_from_by_1(&mut self) {
        self.from -= 1;
    }

    pub fn plus_from_by_1(&mut self) {
        self.from += 1
    }

    pub fn values_differ(&mut self) -> bool {
        self.from != self.to
    }

    pub fn which_direction(&self) -> Direction {
        if self.from == self.to {
            return Direction::Middle;
        }

        if self.from > self.to {
            Direction::Left
        } else {
            Direction::Right
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Status {
    Finish,
    Ongoing,
}

#[derive(PartialEq, Eq, Debug)]
pub enum Direction {
    // If `from` index bigger than `to` index, we go left
    Left,
    // If `from` index less than `to` index, we go right
    Right,
    // `from` index is same as `to` index
    Middle,
}

// fn get_next_direction(stations: &Stations, graph: &Graph) {
//     let current_station = stations.get_station_name(train.current_index()).unwrap();
//     let neighbors = graph.graph.get(current_station).unwrap();
// }

// fn get_next(
//     train: &Train,
//     to: &str,
//     graph: &Graph,
//     dist_map: &DistanceMap,
//     stations: &Stations,
// ) -> (Direction, String) {
//     let current_station = stations.get_station_name(train.current_index()).unwrap();
//     let neighbors = graph.graph.get(current_station).unwrap();

//     // Check Destination:
//     for neighbor in neighbors {
//         if neighbor == to {
//             if neighbors[0] == to {
//                 return (Direction::Left, neighbor.to_string());
//             } else {
//                 return (Direction::Right, neighbor.to_string());
//             }
//         }
//     }

//     // Handle Case of Only One Neighbor:
//     if neighbors.len() == 1 {
//         return (Direction::Left, neighbors[0].to_string()); // Choose the only available direction
//     }

//     // Compare Distances:
//     let left_distance = dist_map
//         .dm
//         .get(&(current_station.clone(), neighbors[0].clone()))
//         .copied() // Use copied() for Option<u32> to u32 if possible
//         .unwrap_or(u32::MAX); // Handle potential missing distance
//     let right_distance = dist_map
//         .dm
//         .get(&(current_station.clone(), neighbors[1].clone()))
//         .copied()
//         .unwrap_or(u32::MAX);

//     if left_distance < right_distance {
//         (Direction::Left, neighbors[0].to_string())
//     } else if left_distance > right_distance {
//         (Direction::Right, neighbors[1].to_string())
//     } else {
//         // If distances are equal, default to left
//         (Direction::Left, neighbors[0].to_string())
//     }
// }

// TODO: Write test
// Check if capacity is not enough.
// Then check if candidates_train and current_train iteration are close to current package iteration.
// If both trains (candidates and current) have the same distance to reach the current package, check which train has less time to reach the package iteration.
// Train with lesser time to reach the package and has the shortest distance to package will be store in HashMap
// The overall answer will be stored in Hashmap.
fn find_train_candidates(packages: &Packages, trains: &Trains) -> PackageTrainMapping {
    let mut package_train_map = PackageTrainMapping::new();
    let mut candidate_train: Option<Train> = None;
    let mut min_distance: u32 = u32::MAX;

    for (_, pkg) in packages.clone().enumerate_packages() {
        if *pkg.status() == PackageStatus::AwaitingPickup {
            for (_, train) in trains.enumerate_trains() {
                if train.capacity() < pkg.weight() {
                    continue;
                }

                let diff = get_diff(pkg.from(), train.origin());

                if diff < min_distance
                    || (diff == min_distance
                        && train.time
                            < candidate_train.as_ref().map(|t| t.time).unwrap_or(u32::MAX))
                {
                    candidate_train = Some(train.clone());
                    min_distance = diff;
                }
            }
        }

        if let Some(train) = candidate_train {
            package_train_map.assign_package(pkg, train);
        }

        candidate_train = None;
        min_distance = u32::MAX;
    }

    package_train_map
}

fn get_diff(a: u32, b: u32) -> u32 {
    if a > b {
        a - b
    } else {
        b - a
    }
}

#[derive(Debug, Clone)]
struct PackageTrainMapping {
    mapping: HashMap<Package, Train>,
}

impl PackageTrainMapping {
    fn new() -> Self {
        Self {
            mapping: HashMap::new(),
        }
    }

    fn assign_package(&mut self, package: Package, train: Train) {
        self.mapping.insert(package, train);
    }

    fn get_train_for_package(&self, package_name: &Package) -> Option<&Train> {
        self.mapping.get(package_name)
    }

    fn get_train_mut(&mut self, package: &Package) -> Option<&mut Train> {
        self.mapping.get_mut(package)
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Station {
    stat: String,
}

impl Station {
    pub fn new(stat: String) -> Self {
        Self { stat }
    }
}

/// All stations
#[derive(Clone)]
pub struct Stations {
    pub stations: Vec<Station>,
}

impl Stations {
    pub fn new() -> Self {
        Self {
            stations: Vec::new(),
        }
    }

    pub fn push(&mut self, station: Station) {
        self.stations.push(station);
    }

    pub fn sort(&mut self) {
        self.stations.sort()
    }

    pub fn get_station_name(&self, idx: u32) -> Option<&String> {
        for (idx_stat, stat) in self.enumerate_stations() {
            if idx_stat == idx as usize {
                return Some(&stat.stat);
            }
        }
        None
    }

    pub fn get_station_idx(&self, name: String) -> Option<u32> {
        for (idx_stat, stat) in self.enumerate_stations() {
            if stat.stat == name {
                return Some(idx_stat.try_into().unwrap());
            }
        }
        None
    }

    fn enumerate_stations(&self) -> impl Iterator<Item = (usize, &Station)> {
        self.stations.iter().enumerate()
    }
}

/// A route which train can use to delivering package
#[derive(Debug, Clone)]
pub struct Edge {
    name: String,
    from: u32,
    to: u32,
    /// Times taken from `from_station` to `to_station`
    times: u32,
}

impl Edge {
    pub fn new(name: String, from: u32, to: u32, times: u32) -> Self {
        Self {
            name,
            from,
            to,
            times,
        }
    }

    pub fn times(&self) -> u32 {
        self.times
    }

    pub fn from(&self) -> u32 {
        self.from
    }

    pub fn to(&self) -> u32 {
        self.to
    }

    pub fn name(&self) -> &String {
        &self.name
    }
}

/// All posible routes exist for trains to delivered various packages
#[derive(Clone)]
pub struct Edges {
    edges: Vec<Edge>,
}

impl Edges {
    pub fn new() -> Self {
        Edges { edges: Vec::new() }
    }

    pub fn push(&mut self, name: String, from: u32, to: u32, times: u32) {
        let edge = Edge::new(name, from, to, times);

        self.edges.push(edge);
    }

    fn enumerate_edges(&self) -> impl Iterator<Item = (usize, &Edge)> {
        self.edges.iter().enumerate()
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum PackageStatus {
    AwaitingPickup,
    InTransit,
    Delivered,
    Dummy,
}

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub struct Package {
    name: String,
    weight: u32,
    // starting_station: String,
    from: u32,
    // destination_station: String,
    to: u32,
    status: PackageStatus,
}

impl Default for Package {
    fn default() -> Self {
        Package {
            name: String::from("Unknown"),
            weight: 0,
            from: 0,
            to: 0,
            status: PackageStatus::Dummy,
        }
    }
}

impl Package {
    pub fn new(name: String, weight: u32, from: u32, to: u32, status: PackageStatus) -> Self {
        Self {
            name,
            weight,
            from,
            to,
            status,
        }
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn weight(&self) -> u32 {
        self.weight
    }

    pub fn from(&self) -> u32 {
        self.from
    }

    pub fn to(&self) -> u32 {
        self.to
    }

    pub fn status(&self) -> &PackageStatus {
        &self.status
    }

    pub fn set_status(&mut self, package_status: PackageStatus) {
        self.status = package_status
    }
}

/// All packages
#[derive(Debug, Clone)]
pub struct Packages {
    packages: Vec<Package>,
}

impl Packages {
    pub fn new() -> Self {
        Self {
            packages: Vec::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.packages.len()
    }

    pub fn push(&mut self, package: Package) {
        self.packages.push(package)
    }

    pub fn pop(&mut self, package: Package) {
        let mut index = 0;
        let mut found = false;
        for (idx, pkg) in self.clone().enumerate_packages() {
            if package.name() == pkg.name() {
                index = idx;
                found = true;
                break;
            }
        }

        if found {
            self.packages.remove(index);
        }
    }

    pub fn is_empty(&self) -> bool {
        self.packages.is_empty()
    }

    pub fn first(&self) -> Option<&Package> {
        self.packages.first()
    }

    pub fn push_with(
        &mut self,
        name: String,
        weight: u32,
        from: u32,
        to: u32,
        status: PackageStatus,
    ) {
        let package = Package::new(name, weight, from, to, status);

        self.packages.push(package);
    }

    fn enumerate_packages(self) -> impl Iterator<Item = (usize, Package)> {
        self.packages.into_iter().enumerate()
    }
}

#[derive(Debug, Clone)]
pub struct Train {
    name: String,
    /// Maximum weight, a train can carry packages. A train can carry `MORE` than 1 packages if
    /// those package doesn't exceed this capacity
    capacity: u32,

    /// Remaining capacity after load package to carriage
    remain_capacity: u32,

    /// Origin index
    origin: u32,

    // TODO: At some point, we need to update this current index. Where and When?
    /// Current index
    current: u32,

    /// Current packages in carriage
    packages: Vec<Package>,

    /// Time taken for a train to delivering packages
    time: u32,
}

impl Train {
    pub fn new(
        name: String,
        capacity: u32,
        remain_capacity: u32,
        origin: u32,
        current: u32,
        packages: Vec<Package>,
        time: u32,
    ) -> Self {
        Self {
            name,
            capacity,
            remain_capacity,
            origin,
            current,
            packages,
            time,
        }
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn capacity(&self) -> u32 {
        self.capacity
    }

    pub fn remain_capacity(&self) -> u32 {
        self.remain_capacity
    }

    pub fn push_packages(&mut self, package: Package) {
        self.packages.push(package)
    }

    pub fn origin(&self) -> u32 {
        self.origin
    }

    pub fn current_index(&self) -> u32 {
        self.current
    }
    pub fn time(&self) -> u32 {
        self.time
    }

    pub fn accumulate_time(&mut self, time_taken: u32) {
        self.time += time_taken
    }

    pub fn already_load_package(&self, package: Package) -> bool {
        for pkg in self.packages.clone() {
            // We compare name instead of the whole Package struct to avoid ambiguity
            if pkg.name() == package.name() {
                return true;
            }
        }
        false
    }

    /// Load package to carriage while take into account `package weight` and train `remaining capcity`
    pub fn load_package(&mut self, pkg: Package) {
        let pkg_weight = pkg.weight();

        if pkg_weight > self.remain_capacity {
            return;
        }

        self.packages.push(pkg);
        self.remain_capacity -= pkg_weight;
    }

    pub fn remove_package(&mut self, pkg: Package) {
        let mut rm_idx: usize = 0;
        for (index, package) in self.packages.clone().iter().enumerate() {
            if package.name() == pkg.name() {
                rm_idx = index;
                break;
            }
        }

        self.packages.remove(rm_idx);
    }

    pub fn update_current_idx(&mut self, index: u32) {
        self.current = index
    }
}

/// All trains
pub struct Trains {
    trains: Vec<Train>,
}

impl Trains {
    pub fn new() -> Self {
        Self { trains: Vec::new() }
    }

    pub fn push_with(
        &mut self,
        name: String,
        capacity: u32,
        origin: u32,
        current: u32,
        package: Package,
        time: u32,
    ) {
        let mut pkgs = Vec::new();
        pkgs.push(package);

        let mut pkg_weight: u32 = 0;

        for pkg in pkgs.clone() {
            pkg_weight += pkg.weight();
        }

        let remain_capacity = capacity - pkg_weight;

        let train = Train::new(name, capacity, remain_capacity, origin, current, pkgs, time);
        self.trains.push(train);
    }

    pub fn enumerate_trains(&self) -> impl Iterator<Item = (usize, &Train)> {
        self.trains.iter().enumerate()
    }

    /// Find train's index based on package's index
    pub fn find_by_idx(&self, index: u32) -> bool {
        for (_, train) in self.enumerate_trains() {
            if train.current_index() == index {
                return true;
            }
        }
        false
    }

    // pub fn get_name_by_idx(&self, index:u32) -> String {

    // }
}

#[derive(Debug, Clone)]
pub struct PackageCollection {
    ps: HashMap<String, PackageStatus>,
}

impl PackageCollection {
    pub fn new() -> Self {
        Self { ps: HashMap::new() }
    }

    pub fn add_package(&mut self, package_id: String, status: PackageStatus) {
        self.ps.insert(package_id, status);
    }

    pub fn update_status(&mut self, package_id: &str, new_status: PackageStatus) {
        if let Some(status) = self.ps.get_mut(package_id) {
            *status = new_status;
        } else {
            eprintln!("Package not found: {}", package_id);
        }
    }

    pub fn get_status(&self, package_id: &str) -> Option<&PackageStatus> {
        self.ps.get(package_id)
    }

    pub fn remove_package(&mut self, package_id: &str) -> Option<PackageStatus> {
        self.ps.remove(package_id)
    }
}

#[derive(Debug, Clone)]
pub struct Graph {
    graph: HashMap<String, Vec<String>>,
}

impl Graph {
    pub fn new() -> Self {
        Self {
            graph: HashMap::new(),
        }
    }

    pub fn init_key(&mut self, stations: Stations) {
        for (_, station) in stations.enumerate_stations() {
            self.graph.insert(station.stat.clone(), Vec::new());
        }
    }

    pub fn init_value(&mut self, edges: Edges, stations: Stations) {
        for (_, edge) in edges.enumerate_edges() {
            let start = stations.get_station_name(edge.from()).unwrap();
            let end = stations.get_station_name(edge.to()).unwrap();

            self.graph.get_mut(start).unwrap().push(end.clone());
            self.graph.get_mut(end).unwrap().push(start.clone());
        }
    }

    /// Get train alternative stations
    pub fn get_neighbors(&self, node: &str) -> Option<&Vec<String>> {
        self.graph.get(node)
    }
}

#[derive(Debug, Clone)]
pub struct DistanceMap {
    dm: HashMap<(String, String), u32>,
}

impl DistanceMap {
    pub fn new() -> Self {
        Self { dm: HashMap::new() }
    }

    pub fn init_key_value(&mut self, graph: Graph, edges: Edges, stations: Stations) {
        for (start_station, neighbors) in graph.graph.iter() {
            for end_station in neighbors {
                let key = (start_station.clone(), end_station.clone());

                let distance = edges
                    .enumerate_edges()
                    .find(|(_, edge)| {
                        (stations
                            .get_station_name(edge.from())
                            .unwrap()
                            .starts_with(start_station)
                            && stations
                                .get_station_name(edge.to())
                                .unwrap()
                                .starts_with(end_station))
                            || (stations
                                .get_station_name(edge.from())
                                .unwrap()
                                .starts_with(end_station)
                                && stations
                                    .get_station_name(edge.to())
                                    .unwrap()
                                    .starts_with(start_station))
                    })
                    .and_then(|(_, edge)| Some(edge.times()))
                    .unwrap_or_else(|| 0);

                self.dm.insert(key, distance);
            }
        }
    }

    pub fn sorted_entries(&self) -> Vec<((String, String), u32)> {
        let mut entries: Vec<_> = self
            .dm
            .iter()
            .map(|(&(ref a, ref b), &distance)| ((a.clone(), b.clone()), distance))
            .collect();

        entries.sort_by_key(|&((ref a, ref b), _)| (a.clone(), b.clone()));
        entries
    }

    /// Get time taken between 2 nodes
    pub fn get_distance(&self, start: String, end: String) -> String {
        self.dm
            .get(&(start.to_string(), end.to_string()))
            .or_else(|| self.dm.get(&(end.to_string(), start.to_string())))
            .copied()
            .unwrap_or(0)
            .to_string()
    }
}

#[derive(Debug, Clone)]
pub struct TrainMovement {
    time: u32,
    train: String,
    from: u32,
    picked_pkgs: Vec<Package>,
    to: u32,
    /// Resembeling `Load` and `Drop`
    drop_pkgs: Vec<Package>,
    carriages: Vec<Package>,
}

// impl fmt::Display for TrainMovement {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         write!(f, "W={}", self.time)?;
//         write!(f, ", ")?;
//         write!(f, "T={}", self.train)?;
//         write!(f, ", ")?;
//         write!(f, "N1={}", self.from)?;
//         write!(f, ", ")?;
//         write!(f, "P1=[{:?}]", self.packages)?;
//         write!(f, ", ")?;
//         write!(f, "N2={}", self.to)?;
//         // write!(f, ", ")?;
//         // write!(f, "P2=[{}]", self.dropped_off.join(" "))?;
//         // TODO: Add `in_carriage`
//         Ok(())
//     }
// }

impl TrainMovement {
    pub fn new() -> Self {
        Self {
            time: 0,
            train: "".to_string(),
            from: 0,
            picked_pkgs: Vec::new(),
            to: 0,
            drop_pkgs: Vec::new(),
            carriages: Vec::new(),
        }
    }

    pub fn with_time(&mut self, time: u32) {
        self.time = time
    }

    pub fn with_train(&mut self, train_name: String) {
        self.train = train_name
    }

    pub fn with_from(&mut self, from_index: u32) {
        self.from = from_index
    }

    pub fn with_to(&mut self, to_index: u32) {
        self.to = to_index
    }

    pub fn with_picked_pkgs(&mut self, picked: Vec<Package>) {
        self.picked_pkgs = picked
    }

    pub fn with_drop_pkgs(&mut self, drop: Vec<Package>) {
        self.drop_pkgs = drop
    }

    pub fn with_carriages(&mut self, carr: Vec<Package>) {
        self.carriages = carr
    }

    // pub fn drop_package(&mut self, station_idx: usize) {
    //     for pkg in &self.packages {
    //         if pkg.to() == station_idx as u32 {

    //         }
    //     }
    // }
}

#[derive(Debug)]
pub struct Logger {
    log: Vec<Package>,
}

impl Logger {
    pub fn new() -> Self {
        Self { log: Vec::new() }
    }

    pub fn already_add(&self, pkg: &Package) -> bool {
        for pkg_ in self.log.clone() {
            if pkg_.name() == pkg.name() {
                return true;
            }
        }
        false
    }

    pub fn push(&mut self, pkg: &Package) {
        self.log.push(pkg.clone())
    }
}

#[derive(Debug, Clone)]
pub struct PackageCandidates {
    cand: Vec<Package>,
}

impl PackageCandidates {
    pub fn new() -> Self {
        Self { cand: Vec::new() }
    }

    pub fn push(&mut self, pkg: &Package) {
        self.cand.push(pkg.clone())
    }

    pub fn is_empty(&self) -> bool {
        self.cand.is_empty()
    }

    pub fn len(&self) -> usize {
        self.cand.len()
    }

    pub fn enumerate_cands(&self) -> impl Iterator<Item = (usize, &Package)> {
        self.cand.iter().enumerate()
    }
}

// impl TrainMove {
//     pub fn add_time(&mut self, time: u32) {
//         self.time = time;
//     }

//     pub fn add_train_id(&mut self, train_id: String) {
//         self.train_id = train_id
//     }

//     pub fn add_start_node(&mut self, start_node: String) {
//         self.start_node = start_node
//     }

//     pub fn push_picked_up(&mut self, package: String) {
//         self.picked_up.push(package.clone());
//         // Later, we pop the packages from this field.
//         // Logically, we only need a struct's field to monitor picked packages and dropped packages,
//         // so, late, this `dropped_off` field will be remove.
//         self.dropped_off.push(package);
//     }

//     pub fn end_node(&mut self, end_node: String) {
//         self.end_node = end_node
//     }

//     pub fn dropped_off(&self) {
//         todo!("redundant")
//     }

//     // TODO: Weird
//     pub fn in_carriage(&self) -> &Vec<String> {
//         &self.in_carriage
//     }
// }

// impl Default for TrainMove {
//     fn default() -> Self {
//         TrainMove {
//             time: 0,
//             train_id: String::new(),
//             start_node: String::new(),
//             picked_up: Vec::new(),
//             end_node: String::new(),
//             dropped_off: Vec::new(),
//             in_carriage: Vec::new(),
//         }
//     }
// }

// impl fmt::Display for TrainMovement {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         write!(f, "W={}", self.time)?;
//         write!(f, ", ")?;
//         write!(f, "T={}", self.train_id)?;
//         write!(f, ", ")?;
//         write!(f, "N1={}", self.start_node)?;
//         write!(f, ", ")?;
//         write!(f, "P1=[{}]", self.picked_up.join(" "))?;
//         write!(f, ", ")?;
//         write!(f, "N2={}", self.end_node)?;
//         write!(f, ", ")?;
//         write!(f, "P2=[{}]", self.dropped_off.join(" "))?;
//         // TODO: Add `in_carriage`
//         Ok(())
//     }
// }
