mod some;
mod test;
mod test_unit;
mod utils;

use std::cmp::Ordering;
use std::collections::{BTreeMap, HashMap};
use std::hash::{Hash, Hasher};
use std::{fmt, process};

fn main() {
    println!("Hello, world!");
}

pub fn start_searching(
    mut pkg_c: PackageCollection,
    pkg_c_2: PackageCollection,
    tr_c: &mut TrainCollection,
    stat_c: StationCollection,
    gr: Graph,
    cands: PackageCandidates,
    tr_m: &mut TrainMovement,
    dist_m: &mut DistanceMap,
    tl: &mut Timeline,
    mut pkg_tracker: PackageTracker,
    mut loggerize: Logger,
) {
    for pkg in pkg_c.iter_mut() {
        tracer!(&pkg);

        while pkg_tracker.get_status(&pkg).unwrap() != PackageStatus::Delivered {
            if pkg_tracker.get_status(&pkg).unwrap() == PackageStatus::AwaitingPickup {
                // Store the result of current_locations in a variable with a longer lifetime
                let current_locations = tr_c.current_locations();

                // Get the train indices from the station collection
                let tr_indices = stat_c.indices_of_trains(current_locations.clone());

                // Get pkg.from()
                let nearest_indices =
                    find_nearest_trains(stat_c.get_station_index(pkg.from()).unwrap(), &tr_indices);
                tracer!(&nearest_indices);

                tracer!(&current_locations);
                tracer!(&tr_indices);
                tracer!(&nearest_indices);

                if let Some(nearest) = nearest_indices.first() {
                    let station_name = stat_c.get_station_name(*nearest).unwrap();

                    tracer!(&station_name);

                    // Check if the package's 'from' location matches the nearest station
                    if pkg.from() == stat_c.get_station_name(*nearest).unwrap() {
                        tracer!(nearest);
                        tracer!(&pkg);

                        // How to get initial postion of train
                        let station_name = stat_c.get_station_name(*nearest).unwrap();

                        tracer!(&station_name);

                        // Package same as train index
                        tr_c.push_train_carriage(station_name, pkg);
                        tracer!(&tr_c);
                    } else {
                        let from = pkg.from().clone();

                        // Package not same as train index
                        move_train(
                            &stat_c.clone(),
                            tr_c,
                            nearest,
                            pkg,
                            gr.clone(),
                            pkg_c_2.clone(),
                            pkg_tracker.clone(),
                            &mut loggerize,
                            tr_m,
                            tl,
                            dist_m,
                            &from,
                        );

                        tracer!(&nearest);

                        let station_name = stat_c.get_station_name(*nearest).unwrap();

                        tracer!(&station_name);

                        // process::exit(1);
                        // // Sanity check, if train didnt carry any packages
                        // if !pkg_check.packages.is_empty() {
                        //     pkg_tracker
                        //         .update_status(&pkg, PackageStatus::InTransit)
                        //         .unwrap();
                        // }

                        pkg_tracker
                            .update_status(&pkg, PackageStatus::InTransit)
                            .unwrap();

                        // traced!(&pkg_tracker);
                        tracer!(&tr_c);

                        // process::exit(1);
                        // if pkg.name() == "K2" {
                        //     let locs = tr_c.current_locations();
                        //     tracer!(&locs);
                        //     tracer!(&stat_c);
                        //     tracer!(&tr_indices);
                        //     process::exit(1);
                        // }
                    }
                }
            }

            if pkg_tracker.get_status(&pkg).unwrap() == PackageStatus::InTransit {
                // // Store the result of current_locations in a variable with a longer lifetime
                // let current_locations = tr_c.current_locations();

                // // Get the train indices from the station collection
                // let tr_indices = stat_c.indices_of_trains(current_locations.clone());

                // // Get nearest index to pkg.to()
                // let nearest_indices =
                //     find_nearest_trains(stat_c.get_station_index(pkg.to()).unwrap(), &tr_indices);

                let from = pkg.to().clone();

                // tracer!(&nearest_indices);
                // tracer!(&current_locations);
                // tracer!(&tr_indices);
                // tracer!(&tr_c);

                // Get current train index that hold our current package
                let curr_train = tr_c.find_train_hold_this_pkg(&pkg.name()).unwrap();
                tracer!(&curr_train);
                tracer!(&stat_c);

                let current_index = stat_c
                    .get_station_index(curr_train.current_index())
                    .unwrap();

                // if let Some(nearest) = nearest_indices.first() {
                move_train(
                    &stat_c.clone(),
                    tr_c,
                    &current_index,
                    pkg,
                    gr.clone(),
                    pkg_c_2.clone(),
                    pkg_tracker.clone(),
                    &mut loggerize,
                    tr_m,
                    tl,
                    dist_m,
                    &from,
                );
                // }

                pkg_tracker
                    .update_status(&pkg, PackageStatus::Delivered)
                    .unwrap();
            }

            tracer!(&pkg_tracker);
        }
    }
}

