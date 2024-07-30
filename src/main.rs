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

fn get_diff(a: u32, b: u32) -> u32 {
    if a > b {
        a - b
    } else {
        b - a
    }
}

fn find_best_train<'a>(
    tr_collection: &'a mut TrainCollection,
    package: &Package,
    stats_collection: &StationCollection,
) -> Option<&'a Train> {
    let mut traveled_less = u32::MAX;
    let mut close_distance = u32::MAX;
    let mut best_train: Option<&'a Train> = None;

    for train in tr_collection.trains.iter() {
        // Skip trains that can't carry the package
        if train.remain_capacity < package.weight {
            continue;
        }

        // Get indices of the package's origin and the train's current location
        let pkg_from_index = stats_collection.get_station_index(&package.from).unwrap();
        let train_curr_index = stats_collection.get_station_index(&train.current).unwrap();

        // Calculate the distance difference
        let diff = get_diff(pkg_from_index as u32, train_curr_index as u32);

        // Determine if this train has traveled less time
        let is_less = if train.time < traveled_less {
            traveled_less = train.time;
            true
        } else {
            false
        };

        // Update the best train if the current train is closer or has less travel time
        if diff < close_distance || (diff == close_distance && is_less) {
            close_distance = diff;
            best_train = Some(train);
        }
    }

    best_train
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
        if let Some((_, package)) = pkg_collection.first() {
            // tracer!("First package key: {}, value: {:?}", key, package);
            let mut dummy_tr_collection = tr_collection.clone();

            let candidate =
                find_best_train(&mut dummy_tr_collection, package, stats_collection).unwrap();

            if pkg_tracker.get_status(&package).unwrap() == PackageStatus::AwaitingPickup {
                // tracer!(&candidate);

                tracer!(&pkg_tracker);
                // tracer!(&tr_collection);
                // tracer!(&nearest);
                // tracer!(&package);

                // If package is same as train location
                // if let Some(nearest) = nearest_indices.first() {
                // if the package.from() matches the nearest station
                if package.from() == candidate.current_index() {
                    // Convert nearest index to station's name (station's name is package's index)
                    // let station_name = stats_collection.get_station_name(nearest).unwrap();

                    // Push station's name and package to current train's carriage
                    tr_collection.push_train_carriage(&candidate.clone(), &package);

                    // Mark package as InTransit
                    pkg_tracker
                        .update_status(&package, PackageStatus::InTransit)
                        .unwrap();
                } else {
                    //  If package is not same as train location, move train to package.from()
                    advance_train(
                        &stats_collection.clone(),
                        tr_collection,
                        // &nearest, // Pass nearest train to our package
                        &mut package.clone(),
                        graph.clone(),
                        pkg_collection,
                        pkg_tracker.clone(),
                        tr_movement,
                        timeline,
                        dist_map,
                        &package.from(), // Pass packagae location
                        loggerize,
                        candidate,
                    );
                }
            }

            if pkg_tracker.get_status(&package).unwrap() == PackageStatus::InTransit {
                // tracer!(&tr_collection);

                tracer!(&pkg_tracker);

                let mut dummy = tr_collection.clone();
                let candidate = dummy.find_train_hold_this_pkg(package).unwrap();

                // tracer!(&package);
                // tracer!(&candidate);

                advance_train(
                    &stats_collection.clone(),
                    tr_collection,
                    // &current_index, // Pass real time train index that hold current package
                    &mut package.clone(),
                    graph.clone(),
                    pkg_collection,
                    pkg_tracker.clone(),
                    tr_movement,
                    timeline,
                    dist_map,
                    &package.to(), // Passed package destination
                    loggerize,
                    candidate,
                );

                // Mark package as delivered
                pkg_tracker
                    .update_status(&package, PackageStatus::Delivered)
                    .unwrap();

                pkg_collection.remove_first();
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
    // curr_train_stat: String,
    stat_c: &StationCollection,
    from: String,
) -> String {
    let target_station = stat_c.get_station_name(*nearest).unwrap();
    let neighbours = gr.get_neighbors(&target_station).unwrap();

    let (left, right) = match neighbours.as_slice() {
        [] => (String::new(), String::new()),
        [first] => (first.clone(), String::new()),
        [first, second] => (first.clone(), second.clone()),
        _ => (String::new(), String::new()),
    };

    // tracer!(&left);
    // tracer!(&right);
    // tracer!(&from);

    if left == *from || right.is_empty() {
        return left;
    }

    if right == *from {
        return right;
    }

    // tracer!(stat_c.get_station_index(&from));
    // tracer!(stat_c.get_station_index(&target_station));

    if stat_c.get_station_index(&from) >= stat_c.get_station_index(&target_station) {
        return right;
    }

    left
}

pub fn advance_train(
    stats_collection: &StationCollection,
    tr_collection: &mut TrainCollection,
    // nearest: &usize,
    package: &mut Package,
    graph: Graph,
    pkg_collection: &PackageCollection,
    mut pkg_tracker: PackageTracker,
    tr_movement: &mut TrainMovement,
    timeline: &mut Timeline,
    dist_map: &mut DistanceMap,
    where_to: &String,
    loggerize: &mut Logger,
    candidate: &Train,
) {
    let cand_index = stats_collection
        .get_station_index(candidate.current_index())
        .unwrap();

    // Next postion: Left or Right?
    // A   <-   B(current)   ->   C
    let mut next_station = which_direction(
        graph.clone(),
        // nearest, // TODO: We need tha station name only. Not index
        &cand_index,
        &stats_collection.clone(),
        where_to.clone(),
    );

    // Increment.
    // We use this to closing the gap to package location or package destination
    let current_cand_station = candidate.current_index();
    let mut current_idx = stats_collection
        .get_station_index(current_cand_station)
        .expect("Can't get usize version of curr candidate train");

    // Get train's name at the nearest index
    // let station_name = stats_collection.get_station_name(current_idx).unwrap();

    // tracer!(&nearest);
    // tracer!(&current_idx);

    // Find the index of the train with the matching name
    // let train_index = tr_collection
    //     .iter_mut()
    //     .position(|tr| tr.current == *station_name)
    //     .unwrap_or(0);

    // if t

    let train_index = tr_collection
        .find_index_of_train_with_least_time(&candidate.current_index(), package)
        .unwrap();

    // tracer!(&package);
    // tracer!(&train_index);
    // tracer!(&station_name);
    // tracer!(&tr_collection);

    // let train_index = tr_collection
    // .iter_mut().position(|tr| tr.current == *station_name && tr.time is lesser than the rest)

    // Using where_to mean we need to calculate early for package.to(), otherwise
    // we wont reach package.to()
    while *where_to != *stats_collection.get_station_name(current_idx).unwrap() {
        // Try pick package at current train index while moving to where_to
        try_pick_package(
            tr_collection,
            &mut pkg_collection.clone(),
            pkg_tracker.clone(),
            train_index,
            next_station.clone(),
        );

        // tracer!(&tr_collection);
        // tracer!(&package);

        // Get current station as we closing gap to package's location or destination
        let curr_station = stats_collection.get_station_name(current_idx).unwrap();

        // Since we already pick the same package, there is no current package in
        // tr_collection
        if let Some(curr_train) = tr_collection.find_train_hold_this_pkg(&package) {
            for (_, loaded_pkg) in curr_train.packages.clone() {
                // If package location is same as current station
                if *loaded_pkg.from() == *curr_station && !loggerize.already_picked(&loaded_pkg) {
                    tr_movement.with_picked_pkage_btree(loaded_pkg.clone());
                    curr_train.remove_package(&loaded_pkg);
                    loggerize.push_picked(loaded_pkg.clone());
                    pkg_tracker
                        .update_status(&loaded_pkg, PackageStatus::InTransit)
                        .unwrap();
                }

                if pkg_tracker.get_status(&loaded_pkg).unwrap() == PackageStatus::InTransit {
                    // If package destination same as next iteration.
                    // We check this early because there is no ways next iteration to happen
                    // because of our while loop condition.
                    if *loaded_pkg.to() == next_station && !loggerize.already_dropped(&loaded_pkg) {
                        tr_movement.with_drop_pkage_btree(loaded_pkg.clone());
                        loggerize.push_dropped(loaded_pkg.clone());
                        curr_train.remove_package(&loaded_pkg);
                    }
                }
            }
        }

        // let curr_index = stats_collection.get_station_index(&curr_station);
        let next_index = stats_collection.get_station_index(&next_station);

        tr_movement.with_time(timeline.get_time(&curr_station));
        tr_movement.with_from(curr_station.to_string());
        tr_movement.with_to(next_station.clone());

        // tracer!(&tr_collection.clone());

        // Modify current train that hold current package or advance to package location
        // TODO: Wrong
        let current_train = tr_collection.get_train_mut(train_index).unwrap();

        // tracer!(&tr_collection);
        // tracer!(&train_index);
        // tracer!(&current_train);

        current_train.update_current_index(next_station.clone());
        // current_train.accumulate_time()
        tr_movement.with_train(current_train.name().to_string());
        tr_movement.plus_time(timeline.get_time(&current_train.name()));

        // Accumulate timeline
        let distance = dist_map.get_distance(curr_station.to_string(), next_station.clone());
        let numerize_distance = distance.parse::<u32>().unwrap();
        current_train.accumulate_time(numerize_distance);

        // TODO: Remove
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

pub fn try_pick_package(
    tr_collection: &mut TrainCollection,
    pkg_collection: &mut PackageCollection,
    pkg_tracker: PackageTracker,
    train_index: usize,
    next_station: String,
) {
    let train = tr_collection.get_train_mut(train_index).unwrap();

    // If pkg_collection is empty, this loop will be skip
    for pkg_ in pkg_collection.iter() {
        if *pkg_.from() == next_station || *pkg_.to() == next_station {
            if train.capacity() < pkg_.weight() {
                continue;
            }

            if !(pkg_tracker.get_status(pkg_).unwrap() == PackageStatus::AwaitingPickup) {
                continue;
            }

            train.push_pkg(pkg_.clone());

            continue;
        }
    }
}

#[derive(Debug)]
pub struct Logger {
    picked: BTreeMap<String, Package>,
    dropped: BTreeMap<String, Package>,
}

impl Logger {
    pub fn new() -> Self {
        Self {
            picked: BTreeMap::new(),
            dropped: BTreeMap::new(),
        }
    }

    pub fn already_picked(&self, pkg: &Package) -> bool {
        self.picked.contains_key(&pkg.name)
    }

    pub fn push_picked(&mut self, pkg: Package) {
        self.picked.insert(pkg.name.clone(), pkg);
    }

    pub fn already_dropped(&self, pkg: &Package) -> bool {
        self.dropped.contains_key(&pkg.name)
    }

    pub fn push_dropped(&mut self, pkg: Package) {
        self.dropped.insert(pkg.name.clone(), pkg);
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
        if let Some(_) = self.packages.remove(&package.name) {
            //
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

#[derive(Debug, Clone)]
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

    pub fn find_closest_train(&self, package: &Package) -> Option<&Train> {
        self.trains
            .iter()
            .filter(|train| train.current == package.from)
            .min_by_key(|train| train.time)
    }

    pub fn find_closest_train_less_weight(
        &self,
        package: &Package,
        trainee: &Train,
    ) -> Option<&Train> {
        self.trains
            .iter()
            .filter(|train| train.capacity() > package.weight())
            .filter(|train| train.name() != trainee.name())
            .min_by_key(|train| train.time)
    }

    pub fn find_closest_train_with_constraint(&self, package: &Package) -> Option<&Train> {
        self.trains
            .iter()
            .filter(|train| {
                train.remain_capacity >= package.weight && train.current == package.from
            })
            .min_by_key(|train| train.time)
    }

    pub fn find_index_of_train_with_least_time(
        &mut self,
        station_name: &str,
        pkg: &Package,
    ) -> Option<usize> {
        // Find all indices of trains at the given station
        let filtered_trains: Vec<usize> = self
            .trains
            .iter()
            .enumerate()
            .filter_map(|(index, train)| {
                if train.current == station_name {
                    if train.remain_capacity() > pkg.weight() {
                        Some(index)
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();

        // If no trains found at the station, return None
        if filtered_trains.is_empty() {
            return None;
        }

        // Find the index of the train with the least time
        filtered_trains
            .into_iter()
            .min_by_key(|&index| self.trains[index].time)
    }

    pub fn find_train_by_station_and_least_time(
        &mut self,
        station_name: &str,
    ) -> Option<&mut Train> {
        self.trains
            .iter_mut()
            .filter(|train| train.current == station_name)
            .min_by_key(|train| train.time)
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
    pub fn push_train_carriage(&mut self, tr_cand: &Train, pkg: &Package) {
        for train in self.trains.iter_mut() {
            if train.name() == tr_cand.name() {
                train.packages.insert(pkg.name().to_string(), pkg.clone());
            }
        }
    }

    /// Find train based on the origin
    pub fn find_train_by_name(&mut self, name: &str) -> Option<&mut Train> {
        self.trains.iter_mut().find(|train| train.name() == name)
    }

    /// Find a train based on the package name
    pub fn find_train_hold_this_pkg(&mut self, pkg: &Package) -> Option<&mut Train> {
        self.trains.iter_mut().find(|train| {
            train.current_index() == pkg.from() && train.packages.contains_key(pkg.name())
        })
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

    pub fn remove_first(&mut self) {
        if let Some(key) = self.packages.keys().next().cloned() {
            self.packages.remove_entry(&key);
        }
    }

    pub fn first(&self) -> Option<(&String, &Package)> {
        self.packages.iter().next()
    }

    pub fn first_mut(&mut self) -> Option<(&String, &mut Package)> {
        self.packages.iter_mut().next()
    }

    pub fn clone_collection(&self) -> Self {
        self.clone()
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

    pub fn find_least_timeline(&self) -> Option<(&String, &u32)> {
        self.times.iter().min_by_key(|entry| entry.1)
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

        let picked_keys: Vec<String> = self.picked_pkgs.keys().cloned().collect();
        let picked = picked_keys.join(", ");

        write!(f, "P1=[{}]", picked)?;
        write!(f, ", ")?;
        write!(f, "N2={}", self.to)?;
        write!(f, ", ")?;

        let dropped_keys: Vec<String> = self.drop_pkgs.keys().cloned().collect();
        let dropped = dropped_keys.join(", ");

        write!(f, "P2=[{}]", dropped)?;

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

    pub fn with_picked_pkage_btree(&mut self, picked: Package) {
        self.picked_pkgs
            .insert(picked.name().to_string(), picked.clone());
        self.carriages.insert(picked.name().to_string(), picked);
    }

    pub fn with_drop_pkage_btree(&mut self, picked: Package) {
        self.drop_pkgs
            .insert(picked.name().to_string(), picked.clone());
        self.carriages.remove(picked.name());
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
