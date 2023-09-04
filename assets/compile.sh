#!/bin/bash

module load spack-user
source $SPACK_USER_ROOT/share/spack/setup-env.sh
spack load llvm
module load rev/11.06
module load mpich/gc/gcc/64

cd ~/mpi-traffic-sim-rust &&
	rm -rf target &&
	cargo build --release --features complex-computing &&
	mv -rv target/release target/cc &&
	cargo build --release