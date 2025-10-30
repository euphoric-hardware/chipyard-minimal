# Chipyard on ARM-Mac

This documentation provides a step by step tutorial on how to run chipyard stuff on a ARM based Mac machine.

## Install dependencies

- firtool-1.75.0*
- Verilator 5.027
- `java -version`: openjdk version "17.0.17" 2025-10-21 (probably works with off the shelf java versions)

## Compiling riscv-gnu toolchains from source

1. [Create a disk volume that is case sensitive](https://brianboyko.medium.com/a-case-sensitive-src-folder-for-mac-programmers-176cc82a3830)

- Lets say the volume is in `/Volumes/sourcecode`

```bash
mkdir riscv
export RISCV=/Volumes/sourcecode/riscv

(or put this in your .bashrc/.zshrc)
```

2. Clone the repo

```bash
git clone https://github.com/riscv/riscv-gnu-toolchain
cd riscv-gnu-toolchain
git submodule update --init --recursive
```

3. Apply the zlib patch

- [scott riscv gnu homebrew](https://github.com/riscv-software-src/homebrew-riscv/blob/main/zlib-fix.patch)

4. Configure & compile

```bash
mkdir build && cd build
../configure --prefix=$RISCV --disable-gdb --enable-multilib --with-cmodel=medany
gmake -j8
```


## Clone Chipyard

```bash
git clone git@github.com:ucb-bar/chipyard.git
cd chipyard
git checkout chipyard-minimal-macos

./scripts/build-toolchain-extra.sh riscv-tools -p $RISCV
```


## Running sims

- Assuming verilator is in your path

```bash
export TMPDIR=/tmp
export JAVA_TOOL_OPTIONS="-Djava.io.tmpdir=/tmp"
make run-binary-debug -j8 BINARY=$RISCV/riscv64-unknown-elf/share/riscv-tests/benchmarks/towers.riscv CONFIG=RocketConfig
```

Why the `export JAVA_TOOL_OPTIONS="-Djava.io.tmpdir=/tmp"` (*GPT-generated response*)??

- Root cause: sbt’s JNI Unix-domain socket binder (`JNIUnixDomainSocketLibraryProvider_bindNative`) tried to memcpy a socket path longer than the OS buffer, and macOS’s checked libc aborted with `__chk_fail_overflow`. On macOS, `struct sockaddr_un { … char sun_path[104]; }` → the maximum path length is 103 chars plus NUL. Your stack shows the abort from `__memcpy_chk`, exactly what happens when the socket pathname >103 bytes.
- Why this happens with sbt on macOS:
    - sbt server puts its socket under `java.io.tmpdir` (usually `$TMPDIR`, e.g. `/var/folders/.../T/`), then appends a long sbt-… subdir and filename.
    - macOS `$TMPDIR` paths are often 45–70 chars by themselves. Add sbt’s suffix (~50–70 chars) → total can exceed 103.
    - You’re forcing server mode with `-Dsbt.server.forcestart=true`, so sbt always tries to create that UDS path and trips the overflow.
