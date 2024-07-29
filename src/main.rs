mod test;
mod test_unit;
mod utils;

use std::cmp::Ordering;
use std::collections::{BTreeMap, HashMap};
use std::fmt;
use std::hash::{Hash, Hasher};

#[allow(unused_imports)]
use std::process;

fn main() {
    println!("Hello, world!");
}

pub fn start_searching(
    pkg_collection: &mut PackageCollection,
    tr_collection: &mut TrainCollection,
    stats_collection: &StationCollection,
    graph: Graph,
    tr_movement: &mut TrainMovement,
    dist_map: &mut DistanceMap,
    timeline: &mut Timeline,
    mut pkg_tracker: PackageTracker,
    loggerize: &mut Logger,
) {
    while !pkg_collection.is_empty() {
        if let Some((key, package)) = pkg_collection.pick_first() {
            tracer!("First package key: {}, value: {:?}", key, package);

            if pkg_tracker.get_status(&package).unwrap() == PackageStatus::AwaitingPickup {
                // Accumulate realtime location "A, B, C, D..."
                let current_locations = tr_collection.current_locations();

                // Convert train location to index: "A, B, C, D..." => "0, 1, 2, 3..."
                let tr_indices = stats_collection.indices_of_trains(current_locations.clone());

                // TODO: Use `get_nearest_train_to_pkg`
                // Get nearest train to the package
                let nearest_indices = find_nearest_trains(
                    stats_collection.get_station_index(package.from()).unwrap(),
                    &tr_indices,
                );

                // If package is same as train location
                if let Some(nearest) = nearest_indices.first() {
                    // if the package.from() matches the nearest station
                    if package.from() == stats_collection.get_station_name(*nearest).unwrap() {
                        // Convert nearest index to station's name (station's name is package's index)
                        let station_name = stats_collection.get_station_name(*nearest).unwrap();

                        // Push station's name and package to current train's carriage
                        tr_collection.push_train_carriage(station_name, &package);

                        // Mark package as InTransit
                        pkg_tracker
                            .update_status(&package, PackageStatus::InTransit)
                            .unwrap();
                    } else {
                        //  If package is not same as train location, move train to package.from()
                        advance_train(
                            &stats_collection.clone(),
                            tr_collection,
                            nearest, // Pass nearest train to our package
                            &mut package.clone(),
                            graph.clone(),
                            pkg_collection,
                            pkg_tracker.clone(),
                            loggerize,
                            tr_movement,
                            timeline,
                            dist_map,
                            &package.from(), // Pass packagae location
                        );
                    }
                }
            }

            if pkg_tracker.get_status(&package).unwrap() == PackageStatus::InTransit {
                // Get real time current train index that hold our current package
                let curr_train = tr_collection
                    .find_train_hold_this_pkg(&package.name())
                    .unwrap();

                // Get real time train's index
                let current_index = stats_collection
                    .get_station_index(curr_train.current_index())
                    .unwrap();

                advance_train(
                    &stats_collection.clone(),
                    tr_collection,
                    &current_index, // Pass real time train index that hold current package
                    &mut package.clone(),
                    graph.clone(),
                    pkg_collection,
                    pkg_tracker.clone(),
                    loggerize,
                    tr_movement,
                    timeline,
                    dist_map,
                    &package.to(), // Passed package destination
                );

                // Mark package as delivered
                pkg_tracker
                    .update_status(&package, PackageStatus::Delivered)
                    .unwrap();
            }
        } else {
            tracer!("No packages in the collection.");
            break;
        }
    }
}

pub fn which_direction(
    gr: Graph,
    nearest: &usize,
    stat_c: &StationCollection,
    from: String,
) -> String {
    let target_station = stat_c.get_station_name(*nearest).unwrap();
    let neighbours = gr.get_neighbors(target_station).unwrap();

    let (left, right) = match neighbours.as_slice() {
        [] => (String::new(), String::new()),
        [first] => (first.clone(), String::new()),
        [first, second] => (first.clone(), second.clone()),
        _ => (String::new(), String::new()),
    };

    tracer!(&left);
    tracer!(&right);
    tracer!(&from);

    if left == *from || right.is_empty() {
        return left;
    }

    if right == *from {
        return right;
    }

    tracer!(stat_c.get_station_index(&from));
    tracer!(stat_c.get_station_index(&target_station));

    if stat_c.get_station_index(&from) >= stat_c.get_station_index(&target_station) {
        return right;
    }

    left
}

