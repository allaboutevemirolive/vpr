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

    let mut tracker = Tracker::new();
    let mut status = Status::Ongoing;

    // while !packages.empty() {
    for (_, mut pkg) in packages.clone().enumerate_packages() {
        if *pkg.status() == PackageStatus::AwaitingPickup {
            if let Some(train) = package_map.get_train_mut(&pkg) {
                if pkg.from() == train.origin() {
                    if train.already_load_package(pkg.clone()) {
                        continue;
                    }
                    // load package
                    pkg.set_status(PackageStatus::InTransit);
                    train.load_package(pkg.clone());
                } else {
                    train_movement.with_train(train.name().to_string());

                    tracker.set_from(train.current_index());
                    tracker.set_to(pkg.from());

                    // move to the package index
                    // If:
                    // - The train has enough capacity to carry the package.
                    // - The package is at the pickup location.
                    // - The package is currently at the same station as the train.
                    // Then: loaded the package onto the train
                    dbg!(&pkg.from());
                    dbg!(&pkg.to());
                    dbg!(&train.current_index());
                    // Check if we can pick packages along the road
                    while !packages.empty() {
                        // dbg!(&pkg_train.to());
                        dbg!(&tracker.to());
                        let which_direction = tracker.which_direction();
                        dbg!(&which_direction);

                        if which_direction == Direction::Left {
                            let prev_index = tracker.from();

                            train_movement.with_from(prev_index);
                            tracker.subtract_from_by_1();
                            train_movement.with_to(tracker.from());

                            // Check if there any package in current index
                            for (idx, package) in packages.clone().enumerate_packages() {
                                if package.from() == tracker.from() {
                                    train.load_package(package);
                                }
                            }
                            let time_dist = dist_map.get_distance(
                                stations.get_station_name(prev_index).unwrap().to_string(),
                                stations
                                    .get_station_name(tracker.from())
                                    .unwrap()
                                    .to_string(),
                            );

                            train_movement.with_time(train.time());
                            train.accumulate_time(time_dist.parse::<u32>().unwrap_or_default());
                        } else if which_direction == Direction::Right {
                            let prev_index = tracker.from();

                            train_movement.with_from(prev_index);

                            // Check if there is any package in current transit
                            for (idx, package) in packages.clone().enumerate_packages() {
                                if package.from() == tracker.from() {
                                    if *pkg_collection.get_status(&package.name()).unwrap()
                                        == PackageStatus::AwaitingPickup
                                    {
                                        train.load_package(package.clone());
                                        pkg_collection.update_status(
                                            &package.name(),
                                            PackageStatus::InTransit,
                                        );
                                        tracker.set_from(package.from());
                                        tracker.set_to(package.to());
                                    }
                                }
                            }

                            dbg!(&train);

                            // Check if pkg in carriage arrived at destination
                            for pkg_train in train.packages.clone() {
                                // if pkg_train.name() == "Unknown" {
                                //     continue;
                                // }
                                if tracker.from() == pkg_train.to() {
                                    dbg!(&pkg_train.to());
                                    // dbg!(&tracker.to());
                                    dbg!(&pkg_train.from());
                                    dbg!(&tracker.to());
                                    dbg!(&tracker.from());
                                    dbg!(&pkg_train.name());
                                    if *pkg_collection.get_status(&pkg_train.name()).unwrap()
                                        == PackageStatus::InTransit
                                    {
                                        train.remove_package(pkg_train.clone());
                                        pkg_collection.update_status(
                                            &pkg_train.name(),
                                            PackageStatus::Delivered,
                                        );
                                        packages.pop(pkg.clone());
                                    }
                                }
                            }

                            tracker.plus_from_by_1();
                            train_movement.with_to(tracker.from());

                            let time_dist = dist_map.get_distance(
                                stations.get_station_name(prev_index).unwrap().to_string(),
                                stations
                                    .get_station_name(tracker.from())
                                    .unwrap()
                                    .to_string(),
                            );

                            train_movement.with_time(train.time());
                            train.accumulate_time(time_dist.parse::<u32>().unwrap_or_default());

                            // packages.pop(pkg.clone());
                            // dbg!(&train);
                            // dbg!(&train_movement);
                        } else {
                            // break;
                            // packages.pop(pkg.clone());
                            dbg!("Hello World");
                        }

                        // dbg!(&train_movement);

                        // Update
                        tracker.set_from(pkg.from());
                        tracker.set_to(pkg.to());

                        // dbg!(&tracker);
                    }
                }
            }
        }

        packages.pop(pkg);
    }
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

fn get_next(
    train: &Train,
    to: &str,
    graph: &Graph,
    dist_map: &DistanceMap,
    stations: &Stations,
) -> (Direction, String) {
    let current_station = stations.get_station_name(train.current_index()).unwrap();
    let neighbors = graph.graph.get(current_station).unwrap();

    // Check Destination:
    for neighbor in neighbors {
        if neighbor == to {
            if neighbors[0] == to {
                return (Direction::Left, neighbor.to_string());
            } else {
                return (Direction::Right, neighbor.to_string());
            }
        }
    }

    // Handle Case of Only One Neighbor:
    if neighbors.len() == 1 {
        return (Direction::Left, neighbors[0].to_string()); // Choose the only available direction
    }

    // Compare Distances:
    let left_distance = dist_map
        .dm
        .get(&(current_station.clone(), neighbors[0].clone()))
        .copied() // Use copied() for Option<u32> to u32 if possible
        .unwrap_or(u32::MAX); // Handle potential missing distance
    let right_distance = dist_map
        .dm
        .get(&(current_station.clone(), neighbors[1].clone()))
        .copied()
        .unwrap_or(u32::MAX);

    if left_distance < right_distance {
        (Direction::Left, neighbors[0].to_string())
    } else if left_distance > right_distance {
        (Direction::Right, neighbors[1].to_string())
    } else {
        // If distances are equal, default to left
        (Direction::Left, neighbors[0].to_string())
    }
}

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

    pub fn empty(&self) -> bool {
        self.packages.is_empty()
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
    to: u32,
    /// Resembeling `Load` and `Drop`
    packages: Vec<Package>,
}

impl fmt::Display for TrainMovement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "W={}", self.time)?;
        write!(f, ", ")?;
        write!(f, "T={}", self.train)?;
        write!(f, ", ")?;
        write!(f, "N1={}", self.from)?;
        write!(f, ", ")?;
        write!(f, "P1=[{:?}]", self.packages)?;
        write!(f, ", ")?;
        write!(f, "N2={}", self.to)?;
        // write!(f, ", ")?;
        // write!(f, "P2=[{}]", self.dropped_off.join(" "))?;
        // TODO: Add `in_carriage`
        Ok(())
    }
}

impl TrainMovement {
    pub fn new() -> Self {
        Self {
            time: 0,
            train: "".to_string(),
            from: 0,
            to: 0,
            packages: Vec::new(),
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

    pub fn load_package(&mut self, package: Package) {
        self.packages.push(package)
    }

    // pub fn drop_package(&mut self, station_idx: usize) {
    //     for pkg in &self.packages {
    //         if pkg.to() == station_idx as u32 {

    //         }
    //     }
    // }
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
