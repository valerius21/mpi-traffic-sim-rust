#!/bin/bash

RUNS=100
WARUMPS=5
VEHICLES=(1 10 100 1000 10000)
WORKER_THREADS=(1 2 4 8 16 $(nproc))
NODES=(2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19)

for N_V in "${VEHICLES[@]}"; do
	for N_T in "${WORKER_THREADS[@]}"; do
		for N_N in "${NODES[@]}"; do
			FILE_NAME="$N_V"_"$N_T"_"$N_N"
			MPI_RUN="mpirun -np $N_N -env TOKIO_WORKER_THREADS=$N_T ./target/release/traffic-sim graph-parts -n $N_V -p multi-threaded --mpi -l info -t tokio assets/graph.json >> ./benchmark/$FILE_NAME.log"
			hyperfine --warmup=$WARUMPS --min-runs=$RUNS --max-runs=$RUNS --time-unit=millisecond --export-csv=./benchmark/$FILE_NAME.csv "$MPI_RUN"
			break
		done
		break
	done
	break
done