pub fn advance_train(
    stats_collection: &StationCollection,
    tr_collection: &mut TrainCollection,
    nearest: &usize,
    package: &mut Package,
    graph: Graph,
    pkg_collection: &mut PackageCollection,
    mut pkg_tracker: PackageTracker,
    loggerize: &mut Logger,
    tr_movement: &mut TrainMovement,
    timeline: &mut Timeline,
    dist_map: &mut DistanceMap,
    where_to: &String,
) {
    // Next postion: Left or Right?
    // A   <-   B(current)   ->   C
    let mut next_station = which_direction(
        graph.clone(),
        nearest,
        &stats_collection.clone(),
        where_to.clone(),
    );

    // Increment.
    // We use this to closing the gap to package location or package destination
    let mut current_idx = nearest.clone();

    // Get train's name at the nearest index
    let train_name = stats_collection.get_station_name(current_idx).unwrap();

    // Find the index of the train with the matching name
    let train_index = tr_collection
        .iter_mut()
        .position(|tr| tr.current == *train_name)
        .unwrap_or(0);

    while where_to.clone() != *stats_collection.get_station_name(current_idx).unwrap() {
        // Try pick package at current train index while moving to where_to
        try_pick_package(
            tr_collection,
            &mut pkg_collection.clone(),
            pkg_tracker.clone(),
            timeline,
            stats_collection,
            train_index,
            next_station.clone(),
        );

        tracer!(&tr_collection);
        tracer!(&package);

        let curr_train = tr_collection
            .find_train_hold_this_pkg(&package.name())
            .unwrap();

        // Get current station as we closing gap to package's location or destination
        let curr_station = stats_collection.get_station_name(current_idx).unwrap();

        // If package location is same as current station
        if *package.from() == *curr_station {
            tr_movement.with_picked_pkgs_btree(curr_train.packages.clone());
            pkg_tracker
                .update_status(&package, PackageStatus::InTransit)
                .unwrap();
        }

        if pkg_tracker.get_status(&package).unwrap() == PackageStatus::InTransit {
            // If package destination same as next iteration.
            // We check this early because there is no ways next iteration to happen
            // because of our while loop condition.
            if *package.to() == next_station {
                tr_movement.with_drop_pkgs(curr_train.packages.clone());
            }
        }

        // let curr_index = stats_collection.get_station_index(&curr_station);
        let next_index = stats_collection.get_station_index(&next_station);

        tr_movement.with_time(timeline.get_time(&curr_station));
        tr_movement.with_from(curr_station.to_string());
        tr_movement.with_to(next_station.clone());

        // Modify current train that hold current package or advance to package location
        let current_train = tr_collection.get_train_mut(train_index).unwrap();
        current_train.update_current_index(next_station.clone());
        tr_movement.with_train(current_train.name().to_string());
        tr_movement.plus_time(timeline.get_time(&current_train.name()));

        // Accumulate timeline
        let distance = dist_map.get_distance(curr_station.to_string(), next_station.clone());
        let numerize_distance = distance.parse::<u32>().unwrap();
        timeline.accumulate_time(&current_train.name(), numerize_distance);

        println!("\x1b[0;33m{}\x1b[0m", &tr_movement);

        // Reassign to closing gap between train's index and where_to
        current_idx = next_index.unwrap();

        next_station = which_direction(
            graph.clone(),
            &current_idx,
            &stats_collection.clone(),
            where_to.to_string(),
        );

        tr_movement.picked_pkgs.clear();
        tr_movement.drop_pkgs.clear();
    }
}

fn get_diff(a: u32, b: u32) -> u32 {
    if a > b {
        a - b
    } else {
        b - a
    }
}

