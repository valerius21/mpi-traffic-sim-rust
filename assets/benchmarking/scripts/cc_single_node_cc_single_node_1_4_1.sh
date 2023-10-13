#!/bin/bash
#SBATCH -p medium
#SBATCH -n 4
#SBATCH -N 1-1
#SBATCH -t 120:00
#SBATCH -o /usr/users/hpctraining57/mpi-traffic-sim-rust/cc_single_node_1_4_1.out
        
# warmup
for ((i=0; i<=2; i+=1)); do
    srun -N 1-1         -n 4 /usr/users/hpctraining57/mpi-traffic-sim-rust/target/cc/traffic-sim graph-parts         -p multi-threaded         -n 1 /usr/users/hpctraining57/mpi-traffic-sim-rust/assets/graph.json
done

# run
for ((i=0; i<=10; i+=1)); do
    srun -N 1-1         -n 4 /usr/users/hpctraining57/mpi-traffic-sim-rust/target/cc/traffic-sim graph-parts         -p multi-threaded         -n 1 /usr/users/hpctraining57/mpi-traffic-sim-rust/assets/graph.json
done
        