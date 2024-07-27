mod some;
mod test;
mod test_unit;
mod utils;

use std::collections::{BTreeMap, HashMap};

fn main() {
    println!("Hello, world!");
}

pub fn start_searching(
    mut pkg_c: PackageCollection,
    tr_c: TrainCollection,
    stat_c: StationCollection,
    gr: Graph,
    cands: PackageCandidates,
    tr_m: TrainMovement,
    dist_m: DistanceMap,
    tl: Timeline,
) {
    //
    for pkg in pkg_c.iter_mut() {
        while *pkg.status() != PackageStatus::Delivered {
            //
            if *pkg.status() == PackageStatus::AwaitingPickup {
                //
            }

            if *pkg.status() == PackageStatus::InTransit {
                //
            }
        }
    }
}

pub fn move_train() {}

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub enum PackageStatus {
    InTransit,
    Delivered,
    Pending,
    AwaitingPickup,
    Dummy,
}

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub struct Package {
    name: String,
    weight: u32,
    from: String,
    to: String,
    status: PackageStatus,
}

impl Default for Package {
    fn default() -> Self {
        Package {
            name: String::from("Unknown"),
            weight: 0,
            from: String::from("Unknown"),
            to: String::from("Unknown"),
            status: PackageStatus::Dummy,
        }
    }
}

