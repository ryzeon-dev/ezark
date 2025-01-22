# EZARK <img src="https://img.shields.io/badge/version-1.0.0-16a085" /> <img src="https://img.shields.io/badge/Rust-585858?logo=rust&logoColor=16a085&style=flat" />

- Easy Archiver
- Open source archive utility written in Rust

## Install
- from `cargo`
```
cargo install ezark
```
- from `git`
```
git clone https://github.com/ryzeon-dev/ezark 
cd ezark
make 
sudo make install
```
- `git` repository contains compiled executables, for linux_amd64, linux_arm64, win_x86_64

## Usage
- verbose operation can be obtained using `-v` or `--verbose` flag
- create an archive 
```
ezark --make archive_name.eza files_and_dirs_list
```
- extract an archive
  - if no extraction path is given, the extraction will be executed in current path
```
ezark --extract archive_name.eza extraction_path
```
- inspect archive
```
ezark --inspect archive_name.eza
```
- check version
```
ezark --version
```
- to get help, use the `-h` or `--help` flag
```
ezark --help
```