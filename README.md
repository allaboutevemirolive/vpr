# Vehicle Routing Problem (VRP)

This project provides a solver for the Vehicle Routing Problem (VRP), a classic optimization problem with widespread applications in logistics and transportation. The goal of the VRP is to find the most efficient set of routes for a fleet of vehicles to service a given set of customers.

### Background

The VRP is a generalization of the Traveling Salesman Problem (TSP) and was first introduced by Dantzig and Ramser in 1959. Due to its NP-hard nature, finding exact solutions for large-scale VRPs is computationally challenging. This project implements both exact and heuristic methods to tackle VRP instances.


### Important Notice

This codebase is currently in a **developmental stage** and supports a **limited set of use cases**. It is **not recommended for production environments** due to potential limitations and ongoing refinements.


### How to Run the Tests

To execute the tests, use the following commands:

```bash
cargo test test_first -- --nocapture
cargo test test_second -- --nocapture
