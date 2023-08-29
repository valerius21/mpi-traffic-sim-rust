# Traffic Simulation Project for HPC

## Build

- Make sure mpich is installed
- install rustup then `cargo build --release`

## Run

- `mpirun -n 4 ./target/release/traffic-sim graph-parts -n 100 --mpi -t tokio -p multi-threaded assets/graph.json`

## Benchmark

TODO

## How to get the `graph.json`

TODO
