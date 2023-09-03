RUNS = 100
WARM_UPS = 5
TIMEOUT = '15:00'
VEHICLES=[1, 10, 100, 1000, 10000]
WORKER_THREADS=[2, 4, 8, 16, 32]
NODES=[2,3,4,5,10,15,19]
BASE_DIR = '/usr/users/hpctraining57/mpi-traffic-sim-rust'

# TODO: create optimized worker threads combinations

for vehicle_load in VEHICLES:
    for worker_number in WORKER_THREADS:
        for node_number in NODES:
            if worker_number < node_number:
                continue
            FILE_NAME = f'{node_number}_{worker_number}_{vehicle_load}'
            head_template = f'''#!/bin/bash
#SBATCH -p medium
#SBATCH -n {worker_number}
#SBATCH -N {node_number}-{node_number}
#SBATCH -t {TIMEOUT}
#SBATCH -o {BASE_DIR}/{FILE_NAME}.out
            '''
            tmpl = head_template + f'''
# warmup
for ((i=0; i<={WARM_UPS}; i+=1))
do
  srun -N {node_number}-{node_number} -n {worker_number} {BASE_DIR}/target/release/traffic-sim graph-parts -n {vehicle_load} --mpi -p multi-threaded {BASE_DIR}/assets/graph.json
done

# run
for ((i=0; i<={RUNS}; i+=1))
do
  srun -N {node_number}-{node_number} -n {worker_number} {BASE_DIR}/target/release/traffic-sim graph-parts -n {vehicle_load} --mpi -p multi-threaded {BASE_DIR}/assets/graph.json
done
            '''

            with open(f'./{FILE_NAME}.sh', 'w', encoding='utf8') as f:
                f.write(tmpl)
                print('=====',FILE_NAME,'=====')