pub fn try_pick_package(
    tr_collection: &mut TrainCollection,
    pkg_collection: &mut PackageCollection,
    pkg_tracker: PackageTracker,
    timeline: &mut Timeline,
    stats_collection: &StationCollection,
    train_index: usize,
    next_station: String,
) {
    let mut candidate_train: Option<Train> = None;
    let mut min_distance: u32 = u32::MAX;
    let mut target_pkg = Package::default();
    {
        let train = tr_collection.get_train_mut(train_index).unwrap();

        for try_pkg in pkg_collection.iter() {
            if *try_pkg.from() == next_station || *try_pkg.to() == next_station {
                let pkg_pos = try_pkg.from();

                if train.capacity() < try_pkg.weight() {
                    continue;
                }

                let tr_pos = train.current_index();
                let pkg_idx = stats_collection.get_station_index(&pkg_pos).unwrap();
                let tr_idx = stats_collection.get_station_index(tr_pos).unwrap();
                let diff = get_diff(pkg_idx as u32, tr_idx as u32);
                let tr_name = train.name();

                let current_candidate_name = candidate_train
                    .as_ref()
                    .map(|t| t.name().to_string())
                    .unwrap_or_default();

                let traveled_less =
                    timeline.get_time(&tr_name) < timeline.get_time(&current_candidate_name);

                if diff == min_distance && traveled_less {
                    candidate_train = Some(train.clone());
                    min_distance = diff;
                }

                if diff < min_distance {
                    candidate_train = Some(train.clone());
                    min_distance = diff;
                }

                if let Some(ref train) = candidate_train {
                    target_pkg = try_pkg.clone();
                    tracer!(&train.name());
                    tracer!(&try_pkg);
                }
            }
        }

        if let Some(ref train) = candidate_train {
            let target_tr = tr_collection.find_train_by_name(&train.name()).unwrap();
            target_tr.push_pkg(target_pkg.clone());

            tracer!(&target_pkg);
            tracer!(&train);
        }
    }
}

#[derive(Debug)]
pub struct Logger {
    log: BTreeMap<String, Package>,
}

impl Logger {
    pub fn new() -> Self {
        Self {
            log: BTreeMap::new(),
        }
    }

    pub fn already_add(&self, pkg: &Package) -> bool {
        self.log.contains_key(&pkg.name)
    }

    pub fn push(&mut self, pkg: Package) {
        self.log.insert(pkg.name.clone(), pkg);
    }
}

pub struct PackageName {
    name: Vec<String>,
}

impl PackageName {
    // Creates a new PackageName with an empty Vec<String>
    pub fn new() -> Self {
        PackageName { name: Vec::new() }
    }

    // Adds a new name to the package
    pub fn add_name(&mut self, name: String) {
        self.name.push(name);
    }

    // Retrieves a reference to the names
    pub fn names(&self) -> &[String] {
        &self.name
    }

    // Retrieves the number of names in the package
    pub fn count(&self) -> usize {
        self.name.len()
    }

    // Checks if the package contains a specific name
    pub fn contains(&self, name: &str) -> bool {
        self.name.contains(&name.to_string())
    }

    // Removes a specific name from the package
    pub fn remove_name(&mut self, name: &str) -> bool {
        if let Some(pos) = self.name.iter().position(|x| x == name) {
            self.name.remove(pos);
            true
        } else {
            false
        }
    }

    // Returns an iterator over the names in the package
    pub fn iter(&self) -> impl Iterator<Item = &String> {
        self.name.iter()
    }
}

#[derive(Debug, Clone)]
pub struct PackageTracker {
    track: BTreeMap<Package, PackageStatus>,
}

impl PackageTracker {
    pub fn new() -> Self {
        Self {
            track: BTreeMap::new(),
        }
    }

    // Add a package to the tracker
    pub fn add_package(&mut self, package: Package, status: PackageStatus) {
        self.track.insert(package, status);
    }

