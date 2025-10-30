# Minimal Chipyard

This is a lightweight chipyard.
No more conda, submodules, CI, irrelevant accelerator IP, and garbage software tests.
This repo expects that you install and manage the dependencies on your own.
The dependencies are listed in the [install dependencies section of this document](#install-dependencies)

## Install Dependencies

- CIRCT firtool
    - You can download the [releases](https://github.com/llvm/circt/releases) or [compile it from source](https://github.com/llvm/circt?tab=readme-ov-file#getting-started)
    - Tested with version 1.75.0
- Verilator
    - [Installation docs](https://verilator.org/guide/latest/install.html)
    - Tested with 5.027
- Java
    - Tested with openjdk version "17.0.17" but should work with most versions
- `riscv-gnu-toolchain`
    - Linux: [follow the instructions here](https://github.com/riscv-collab/riscv-gnu-toolchain?tab=readme-ov-file#installation-linux)
    - Macos: [detailed step by step instructions here](#compiling-the-riscv-toolchain-from-source)
- Cargo
    - This is for compiling some rust baremetal benchmarks
    - [Install Rust and Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html)

## Clone and Setup Chipyard

This step assumes that you have compiled the `riscv-gnu-toolchain` and set the `RISCV` environment variable to the installation path.
This compiles some software such as spike, pk, libgloss, spike-devices, and DRAMSim.

```bash
git clone git@github.com:euphoric-hardware/chipyard-minimal.git
cd chipyard-minimal

./scripts/build-toolchain-extra.sh riscv-tools -p $RISCV
./scripts/repo-clean.sh
```

## Running Sims

If you are on Linux, you can skip this part.
If you are on MacOS, you have to set the java tmpdir path (or you can add this to your `.zshrc` or `.bashrc`):

```bash
export TMPDIR=/tmp
export JAVA_TOOL_OPTIONS="-Djava.io.tmpdir=$TMPDIR"
```

Why the `export JAVA_TOOL_OPTIONS="-Djava.io.tmpdir=/tmp"`?? (*Edited from a GPT-generated response*)

- sbt's JNI Unix-domain socket binder (`JNIUnixDomainSocketLibraryProvider_bindNative`) tried to memcpy a socket path longer than the OS buffer, and macOS’s checked libc aborted with `__chk_fail_overflow`
- On macOS, `struct sockaddr_un { … char sun_path[104]; }` which means the maximum path length is **103** chars plus NULL
- When sbt runs on macOS:
    - sbt server puts its socket under `java.io.tmpdir` (usually `$TMPDIR`, e.g. `/var/folders/.../T/`), then appends a long `sbt-...` subdir and filename
    - macOS `$TMPDIR` paths are often 45–70 chars by themselves and adding sbt suffix can lead to a longer path length than **103**
    - You’re forcing server mode with `-Dsbt.server.forcestart=true`, so sbt always tries to create that UDS path and trips the overflow

- Assuming verilator is in your path, run:

```bash
cd sims/verilator
make run-binary-debug -j8 BINARY=$RISCV/riscv64-unknown-elf/share/riscv-tests/benchmarks/towers.riscv CONFIG=RocketConfig
```


## Compiling Baremetal Workloads

We have some baremetal benchmarks written in rust in [baremetal-rust-riscv](https://github.com/euphoric-hardware/chipyard-minimal/tree/main-minimal/software/baremetal-rust-riscv).
You can simply run `cargo build --release` and it will compile the benchmarks.
The benchmarks will be located in `target/riscv64gc-unknown-none-elf/release`.

## Have Fun Hacking!

This sets up most of the functionality that is required when developing new IP.
What you want is to setup a new repo quickly, develop/change your IP without having to recursively bump submodules, quickly write baremetal benchmarks, and run RTL simulations.
Most of the time, you don't need to boot Linux using FPGA based simulations or run through VLSI flows.
Hopefully this repo provides you with a better IP development experience vs the upstream Chipyard.

---

## Compiling the RISCV Toolchain from Source

1. The RISCV toolchain requires that the filesystem is case sensitive. The default mac filesystem is case insensitive but we can create a disk volume that is case sensitive

- [Create a case sensitive volume](https://brianboyko.medium.com/a-case-sensitive-src-folder-for-mac-programmers-176cc82a3830)
- Lets say the created volume is in `/Volumes/sourcecode`

```bash
mkdir riscv # we will install the compiled outputs here
export RISCV=/Volumes/sourcecode/riscv
(or put this in your .bashrc/.zshrc)
```

2. Clone the repo

```bash
git clone https://github.com/riscv/riscv-gnu-toolchain
cd riscv-gnu-toolchain
git submodule update --init --recursive
```

3. Apply the [zlib patch](https://github.com/riscv-software-src/homebrew-riscv/blob/main/zlib-fix.patch)

```bash
wget <link to zlib-fix.patch>
git apply zlib-fix.patch
```

4. Configure & compile. This will install the compiled outputs to the `$RISCV` directory

```bash
mkdir build && cd build
../configure --prefix=$RISCV --disable-gdb --enable-multilib --with-cmodel=medany
gmake -j8
```
