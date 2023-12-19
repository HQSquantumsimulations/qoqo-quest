<img src="qoqo_Logo_vertical_color.png" alt="qoqo logo" width="300" />

# qoqo-quest

[QuEST](https://github.com/QuEST-Kit/QuEST) simulator backend for the qoqo/roqoqo quantum toolkit by [HQS Quantum Simulations](https://quantumsimulations.de).

This repository contains three components:

* The qoqo_quest backend to simulate quantum programms from the qoqo python interface to roqoqo,
* The roqoqo-quest backend backend to simulate quantum programms from roqoqo directly,
* The quest-sys crate providing rust bindings for the QuEST C library.

## qoqo-quest


[![PyPI](https://img.shields.io/pypi/v/qoqo-quest)](https://pypi.org/project/qoqo-quest/)
[![PyPI - Format](https://img.shields.io/pypi/format/qoqo-quest)](https://pypi.org/project/qoqo-quest/)
![Crates.io](https://img.shields.io/crates/l/roqoqo-quest)
<!-- [![GitHub Workflow Status](https://github.com/HQSquantumsimulations/qoqo-quest/workflows/qoqo-quest-ci-tests/badge.svg)](https://github.com/HQSquantumsimulations/qoqo-quest/actions/) -->

[QuEST](https://github.com/QuEST-Kit/QuEST) based simulator backend for the qoqo quantum toolkit by [HQS Quantum Simulations](https://quantumsimulations.de).

qoqo-quest allows to simulate the execution of qoqo quantum circuits with the help of the QuEST quantum simulator.
Based on QuEST qoqo supports the simulation of error-free and noisy quantum computers.
qoqo-quest is designed to be able to simulate all operations that are part of qoqo.
For usage examples see the examples section of [qoqo](https://github.com/HQSquantumsimulations/qoqo/)

### Installation

For linux and macos and windows on x86_64 hardware and macos on arm64 pre-built Python packages are available on PyPi and can be installed with

```shell
pip install qoqo-quest
```

For other platforms please use the source distribution that requires a Rust install with a rust version > 1.47 and a maturin version in order to be built.

After installing Rust (for example via [rustup](ghcr.io/rust-cross/manylinux2014-cross:aarch64))

run the following

```shell
pip install maturin
pip install qoqo-quest
```

## roqoqo-quest

[![Crates.io](https://img.shields.io/crates/v/roqoqo-quest)](https://crates.io/crates/roqoqo-quest)
[![docs.rs](https://img.shields.io/docsrs/roqoqo-quest)](https://docs.rs/roqoqo-quest/)
![Crates.io](https://img.shields.io/crates/l/roqoqo-quest)
<!-- [![GitHub Workflow Status](https://github.com/HQSquantumsimulations/qoqo-quest/workflows/qoqo-quest-ci-tests/badge.svg)](https://github.com/HQSquantumsimulations/qoqo-quest/actions) -->

[QuEST](https://github.com/QuEST-Kit/QuEST) based simulator backend for the roqoqo quantum toolkit by [HQS Quantum Simulations](https://quantumsimulations.de).

roqoqo-quest allows to simulate the execution of roqoqo quantum circuits directly from rust code with the help of the QuEST quantum simulator.
roqoqo-quest is designed to be able to simulate all operations that are part of roqoqo.
For usage examples see the examples section of [roqoqo](https://github.com/HQSquantumsimulations/qoqo/).

## QuEST build options

QuEST supports distributed computing and the use of GPU computing. `qoqo-quest` and `roqoqo-quest` are not tested against distributed builds, but have preliminary support for `GPU` computation. The `PyPi` distributed versions do not support GPU computation. For the moment GPU support can be enabled for NVIDIA GPUs using either the `cuda` feature or the `cuquantum` feature when compiling  `qoqo-quest` with `maturin` or `roqoqo-quest` as a rust library. The `cuda` feature uses QuEST's CUDA-based simulator implementation and needs the cuda-compiler (nvcc) available during build. The `cuquantum` feature uses the cuda quantum simulator and requires the cuda-compiler (nvcc) during build as well as the cuquantum libraries.

## General Notes

This software is still in the beta stage. Functions and documentation are not yet complete and breaking changes can occur.

This project has been partly supported by [PlanQK](https://planqk.de) and is partially supported by [QSolid](https://www.q-solid.de/).


## Contributing

We welcome contributions to the project. If you want to contribute code, please have a look at CONTRIBUTE.md for our code contribution guidelines.