impl Package {
    pub fn new(name: String, weight: u32, from: String, to: String, status: PackageStatus) -> Self {
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

    pub fn from(&self) -> &String {
        &self.from
    }

    pub fn to(&self) -> &String {
        &self.to
    }

    pub fn status(&self) -> &PackageStatus {
        &self.status
    }

    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    pub fn set_weight(&mut self, weight: u32) {
        self.weight = weight;
    }

    pub fn set_from(&mut self, from: String) {
        self.from = from;
    }

    pub fn set_to(&mut self, to: String) {
        self.to = to;
    }

    pub fn set_status(&mut self, status: PackageStatus) {
        self.status = status;
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Train {
    name: String,
    capacity: u32,
    remain_capacity: u32,
    origin: String,
    current: String,
    packages: Vec<Package>,
    time: u32,
}

impl Train {
    pub fn new(
        name: String,
        capacity: u32,
        remain_capacity: u32,
        origin: String,
        current: String,
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

    /// Get the name of the train.
    pub fn name(&self) -> &String {
        &self.name
    }

    /// Get the capacity of the train.
    pub fn capacity(&self) -> u32 {
        self.capacity
    }

    /// Get the remaining capacity of the train.
    pub fn remain_capacity(&self) -> u32 {
        self.remain_capacity
    }

    /// Add a package to the train.
    pub fn push_package(&mut self, package: Package) {
        self.packages.push(package);
    }

    /// Get the origin of the train.
    pub fn origin(&self) -> &str {
        &self.origin
    }

    /// Get the current index of the train.
    pub fn current_index(&self) -> &str {
        &self.current
    }

    /// Get the current time of the train.
    pub fn time(&self) -> u32 {
        self.time
    }

    /// Accumulate time to the train's time.
    pub fn accumulate_time(&mut self, time_taken: u32) {
        self.time += time_taken;
    }

    /// Check if a package is already loaded on the train.
    pub fn already_loaded_package(&self, package: &Package) -> bool {
        self.packages.iter().any(|pkg| pkg.name() == package.name())
    }

    /// Load a package onto the train.
    pub fn load_package(&mut self, pkg: Package) {
        let pkg_weight = pkg.weight();

        if pkg_weight <= self.remain_capacity {
            self.packages.push(pkg);
            self.remain_capacity -= pkg_weight;
        }
    }

    /// Remove a package from the train.
    pub fn remove_package(&mut self, package: &Package) {
        self.packages.retain(|pkg| pkg.name() != package.name());
    }

    /// Update the current index of the train.
    pub fn update_current_index(&mut self, index: String) {
        self.current = index;
    }

    /// Update the name of the train.
    pub fn update_name(&mut self, name: String) {
        self.name = name;
    }
}

#[derive(Debug)]
pub struct TrainCollection {
    trains: Vec<Train>,
}

impl TrainCollection {
    pub fn new() -> Self {
        Self { trains: Vec::new() }
    }

    pub fn add_train(
        &mut self,
        name: String,
        capacity: u32,
        remain_capacity: u32,
        origin: String,
        current: String,
        packages: Vec<Package>,
        time: u32,
    ) {
        let train = Train::new(
            name,
            capacity,
            remain_capacity,
            origin,
            current,
            packages,
            time,
        );
        self.trains.push(train);
    }

    pub fn get_train(&self, index: usize) -> Option<&Train> {
        self.trains.get(index)
    }

    pub fn get_train_mut(&mut self, index: usize) -> Option<&mut Train> {
        self.trains.get_mut(index)
    }

    // Mutable iterator for modifying the trains
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Train> {
        self.trains.iter_mut()
    }
}

#[derive(Debug)]
pub struct PackageCollection {
    packages: Vec<Package>,
}

impl PackageCollection {
    pub fn new() -> Self {
        Self {
            packages: Vec::new(),
        }
    }

    pub fn add_package(
        &mut self,
        name: String,
        weight: u32,
        from: String,
        to: String,
        status: PackageStatus,
    ) {
        let package = Package::new(name, weight, from, to, status);
        self.packages.push(package);
    }

    pub fn get_package(&self, index: usize) -> Option<&Package> {
        self.packages.get(index)
    }

    pub fn get_package_mut(&mut self, index: usize) -> Option<&mut Package> {
        self.packages.get_mut(index)
    }

    // Method to get an immutable iterator over the packages
    pub fn iter(&self) -> impl Iterator<Item = &Package> {
        self.packages.iter()
    }

    // Method to get a mutable iterator over the packages
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Package> {
        self.packages.iter_mut()
    }
}

#[derive(Debug, Clone)]
pub struct StationCollection {
    stations: BTreeMap<usize, String>,
}

impl StationCollection {
    pub fn new() -> Self {
        Self {
            stations: BTreeMap::new(),
        }
    }

    pub fn add_station(&mut self, name: String) {
        let index = self.stations.len(); // Zero-based index
        self.stations.insert(index, name);
    }

    pub fn get_station_name(&self, index: usize) -> Option<&String> {
        self.stations.get(&index)
    }

    pub fn get_station_index(&self, name: &str) -> Option<usize> {
        self.stations.iter().find_map(|(&idx, station_name)| {
            if station_name == name {
                Some(idx)
            } else {
                None
            }
        })
    }

    pub fn remove_station(&mut self, name: &str) {
        // First, find the index to remove
        let index_to_remove = self.stations.iter().find_map(|(&idx, station_name)| {
            if station_name == name {
                Some(idx)
            } else {
                None
            }
        });

        // Then, remove the station by the found index
        if let Some(index) = index_to_remove {
            self.stations.remove(&index);

            // Reassign indices to keep them consecutive
            let mut new_stations = BTreeMap::new();
            for (i, (_, station_name)) in self.stations.iter().enumerate() {
                new_stations.insert(i, station_name.clone());
            }
            self.stations = new_stations;
        }
    }

    pub fn list_stations(&self) {
        for (index, name) in &self.stations {
            println!("Index: {}, Station: {}", index, name);
        }
    }

    // Method to get an iterator over the station collection
    pub fn iter(&self) -> std::collections::btree_map::Iter<'_, usize, String> {
        self.stations.iter()
    }

    // Optional: Method to get an iterator over the station names only
    pub fn names(&self) -> impl Iterator<Item = &String> {
        self.stations.values()
    }
}

#[derive(Debug, Clone)]
pub struct Edge {
    name: String,
    from: String,
    to: String,
    times: u32,
}

impl Edge {
    pub fn new(name: String, from: String, to: String, times: u32) -> Self {
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

    pub fn from(&self) -> &String {
        &self.from
    }

    pub fn to(&self) -> &String {
        &self.to
    }

    pub fn name(&self) -> &String {
        &self.name
    }
}

#[derive(Clone)]
pub struct EdgeStorage {
    edges: Vec<Edge>,
}

impl EdgeStorage {
    pub fn new() -> Self {
        Self { edges: Vec::new() }
    }

    pub fn push(&mut self, name: String, from: String, to: String, times: u32) {
        let edge = Edge::new(name, from, to, times);

        self.edges.push(edge);
    }

    fn enumerate_edges(&self) -> impl Iterator<Item = (usize, &Edge)> {
        self.edges.iter().enumerate()
    }

    pub fn iter(&self) -> std::slice::Iter<'_, Edge> {
        self.edges.iter()
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

    pub fn init_key(&mut self, station_collection: &StationCollection) {
        for station_name in station_collection.iter() {
            self.graph.insert(station_name.1.to_string(), Vec::new());
        }
    }

    pub fn init_value(&mut self, edges: &EdgeStorage) {
        for (_, edge) in edges.enumerate_edges() {
            self.graph
                .entry(edge.from().to_string())
                .or_default()
                .push(edge.to().to_string());
            self.graph
                .entry(edge.to().to_string())
                .or_default()
                .push(edge.from().to_string());
        }
    }

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

    pub fn init_key_value(&mut self, graph: Graph, edge_storage: EdgeStorage) {
        for (start_station, neighbors) in graph.graph.iter() {
            for end_station in neighbors {
                let key = (start_station.clone(), end_station.clone());

                let distance = edge_storage
                    .enumerate_edges()
                    .find(|(_, edge)| {
                        (edge.from().starts_with(start_station)
                            && edge.to().starts_with(end_station))
                            || (edge.from().starts_with(end_station)
                                && edge.to().starts_with(start_station))
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

    pub fn get_distance(&self, start: String, end: String) -> String {
        self.dm
            .get(&(start.to_string(), end.to_string()))
            .or_else(|| self.dm.get(&(end.to_string(), start.to_string())))
            .copied()
            .unwrap_or(0)
            .to_string()
    }
}

#[derive(Debug)]
pub struct Timeline {
    times: BTreeMap<String, u32>,
}

impl Timeline {
    pub fn new() -> Self {
        Self {
            times: BTreeMap::new(),
        }
    }

    pub fn insert(&mut self, train: String, time: u32) {
        self.times.insert(train, time);
    }

    pub fn get_time(&self, train: &str) -> u32 {
        *self.times.get(train).unwrap_or(&0)
    }

    pub fn is_traveled_less(&self, cur: &str, candidate: &str) -> bool {
        self.get_time(cur) < self.get_time(candidate)
    }

    pub fn modify_time(&mut self, train: &str, new_time: u32) {
        if self.times.contains_key(train) {
            self.times.insert(train.to_string(), new_time);
        }
    }

    pub fn trains_with_less_time(&self, train: &str) -> Vec<String> {
        let train_time = self.get_time(train);
        self.times
            .iter()
            .filter(|&(_, &time)| time < train_time)
            .map(|(train, _)| train.clone())
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct TrainMovement {
    time: u32,
    train: String,
    from: u32,
    picked_pkgs: Vec<Package>,
    to: u32,
    drop_pkgs: Vec<Package>,
    carriages: Vec<Package>,
}

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
        for pkg in picked {
            if !(*pkg.status() == PackageStatus::Dummy) {
                self.picked_pkgs.push(pkg.clone());
                self.carriages.push(pkg);
            }
        }
    }

    pub fn with_drop_pkgs(&mut self, drop: Vec<Package>) {
        let mut index: usize = 0;
        let mut found = false;
        for (_, pkg) in drop.iter().enumerate() {
            self.drop_pkgs.push(pkg.clone());
            for (idx, carr) in self.carriages.iter().enumerate() {
                if carr.name() == pkg.name() {
                    index = idx;
                    found = true;
                }
            }
        }

        if found {
            self.carriages.remove(index);
        }
    }

    pub fn with_carriages(&mut self, pkg: Package) {
        self.carriages.push(pkg)
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

    // Method to get an immutable iterator over the packages
    pub fn iter(&self) -> impl Iterator<Item = &Package> {
        self.cand.iter()
    }

    // Method to get a mutable iterator over the packages
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Package> {
        self.cand.iter_mut()
    }
}

pub fn find_nearest_trains(package_index: usize, train_indices: &[usize]) -> Vec<usize> {
    // Return an empty vector if there are no train indices
    if train_indices.is_empty() {
        return Vec::new();
    }

    // Calculate distances and find the minimum distance
    let mut distances: Vec<(usize, usize)> = train_indices
        .iter()
        .map(|&train_index| {
            (
                train_index,
                (train_index as isize - package_index as isize).abs() as usize,
            )
        })
        .collect();

    distances.sort_unstable_by_key(|&(_, distance)| distance);

    // Extract the minimum distance
    let min_distance = distances
        .first()
        .map_or(usize::MAX, |&(_, distance)| distance);

    // Collect all trains with the minimum distance
    distances
        .into_iter()
        .filter(|&(_, distance)| distance == min_distance)
        .map(|(train_index, _)| train_index)
        .collect()
}
