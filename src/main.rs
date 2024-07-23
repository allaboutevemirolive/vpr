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

#[derive(Clone)]
/// All stations
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

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum PackageStatus {
    AwaitingPickup,
    InTransit,
    Delivered,
    Dummy,
}

#[derive(Debug, Clone)]
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
#[derive(Debug)]
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

#[derive(Debug)]
pub struct Train {
    name: String,
    /// Maximum weight, a train can carry packages. A train can carry `MORE` than 1 packages if
    /// those package doesn't exceed this capacity
    capacity: u32,

    /// Origin index
    origin: u32,

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
        origin: u32,
        current: u32,
        packages: Vec<Package>,
        time: u32,
    ) -> Self {
        Self {
            name,
            capacity,
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

    pub fn push_packages(&mut self, package: Package) {
        self.packages.push(package)
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
        let train = Train::new(name, capacity, origin, current, pkgs, time);
        self.trains.push(train);
    }

    fn enumerate_trains(&self) -> impl Iterator<Item = (usize, &Train)> {
        self.trains.iter().enumerate()
    }
}

// Our solution
// #[derive(Debug, Clone)]
// pub struct TrainMove {
//     /// Time taken to delivering all packages
//     time: u32,
//     train_id: String,

//     /// Current station
//     start_node: String,
//     /// Does train pick-up any packages at start_node?
//     picked_up: Vec<String>,

//     /// Next station
//     end_node: String,

//     // TODO: Redundant! We only need 1 field that store picked_up and dropped_off. Later, we will remove this.
//     /// Does train drop any packages at end_node?
//     dropped_off: Vec<String>,

//     /// Are there any remaining packages in train's carriage?
//     in_carriage: Vec<String>,
// }

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

// impl fmt::Display for TrainMove {
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
