RUNS = 10
WARM_UPS = 2
TIMEOUT = '120:00'
VEHICLES = [1, 10, 100, 1000, 10000]
NODES = [1]
BASE_DIR = '/usr/users/hpctraining57/mpi-traffic-sim-rust'
WORKERS = [2, 4, 8, 16, 24]

for vehicle_load in VEHICLES:
    for node_number in NODES:
        for worker_number in WORKERS:
            FILE_NAME = f'cc_single_node_{node_number}_{worker_number}_{vehicle_load}'
            head_template = f'''#!/bin/bash
#SBATCH -p medium
#SBATCH -n {worker_number}
#SBATCH -N {node_number}-{node_number}
#SBATCH -t {TIMEOUT}
#SBATCH -o {BASE_DIR}/{FILE_NAME}.out
        '''
            tmpl = head_template + f'''
# warmup
for ((i=0; i<={WARM_UPS}; i+=1)); do
    srun -N {node_number}-{node_number} \
        -n {worker_number} {BASE_DIR}/target/cc/traffic-sim graph-parts \
        -p multi-threaded \
        -n {vehicle_load} {BASE_DIR}/assets/graph.json
done

# run
for ((i=0; i<={RUNS}; i+=1)); do
    srun -N {node_number}-{node_number} \
        -n {worker_number} {BASE_DIR}/target/cc/traffic-sim graph-parts \
        -p multi-threaded \
        -n {vehicle_load} {BASE_DIR}/assets/graph.json
done
        '''

            with open(f'./cc_single_node_{FILE_NAME}.sh', 'w', encoding='utf8') as f:
                f.write(tmpl)
                print('=====', FILE_NAME, '=====')