    // Update the status of a package
    pub fn update_status(
        &mut self,
        package: &Package,
        new_status: PackageStatus,
    ) -> Result<(), &'static str> {
        if let Some(status) = self.track.get_mut(package) {
            *status = new_status;
            Ok(())
        } else {
            Err("Package not found")
        }
    }

    // Get the status of a package
    pub fn get_status(&self, package: &Package) -> Option<PackageStatus> {
        self.track.get(package).cloned()
    }

    // Remove a package from the tracker
    pub fn remove_package(&mut self, package: &Package) -> Result<(), &'static str> {
        if self.track.remove(package).is_some() {
            Ok(())
        } else {
            Err("Package not found")
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum PackageStatus {
    InTransit,
    Delivered,
    AwaitingPickup,
    Dummy,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Package {
    name: String,
    weight: u32,
    from: String,
    to: String,
    status: PackageStatus,
}

impl Ord for Package {
    fn cmp(&self, other: &Self) -> Ordering {
        self.name.cmp(&other.name)
    }
}

impl PartialOrd for Package {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.name.cmp(&other.name))
    }
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

impl Hash for Package {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.weight.hash(state);
        self.from.hash(state);
        self.to.hash(state);
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Train {
    name: String,
    capacity: u32,
    remain_capacity: u32,
    origin: String,
    current: String,
    packages: BTreeMap<String, Package>,
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
        let packages = packages
            .into_iter()
            .map(|pkg| (pkg.name.clone(), pkg))
            .collect();

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

    /// Add a vector of packages to the train.
    pub fn push_packages(&mut self, packages: Vec<Package>) {
        for pkg in packages {
            self.packages.insert(pkg.name.clone(), pkg);
        }
    }

    pub fn push_pkg(&mut self, pkg: Package) {
        self.packages.insert(pkg.name.clone(), pkg);
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
        self.packages.contains_key(&package.name)
    }

    /// Load a vector of packages onto the train.
    pub fn load_packages(&mut self, packages: Vec<Package>) {
        for pkg in packages {
            if pkg.weight <= self.remain_capacity {
                self.packages.insert(pkg.name.clone(), pkg.clone());
                self.remain_capacity -= pkg.weight;
            }
        }
    }

    /// Remove a package from the train.
    pub fn remove_package(&mut self, package: &Package) {
        if self.packages.remove(&package.name).is_some() {
            self.remain_capacity += package.weight;
        }
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

    // Method to return a Vec<String> of current locations of all trains
    pub fn current_locations(&self) -> Vec<String> {
        self.trains
            .iter()
            .map(|train| train.current.clone())
            .collect()
    }

    /// Push package at train's index
    pub fn push_train_carriage(&mut self, train_name: &str, pkg: &Package) {
        for train in self.trains.iter_mut() {
            if train.current == train_name {
                train.packages.insert(pkg.name().to_string(), pkg.clone());
            }
        }
    }

    /// Find train based on the origin
    pub fn find_train_by_name(&mut self, name: &str) -> Option<&mut Train> {
        self.trains.iter_mut().find(|train| train.name() == name)
    }

    /// Find a train based on the package name
    pub fn find_train_hold_this_pkg(&mut self, name: &str) -> Option<&mut Train> {
        self.trains
            .iter_mut()
            .find(|train| train.packages.contains_key(name))
    }
}

#[derive(Debug, Clone)]
pub struct PackageCollection {
    packages: BTreeMap<String, Package>,
}

impl PackageCollection {
    pub fn new() -> Self {
        Self {
            packages: BTreeMap::new(),
        }
    }

    pub fn first(&self) -> Option<(&String, &Package)> {
        self.packages.iter().next()
    }

    pub fn len(&self) -> usize {
        self.packages.len()
    }

    pub fn is_empty(&mut self) -> bool {
        self.packages.is_empty()
    }

    /// Pick first items from vector and drop that item
    pub fn pick_first(&mut self) -> Option<(String, Package)> {
        if let Some(key) = self.packages.keys().next().cloned() {
            self.packages.remove_entry(&key)
        } else {
            None
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
        let package = Package::new(name.clone(), weight, from, to, status);
        self.packages.insert(name, package);
    }

    pub fn get_package(&self, name: &str) -> Option<&Package> {
        self.packages.get(name)
    }

    pub fn get_package_mut(&mut self, name: &str) -> Option<&mut Package> {
        self.packages.get_mut(name)
    }

    // Method to get an immutable iterator over the packages
    pub fn iter(&self) -> impl Iterator<Item = &Package> {
        self.packages.values()
    }

    // Method to get a mutable iterator over the packages
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Package> {
        self.packages.values_mut()
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

    pub fn get_nearest_train_to_pkg(
        &self,
        pkg: &Package,
        train_indices: &Vec<usize>,
    ) -> Vec<usize> {
        let pkg_index = self
            .get_station_index(&pkg.from())
            .expect("Cannot get pkg index");

        let nearest_indices = find_nearest_trains(pkg_index, &train_indices);
        nearest_indices
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

    // Get an iterator over the station collection
    pub fn iter(&self) -> std::collections::btree_map::Iter<'_, usize, String> {
        self.stations.iter()
    }

    // Get an iterator over the station names only
    pub fn names(&self) -> impl Iterator<Item = &String> {
        self.stations.values()
    }

    /// Return Vec<usize> of indices corresponding to the provided train names
    pub fn indices_of_trains(&self, train_names: Vec<String>) -> Vec<usize> {
        train_names
            .iter()
            .filter_map(|name| {
                self.stations
                    .iter()
                    .find(|(_, station_name)| *station_name == name)
                    .map(|(index, _)| *index)
            })
            .collect()
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

    pub fn accumulate_time(&mut self, train: &str, additional_time: u32) {
        self.times
            .entry(train.to_string())
            .and_modify(|time| *time += additional_time)
            .or_insert(additional_time);
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
    from: String,
    to: String,
    picked_pkgs: BTreeMap<String, Package>,
    drop_pkgs: BTreeMap<String, Package>,
    carriages: BTreeMap<String, Package>,
}

impl fmt::Display for TrainMovement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "W={}", self.time)?;
        write!(f, ", ")?;
        write!(f, "T={}", self.train)?;
        write!(f, ", ")?;
        write!(f, "N1={}", self.from)?;
        write!(f, ", ")?;

        let p1 = self
            .picked_pkgs
            .iter()
            .next()
            .map_or("", |(_, pkg)| pkg.name());
        write!(f, "P1=[{}]", p1)?;
        write!(f, ", ")?;
        write!(f, "N2={}", self.to)?;
        write!(f, ", ")?;

        let p2 = self
            .drop_pkgs
            .iter()
            .next()
            .map_or("", |(_, pkg)| pkg.name());
        write!(f, "P2=[{}]", p2)?;

        Ok(())
    }
}

impl TrainMovement {
    pub fn new() -> Self {
        Self {
            time: 0,
            train: "".to_string(),
            from: "".to_string(),
            to: "".to_string(),
            picked_pkgs: BTreeMap::new(),
            drop_pkgs: BTreeMap::new(),
            carriages: BTreeMap::new(),
        }
    }

    pub fn with_time(&mut self, time: u32) {
        self.time = time;
    }

    pub fn plus_time(&mut self, time: u32) {
        self.time += time;
    }

    pub fn with_train(&mut self, train_name: String) {
        self.train = train_name;
    }

    pub fn with_from(&mut self, from_index: String) {
        self.from = from_index;
    }

    pub fn with_to(&mut self, to_index: String) {
        self.to = to_index;
    }

    pub fn with_picked_pkgs(&mut self, picked: Vec<Package>) {
        for pkg in picked {
            self.picked_pkgs.insert(pkg.name.clone(), pkg.clone());
            self.carriages.insert(pkg.name.clone(), pkg);
        }
    }

    pub fn with_picked_pkgs_btree(&mut self, picked: BTreeMap<String, Package>) {
        for (name, pkg) in picked {
            self.picked_pkgs.insert(name.clone(), pkg.clone());
            self.carriages.insert(name, pkg);
        }
    }

    pub fn with_drop_pkgs(&mut self, drop: BTreeMap<String, Package>) {
        for (name, pkg) in drop {
            self.drop_pkgs.insert(name.clone(), pkg.clone());
            self.carriages.remove(&name);
        }
    }

    pub fn with_carriages(&mut self, pkg: Package) {
        self.carriages.insert(pkg.name.clone(), pkg);
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
