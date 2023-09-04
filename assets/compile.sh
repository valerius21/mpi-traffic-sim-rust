#!/bin/bash

echo "[prepare hpc] Loading modules..."
module load spack-user
source $SPACK_USER_ROOT/share/spack/setup-env.sh
spack load llvm
module load rev/11.06
module load mpich/ge/gcc/64
echo "[prepare hpc] Done."

cd ~/mpi-traffic-sim-rust &&
	rm -rf target &&
	cargo build --release --features complex-calculation &&
	mv -v target/release target/cc &&
	cargo build --release