pub fn which_direction(
    gr: Graph,
    tr_c: &mut TrainCollection,
    // from: String,
    nearest: &usize,
    // pkg: &mut Package,
    stat_c: &StationCollection,
    from: String,
) -> String {
    // let from = pkg.from();
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

pub fn move_train(
    stat_c: &StationCollection,
    tr_c: &mut TrainCollection,
    nearest: &usize,
    pkg: &mut Package,
    gr: Graph,
    pkg_c_2: PackageCollection,
    mut pkg_tracker: PackageTracker,
    loggerize: &mut Logger,
    tr_m: &mut TrainMovement,
    tl: &mut Timeline,
    dist_m: &mut DistanceMap,
    from: &String,
) {
    // Next postion
    let mut direction = which_direction(gr.clone(), tr_c, nearest, &stat_c.clone(), from.clone());
    // Current postion
    // let curr_station = stat_c.get_station_name(*nearest).unwrap();

    tracer!(&direction);

    let mut nearest_idx = nearest.clone();
    let train_idx = &stat_c.get_station_name(nearest_idx).unwrap();
    let mut tr_idx = 0;

    for (idx, tr) in tr_c.iter_mut().enumerate() {
        // How to get train on specific index and modify it?
        if tr.current == **train_idx {
            tr_idx = idx;
            break;
        }
    }

    // let mut picked: Vec<Package> = Vec::new();
    // let mut dropped: Vec<Package> = Vec::new();

    while from.clone() != *stat_c.get_station_name(nearest_idx).unwrap() {
        // tr_m.with_picked_pkgs(picked.clone());

        let curr_station = stat_c.get_station_name(nearest_idx).unwrap();
        try_pick_package(
            pkg,
            tr_c,
            &nearest_idx,
            pkg_c_2.clone(),
            pkg_tracker.clone(),
            stat_c,
            from,
            tl,
            stat_c,
            tr_idx,
            direction.clone(),
            curr_station.to_string(),
        );

        // TODO: Remove. Use `find_train_hold_this_pkg`
        // How to get train name?
        // Get station name => Get train origin aka station.
        let station_name = stat_c.get_station_name(nearest_idx).unwrap();

        tracer!(&station_name);

        let curr_train = tr_c.find_train_hold_this_pkg(&pkg.name()).unwrap();

        // if pkg_tracker.get_status(&pkg).unwrap() == PackageStatus::AwaitingPickup {
        if *pkg.from() == *station_name {
            // if pkg_tracker.get_status(&pkg).unwrap() == PackageStatus::AwaitingPickup {
            tr_m.with_picked_pkgs_btree(curr_train.packages.clone());
            // traced!(&pkg.from());
            // traced!(&direction);
            // traced!(&station_name);

            pkg_tracker
                .update_status(&pkg, PackageStatus::InTransit)
                .unwrap();
            // }
        }
        // }

        if pkg_tracker.get_status(&pkg).unwrap() == PackageStatus::InTransit {
            if *pkg.to() == direction {
                tr_m.with_drop_pkgs(curr_train.packages.clone());
            }
        }

        // traced!(&pkg_tracker);
        // tracer!(&picked);

        tr_m.with_time(tl.get_time(&station_name));
        tr_m.with_from(station_name.to_string());
        tr_m.with_to(direction.clone());
        // tr_m.with_drop_pkgs(dropped.clone());

        let distance = dist_m.get_distance(station_name.to_string(), direction.clone());
        let numerize = distance.parse::<u32>().unwrap();
        let curr_name = stat_c.get_station_name(nearest_idx).unwrap();
        let curr_idx = stat_c.get_station_index(&curr_name);
        let near_idx = stat_c.get_station_index(&direction);

        tracer!(&curr_idx);
        tracer!(&near_idx);
        tracer!(&nearest_idx);

        tracer!(&tr_c);
        tracer!(tr_idx);

        let tr_ = tr_c.get_train_mut(tr_idx).unwrap();
        tr_.update_current_index(direction.clone());
        tr_m.with_train(tr_.name().to_string());
        tr_m.plus_time(tl.get_time(&tr_.name()));
        // tr_m.with_picked_pkgs(picked.clone());

        // traced!(&picked);
        // if pkg.from() == station_name {
        //     // tr_m.with_picked_pkgs(picked.clone());
        //     println!("\x1b[0;33m{}\x1b[0m", &tr_m);
        // } else {
        //     println!("\x1b[0;33m{}\x1b[0m", &tr_m);
        // }
        // if first_iter == 0 {
        //     tr_m.with_picked_pkgs(Vec::new());
        //     first_iter += 1;
        // } else if first_iter == 1 {
        //     tr_m.with_picked_pkgs(picked.clone());
        //     traced!(&picked);
        //     // process::exit(1);
        // }

        // traced!(&pkg);
        // traced!(&station_name);

        // if tr_m.train == "Q2" {
        //     traced!(&tr_m);
        //     process::exit(1);
        // }

        // Our output
        tracer!(&tr_m);
        println!("\x1b[0;33m{}\x1b[0m", &tr_m);

        // tr_m.with_picked_pkgs(picked.clone());
        // for p in picked.iter_mut() {
        //     tr_c.push_train_carriage(&station_name, p);
        // }

        // We want the output late in printing time
        tl.accumulate_time(&tr_.name(), numerize);
        tr_m.plus_time(tl.get_time(&tr_.name()));

        tracer!(&tr_c);
        tracer!(&tl);
        tracer!(&loggerize);

        tracer!(&train_idx);
        tracer!(&station_name);
        tracer!(&tr_c);

        // Reassign to closing gap between train and pkg.to()
        nearest_idx = near_idx.unwrap();

        direction = which_direction(
            gr.clone(),
            tr_c,
            &nearest_idx,
            &stat_c.clone(),
            from.to_string(),
        );

        tracer!(&nearest_idx);
        tracer!(&direction);
        tracer!(&station_name);
        tracer!(&tr_c);
        tracer!(&stat_c);
        tracer!(&from);
        tracer!(stat_c.get_station_name(nearest_idx).unwrap());
        tracer!(&direction);

        // Cleaning after prin output
        tr_m.picked_pkgs.clear();
        tr_m.drop_pkgs.clear();
    }
    tracer!(&tr_c);
}

fn get_diff(a: u32, b: u32) -> u32 {
    if a > b {
        a - b
    } else {
        b - a
    }
}

pub fn try_pick_package(
    pkg: &mut Package,
    tr_c: &mut TrainCollection,
    nearest: &usize,
    pkgc_2: PackageCollection,
    pkg_tracker: PackageTracker,
    stat_c: &StationCollection,
    from: &String,
    timeline: &mut Timeline,
    stats_collection: &StationCollection,
    indexx: usize,
    direction: String,
    curr_station: String,
) {
    let mut candidate_train: Option<Train> = None;
    let mut min_distance: u32 = u32::MAX;
    let mut target_pkg = Package::default();
    {
        let train = tr_c.get_train_mut(indexx).unwrap();

        // traced!(&from);
        // traced!(&direction);
        // traced!(&curr_station);
        // traced!(&pkg.from());
        for pkg_ in pkgc_2.iter() {
            if pkg_tracker.get_status(&pkg_).unwrap() == PackageStatus::AwaitingPickup
                && *pkg_.from() == direction
            {
                let pkg_pos = pkg_.from();

                // traced!(&pkg_);
                // let mut candidate_train: Option<Train> = None;
                // let mut min_distance: u32 = u32::MAX;

                // tracer!(&train);
                // tracer!(&pkg_);
                // process::exit(1);
                if train.capacity() < pkg_.weight() {
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

                // tracer!(&pkg_pos);
                // tracer!(&tr_pos);
                // tracer!(&pkg_);
                // tracer!(&traveled_less);
                // tracer!(&diff);
                // tracer!(&current_candidate_name);
                // process::exit(1);
                // }

                if let Some(ref train) = candidate_train {
                    target_pkg = pkg_.clone();
                    tracer!(&train.name());
                    tracer!(&pkg_);
                }
            }
        }

        if let Some(ref train) = candidate_train {
            let target_tr = tr_c.find_train_by_name(&train.name()).unwrap();
            target_tr.push_pkg(target_pkg.clone());

            tracer!(&target_pkg);
            tracer!(&train);
            // traced!(&tr_c);
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
    Pending,
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
            .map(|train| train.current.clone()) // Collect the current location of each train
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

    pub fn len(&self) -> usize {
        self.packages.len()
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

    // Method to return Vec<usize> of indices corresponding to the provided train names
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
