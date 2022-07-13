# subnetcalc
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT) ![checks](https://github.com/delhoyo31415/subnetcalc/actions/workflows/ci.yml/badge.svg)

`subnetcalc` is a simple program which helps you with your subnetting exercises. 

It takes an address block and divides it into the required subnetworks using the specified strategy. For more information, type `subnetcalc --help`

## Examples
If you have the address block `201.70.64.0/24` and you want to divide into 10 subnetworks with equal size, you would type (note that the symbol `$` is used to indicate that the line which begins with that symbol is a shell command)
```
$ subnetcalc 201.70.64.0/24 --flsm 10

1) 201.70.64.0/28
2) 201.70.64.16/28
3) 201.70.64.32/28
4) 201.70.64.48/28
5) 201.70.64.64/28
6) 201.70.64.80/28
7) 201.70.64.96/28
8) 201.70.64.112/28
9) 201.70.64.128/28
10) 201.70.64.144/28
```
If instead you wanted to divide a network into subnetworks in which the number of hosts each one must have is different, you would need to use VLSM. Specifically, if you have the block `20.30.0.0/18` and you want subnetworks of `1000` `5000` `2000` and `1000` hosts, you would type
```
$ 20.30.0.0/18 --vlsm 1000 5000 2000 100

1) 5000 - 20.30.0.0/19
2) 2000 - 20.30.32.0/21
3) 1000 - 20.30.40.0/22
4) 1000 - 20.30.44.0/22
```

## Build
You need to first install Rust on your system. Then type the following:
```
$ git clone https://github.com/delhoyo31415/subnetcalc
$ cd subnetcalc
$ cargo build --release
```
The executable will be located in `target/release/subnetcalc`

## Final notes
The main purpose of this project was getting familiarized with the Rust programming language so it is likely some parts can be improved. I am aware of the existance of crates like [clap](https://crates.io/crates/clap) for argument parsing or [anyhow](https://crates.io/crates/anyhow) and [thiserror](https://crates.io/crates/thiserror) but, for this simple project, I wanted to make everything from scratch.

Every commit is licensed under the MIT license even though the text with its content does not appear in some of them.