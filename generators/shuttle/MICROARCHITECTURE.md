# Shuttle Microarchitecture Documentation

## Overview

Shuttle is a Rocket-based superscalar in-order RISC-V core designed as a demonstrative example of another CPU design point. It supports the RV64IMAFDCB instruction set with supervisor and user modes.

**Key Characteristics:**
- **Pipeline**: 7-stage in-order pipeline
- **Issue Width**: Configurable dual, three, or quad-issue (dual-issue is the most sensible design point)
- **ISA**: RV64IMAFDCB (Base, Multiply/Divide, Atomics, Single/Double-precision Floating-Point, Compressed, Bit-manipulation)
- **Privilege Modes**: Machine, Supervisor, and User modes
- **Vector Support**: Optional RVV (RISC-V Vector) extension support
- **Status**: Tape-out proven with similar physical design complexity as Rocket

## Design Philosophy

Shuttle presents the most advantages for:
1. **Floating-point kernels** - scalar control code can execute concurrently with FP instructions
2. **RoCC accelerator kernels** - maintaining high utilization of accelerator units

The superscalar design allows scalar control code to execute in parallel with floating-point or RoCC instructions, maintaining high utilization of specialized units.

## Pipeline Architecture

### 7-Stage Pipeline Overview

```
F0 (NPC) ? F1 (ICache) ? F2 (Decode/Align) ? RRD (Register Read/Dispatch) ? EX (Execute) ? MEM (Memory) ? COM (Commit)
```

Additional writeback stage (WB) for long-latency operations.

### Pipeline Stages in Detail

#### 1. **F0 - Next PC Selection**
- **Purpose**: Determine the next fetch address
- **Operations**:
  - Branch prediction using BTB (Branch Target Buffer) and BHT (Branch History Table)
  - RAS (Return Address Stack) management for function calls/returns
  - Redirect handling from backend
  - Default to PC+fetchBytes for sequential fetches

#### 2. **F1 - Instruction Cache Access**
- **Purpose**: Translate virtual address and access instruction cache
- **Operations**:
  - TLB (Translation Lookaside Buffer) lookup for virtual-to-physical address translation
  - I-Cache tag check
  - BTB access for branch prediction
  - Handle TLB misses and page faults
  - BHT advance for conditional branch prediction

#### 3. **F2 - Instruction Alignment and Decode**
- **Purpose**: Align and expand compressed instructions, perform initial decode
- **Operations**:
  - Handle instruction alignment (RISC-V allows 16-bit aligned instructions)
  - Expand RVC (compressed) instructions to 32-bit format
  - Handle edge cases (instructions spanning cache lines)
  - Update RAS for calls/returns
  - Generate fetch bundle with up to `fetchWidth` instructions
  - Manage half-instruction tracking across fetch boundaries

#### 4. **RRD - Register Read and Dispatch**
- **Purpose**: Decode instructions, read operands, and dispatch to execution units
- **Operations**:
  - Full instruction decode for all issue slots
  - Detect structural hazards (only one memory operation, one branch, etc.)
  - Detect data hazards (RAW, WAW, WAR)
  - Register file read with bypass network
  - CSR (Control and Status Register) operations
  - Scoreboard checking for both integer and floating-point registers
  - **Dispatch Rules**:
    - Only one memory operation per cycle
    - Only one branch/jump per cycle
    - Pipe0-only instructions: system instructions, fences, AMOs, CSR ops, multiply/divide, FP ops, vector ops
    - Instructions stall if operands not ready
    - Support for mem-ALU and late-ALU forwarding paths

**Key Features**:
- **MemALU**: On dual-issue, pipe1 can use results from pipe0's ALU in the MEM stage
- **LateALU**: On dual-issue, pipe1 can use results from pipe0's memory load in the COM stage
- **SFB (Shallow Fetch Buffer) optimizations**: Special handling for tight branch loops

#### 5. **EX - Execute**
- **Purpose**: Execute ALU operations, generate addresses, issue memory/FP operations
- **Operations**:
  - **Integer ALU**: All issue slots have dedicated ALUs
  - **Address Generation**: For memory operations
  - **Branch Resolution**: Determine taken/not-taken, calculate target
  - **FP Pipeline Start**: FP operations begin 4-stage pipeline
  - **Multiply Start**: 2-cycle pipelined multiplier
  - **Divide/DIVSQRT Start**: Iterative divider
  - **Memory Request**: Send request to D-Cache
  - **Vector Dispatch**: Issue vector operations to vector unit
  - **RoCC Dispatch**: Issue RoCC accelerator commands
  - **VConfig**: Handle vector configuration instructions (vsetvl)

**Bypass Forwarding**: Results can be bypassed to RRD stage

#### 6. **MEM - Memory Access**
- **Purpose**: Access data cache, complete TLB translation, handle branches
- **Operations**:
  - **D-TLB Translation**: Virtual-to-physical address translation for loads/stores
  - **D-Cache Access**: Tag check and data array read (if hit)
  - **Branch Misprediction Resolution**: Update BTB, BHT, RAS
  - **Exception Detection**: TLB miss, page faults, access faults
  - **Store Buffer**: Stores complete here on hit
  - **MSHR Allocation**: On cache miss
  - **MemALU Execution**: Dependent ALU operations execute here
  - **FP Store Data**: Floating-point store data prepared

**Replay Mechanism**: Memory operations that miss replay through MEM stage

#### 7. **COM - Commit**
- **Purpose**: Commit architectural state, handle exceptions, retire instructions
- **Operations**:
  - **Exception Handling**: Precise exceptions, redirect to trap handler
  - **CSR Updates**: Write CSR registers
  - **Integer Writeback**: Write integer results to register file
  - **FP Writeback**: Write FP results (from FP pipeline)
  - **Divide Results**: Integer divide completion
  - **Memory Load Complete**: On cache hit
  - **Retirement**: Up to `retireWidth` instructions retire per cycle (in-order)
  - **LateALU Execution**: Final ALU stage for pipe1 instructions dependent on pipe0 loads
  - **Fence/SFENCE**: Complete ordering operations
  - **Vector Scalar Check**: Verify no memory conflicts with vector operations

**Replay on**:
- Cache miss (reissue from MSHR)
- Structural hazards
- Exceptional conditions

#### 8. **WB - Writeback (Long-latency)**
- **Purpose**: Handle completions of long-latency operations
- **Operations**:
  - **Memory Load Completion**: Cache refill data writeback
  - **Divide Completion**: Integer divide results
  - **RoCC Responses**: Accelerator writeback
  - **Vector Responses**: Vector operation results
  - **FP DivSqrt**: Floating-point divide/square root completion

## Execution Units

### Integer Execution

#### ALUs
- **Count**: One per issue slot (e.g., 2 ALUs for dual-issue)
- **Operations**: ADD, SUB, AND, OR, XOR, shifts, compares
- **Latency**: 1 cycle
- **Extensions**: Zba (address generation), Zbb (basic bit manipulation), Zbs (single-bit operations)

#### Multiply Unit
- **Type**: Pipelined multiplier
- **Latency**: 2 cycles
- **Operations**: MUL, MULH, MULHSU, MULHU, MULW

#### Divide Unit
- **Type**: Iterative divider (MulDiv module)
- **Latency**: Variable (early-out optimization)
- **Operations**: DIV, DIVU, REM, REMU, DIVW, REMW
- **Throughput**: Not fully pipelined

### Floating-Point Execution

#### FP Pipeline (ShuttleFPPipe)
- **Latency**: 4 cycles for FMA (configurable)
- **Units**:
  - **DFMA**: Double-precision FMA (fused multiply-add)
  - **SFMA**: Single-precision FMA
  - **HFMA**: Half-precision FMA
  - **IntToFP**: Integer to FP conversion
  - **FPToInt**: FP to integer conversion
  - **FPToFP**: FP conversion and move operations

#### FP Divide/Square Root
- **Type**: Iterative DivSqrtRecFN_small module
- **Latency**: Variable (10-20+ cycles depending on precision)
- **Operations**: FDIV, FSQRT for H/S/D precision
- **Throughput**: Not fully pipelined (one operation at a time)

### Memory Unit

#### Load/Store Unit
- **Capability**: One memory operation per cycle
- **Operations**: Load, Store, AMO (Atomic Memory Operations)
- **Sizes**: Byte, halfword, word, doubleword
- **AMO Support**: LR/SC (Load-Reserved/Store-Conditional), AMO operations

### Vector Unit (Optional)
- **Interface**: Decoupled vector unit interface
- **Operations**: RVV 1.0 instructions
- **Memory**: Independent memory path to D-Cache
- **Configuration**: Configurable VLEN (vector length)

### RoCC Interface
- **Purpose**: Custom accelerator integration
- **Features**: Standard Rocket Custom Coprocessor interface
- **Concurrent Execution**: Can execute while scalar instructions proceed

## Memory Subsystem

### Instruction Cache (ICache)

**Configuration** (default):
- **Size**: 8KB (64 sets ? 8 ways ? 64B blocks)
- **Associativity**: 8-way set associative
- **Line Size**: 64 bytes
- **Policy**: LRU replacement (pseudo-LRU)
- **Features**: 
  - Prefetching support
  - Single-ported
  - TileLink interface

**Fetch Width**: 
- Configurable: 4, 8, 16, or 32 bytes per cycle
- Default: 8 bytes (4 ? 16-bit slots, 2-4 instructions accounting for RVC)

**I-TLB**:
- **Sets**: 32 (default)
- **Ways**: 8
- **Page Size Support**: 4KB, 2MB, 1GB (Sv39)

### Data Cache (DCache)

**Configuration** (default):
- **Size**: 16KB (64 sets ? 4 ways ? 64B blocks)
- **Associativity**: 4-way set associative
- **Line Size**: 64 bytes
- **Policy**: Configurable replacement (default: random)
- **Banks**: 4 data banks, 4 tag banks
- **Port**: Single-ported for area efficiency

**MSHRs (Miss Status Holding Registers)**:
- **Count**: 4 (configurable)
- **Purpose**: Track outstanding cache misses
- **Features**: 
  - Coalesce multiple misses to same line
  - Secondary miss support
  - Replay mechanism for dependent operations

**I/O MSHRs**:
- **Count**: 1 (configurable)
- **Purpose**: Handle uncached/MMIO accesses

**D-TLB**:
- **Sets**: 32 (default)
- **Ways**: 8
- **Ports**: Up to 2 (scalar + vector)

**Write Buffer**:
- Multiple writeback units (configurable, default: 2)
- Handle evictions and voluntary writebacks

**Features**:
- Store-to-load forwarding
- Non-blocking loads (with MSHRs)
- Multi-banking for improved bandwidth
- Atomic operation support (AMOs, LR/SC)
- PMA (Physical Memory Attributes) checking

### Page Table Walker (PTW)

- **Ports**: 2 base + RoCC ports + vector ports
- **Page Tables**: Sv39 (3-level page table)
- **Caching**: L2 TLB with 512 entries (configurable)
- **Shared**: Between I-TLB and D-TLB

### Optional Memory Features

#### TCM (Tightly Coupled Memory)
- **Purpose**: Low-latency local memory
- **Size**: Configurable (default: 64KB)
- **Banks**: Configurable (default: 4)
- **Features**: 
  - Replicated per-tile address space
  - No cache coherence

#### SGTCM (Scatter-Gather TCM)
- **Purpose**: Specialized memory for vector operations
- **Size**: Configurable (default: 8KB)
- **Banks**: High bank count (default: 32)
- **Features**:
  - Optimized for strided and indexed accesses
  - Vector unit integration

## Branch Prediction

### Branch Target Buffer (BTB)

**Configuration** (default):
- **Entries**: 32
- **Organization**: Fully associative with pseudo-LRU
- **Features**:
  - Stores target addresses for branches, jumps
  - Predicts taken/not-taken for conditional branches
  - CFI type tracking (branch, call, return, jump)
  - PC-relative target calculation

**BTB Entry**:
- Index match bits
- Target address
- Page tracking for compressed representation
- Branch index within fetch packet
- CFI type

### Branch History Table (BHT)

**Configuration**:
- **Type**: 2-bit saturating counters (default)
- **History**: Global history register
- **Indexing**: XOR of PC and history

### Return Address Stack (RAS)

**Configuration**:
- **Depth**: Configurable (default: 8 entries)
- **Purpose**: Predict return addresses for function returns
- **Updates**: 
  - Push on call (JAL/JALR with rd=x1)
  - Pop on return (JALR with rs1=x1, rd=x0)

### Branch Resolution

- **Stage**: MEM stage
- **Mechanism**: Compare predicted target with actual target
- **Mispredict Action**: Flush frontend pipeline, redirect to correct PC
- **Update**: BTB, BHT, and RAS updated on mispredict

### Special Optimizations

#### SFB (Shallow Fetch Buffer) Branch Optimization
- **Purpose**: Optimize tight loops with a branch and dependent instruction
- **Mechanism**: 
  - Detect branch in pipe0 with dependent instruction in pipe1
  - If branch taken, kill pipe1 instruction that cycle
  - Allows tight 1-cycle loops

## Register Files

### Integer Register File

- **Registers**: 32 ? 64-bit (x0-x31, x0 hardwired to 0)
- **Read Ports**: 2 per issue slot (e.g., 4 read ports for dual-issue)
- **Write Ports**: 1 per issue slot + long-latency port
- **Scoreboard**: Track pending writes to detect hazards

### Floating-Point Register File

- **Registers**: 32 ? 65-bit (f0-f31, includes NaN-boxing bit)
- **Read Ports**: 3 for pipe0 (FMA uses 3 operands)
- **Write Ports**: 1 for short-latency + 1 for long-latency (div/sqrt)
- **Scoreboard**: Track pending FP operations
- **Precision**: Support H (16-bit), S (32-bit), D (64-bit)

### Vector Register File (Optional)

- **Registers**: 32 vector registers
- **Width**: Configurable VLEN (e.g., 128, 256, 512 bits)
- **Elements**: Configurable SEW (8, 16, 32, 64-bit elements)
- **Implementation**: In vector unit module

### CSRs (Control and Status Registers)

**Standard CSRs**:
- Machine-mode CSRs (mstatus, mie, mip, mtvec, mepc, mcause, mtval, etc.)
- Supervisor-mode CSRs (sstatus, sie, sip, stvec, sepc, scause, stval, satp)
- User-mode counters (cycle, time, instret)
- FP CSRs (fcsr, frm, fflags)
- Vector CSRs (vtype, vl, vstart, vxrm, vxsat) [if vector enabled]

**Custom CSRs**:
- MARCHID: 34 (identifies Shuttle core)

## Hazard Detection and Resolution

### Data Hazards

#### RAW (Read-After-Write)
- **Detection**: Compare source registers with destination registers in pipeline
- **Resolution**: Stall or bypass
- **Bypass Network**: 
  - From EX stage (early bypass)
  - From MEM stage (mem bypass)
  - From COM stage (commit bypass)
  - From WB stage (writeback bypass)
  - From long-latency (ll_bypass)

#### WAW (Write-After-Write)
- **Detection**: Compare destination registers
- **Resolution**: Stall younger instruction

#### WAR (Write-After-Read)
- **Detection**: Not an issue in in-order execution
- **Resolution**: N/A

### Structural Hazards

**Detected at RRD stage**:
1. **Memory Unit**: Only one memory operation per cycle
2. **Multiplier**: One multiply operation at a time (pipelined)
3. **Divider**: One divide operation at a time (not pipelined)
4. **FP Unit**: One FP operation per cycle in pipe0
5. **Cache Banks**: Bank conflicts stall
6. **MSHR Full**: Memory operations stall

### Control Hazards

- **Resolution**: Branch prediction (BTB/BHT/RAS)
- **Mispredict**: Flush pipeline, 3-6 cycle penalty depending on stage
- **Mechanism**: Speculative execution with precise exception handling

## Instruction Dispatch Rules

### Pipe Assignment (Dual-Issue Example)

**Pipe 0 (Mandatory)**:
- System instructions (ECALL, EBREAK, MRET, SRET, WFI)
- Fence instructions (FENCE, FENCE.I, SFENCE.VMA)
- CSR instructions
- Multiply/Divide instructions
- Floating-point instructions
- Vector instructions
- RoCC instructions
- AMO (Atomic Memory Operations)
- Branches/Jumps (can be in any pipe, but only one per cycle)
- First instruction after shadowed branch (SFB)
- Non-RVC instruction after shadowed branch

**Pipe 1 (Flexible)**:
- Simple ALU operations
- Load/Store (if no memory op in pipe 0)
- Branches (if no branch in pipe 0)

**Multi-Issue Constraints**:
- At most one memory operation per cycle
- At most one branch per cycle
- At most one vsetvl per cycle (if not issuing to vector unit)
- No same-cycle dependencies unless handled by MemALU/LateALU
- Instructions must be in-order within fetch packet

## Exception and Interrupt Handling

### Exception Types

**Synchronous Exceptions**:
- Instruction access fault (page fault, access exception)
- Illegal instruction
- Breakpoint
- Load/Store address misaligned
- Load/Store access fault
- Load/Store page fault
- Environment call (ECALL)

**Asynchronous Interrupts**:
- Machine-mode interrupts (timer, software, external)
- Supervisor-mode interrupts (timer, software, external)
- RoCC interrupts

### Exception Mechanism

1. **Detection**: Exceptions detected at various stages (F2, RRD, MEM, COM)
2. **Priority**: Earlier exceptions take priority
3. **Commit Stage**: All exceptions handled at COM stage for precise semantics
4. **State Update**:
   - Update xEPC (exception PC)
   - Update xCAUSE (exception cause)
   - Update xTVAL (trap value)
   - Update xSTATUS (save privilege mode, interrupt enable)
5. **Redirect**: Jump to trap vector (xTVEC)
6. **Pipeline**: Flush all in-flight instructions
7. **Atomic**: No instructions retire with or after excepting instruction

### Interrupt Handling

- **Check**: Interrupts checked at RRD stage
- **Priority**: Highest priority interrupt taken
- **Masking**: Controlled by MIE/SIE registers and status.xIE bits
- **Delegation**: Machine mode can delegate to supervisor mode
- **Mechanism**: Similar to synchronous exceptions
- **Return**: MRET/SRET instructions return from trap

## Performance Features

### Bypass Network

**Forwarding Paths**:
1. **EX ? RRD**: ALU results (1 cycle)
2. **MEM ? RRD**: Memory ALU results, load addresses (2 cycles)
3. **COM ? RRD**: Load data (3 cycles)
4. **WB ? RRD**: Long-latency results (4+ cycles)
5. **LL ? RRD**: Last-level writeback

**FP Bypass**: Limited - primarily scoreboard-based stalling

### MemALU Optimization

- **Purpose**: Reduce latency for dependent ALU operations after ALU operations
- **Mechanism**: 
  - Pipe0 ALU result forwarded to Pipe1 in MEM stage
  - Pipe1 can execute dependent ALU op
  - Saves 1 cycle vs. normal bypass
- **Requirement**: Dual-issue or higher

### LateALU Optimization

- **Purpose**: Reduce latency for ALU operations dependent on loads
- **Mechanism**:
  - Pipe0 load result forwarded to Pipe1 in COM stage
  - Pipe1 can execute dependent ALU op in COM/WB stage
  - Saves 1-2 cycles vs. replaying through pipeline
- **Requirement**: Dual-issue or higher, MemALU enabled

### Store-to-Load Forwarding

- **Mechanism**: Stores forward data to younger loads with matching addresses
- **Latency**: Same-cycle to 2-cycle forwarding
- **Granularity**: Byte-level forwarding

### Non-Blocking Cache

- **MSHRs**: Track multiple outstanding misses
- **Secondary Misses**: Coalesce requests to same cache line
- **Replay**: Missed operations replay from MSHR
- **Continued Execution**: Independent instructions continue while cache misses resolve

### Multi-Banking

- **D-Cache**: 4 data banks reduce bank conflicts
- **Improvement**: Allow parallel accesses to different banks
- **Scheduling**: Minimal bank conflict stalling

## Configurability

### Core Parameters (ShuttleCoreParams)

- **retireWidth**: 2, 3, or 4 (default: 2)
- **fetchWidth**: 4, 8, 16, or 32 bytes (default: 8)
- **enableMemALU**: Enable MemALU optimization (default: true)
- **enableLateALU**: Enable LateALU optimization (default: true)
- **debugROB**: Enable debug reorder buffer (default: false)
- **vector**: Optional vector unit configuration
- **enableTraceCoreIngress**: Enable trace ingress (default: false)

### Cache Parameters

**ICache**:
- **nSets**: Number of sets (default: 64)
- **nWays**: Associativity (default: 8)
- **fetchBytes**: Fetch bandwidth (default: 8)

**DCache**:
- **nSets**: Number of sets (default: 64)
- **nWays**: Associativity (default: 4)
- **nMSHRs**: Miss status holding registers (default: 4)
- **nMMIOs**: I/O MSHRs (default: 1)
- **nBanks**: Data banks (default: 4)
- **nTagBanks**: Tag banks (default: 4)

### Branch Prediction Parameters

**BTB**:
- **nEntries**: BTB entries (default: 32)
- **nRAS**: Return address stack depth (default: 8)

**BHT**:
- **counterLength**: History bits (default: 2)

### TLB Parameters

- **nTLBSets**: TLB sets (default: 32)
- **nTLBWays**: TLB ways (default: 8)
- **nL2TLBEntries**: L2 TLB entries (default: 512)
- **nL2TLBWays**: L2 TLB ways (default: 1)

## Debug and Trace Features

### Debug ROB (Reorder Buffer)

- **Purpose**: Debugging support for non-speculative tracing
- **Function**: Track instruction retirement and register writes
- **Configuration**: Optional (debugROB parameter)

### Trace Support

- **Trace Core Ingress**: Instruction trace at commit
- **Trace Encoder**: Compress trace data
- **Trace Sinks**: Multiple trace output destinations
- **Configuration**: Optional (traceParams)

### Performance Counters

- **Standard Counters**: cycle, instret
- **Custom Counters**: Configurable per implementation
- **Access**: Via CSRs in machine/supervisor mode

## Implementation Details

### Technology

- **Language**: Chisel 3 (Scala-embedded HDL)
- **Framework**: Rocket Chip generator framework
- **Interface**: TileLink for on-chip interconnect

### Modularity

**Major Modules**:
- **ShuttleTile**: Top-level tile wrapper
- **ShuttleCore**: Core pipeline
- **ShuttleFrontend**: Fetch pipeline (F0-F2)
- **ShuttleICache**: Instruction cache
- **ShuttleDCache**: Data cache
- **ShuttleBTB**: Branch target buffer
- **ShuttleFPPipe**: Floating-point pipeline
- **ShuttleDTLB**: Data TLB

### Verification

- Status: Tape-out proven
- Complexity: Similar to Rocket core
- Testing: Extensive verification suite (inherited from Rocket)

## Performance Characteristics

### IPC (Instructions Per Cycle)

- **Peak**: Up to `retireWidth` (e.g., 2 for dual-issue)
- **Typical**: 1.2-1.6 for dual-issue on integer workloads
- **FP Workloads**: Higher IPC due to concurrent scalar/FP execution
- **Factors**: Depends on instruction mix, cache hit rate, branch prediction accuracy

### Latencies

**Pipeline**:
- Minimum instruction latency: 4 cycles (RRD ? COM)
- Branch mispredict penalty: 3-6 cycles
- Load-to-use latency: 3 cycles (best case)

**Functional Units**:
- Integer ALU: 1 cycle
- Integer Multiply: 2 cycles
- Integer Divide: 10-40 cycles (variable)
- FP Add/Mul/FMA: 4 cycles
- FP Divide: 10-25 cycles (variable, precision-dependent)
- FP Sqrt: 10-30 cycles (variable, precision-dependent)

**Memory**:
- L1 D-Cache hit: 3 cycles (load-to-use)
- L1 I-Cache hit: 2 cycles (fetch-to-execute)
- L1 miss: 20-50+ cycles (depends on L2/memory)
- TLB hit: 0 cycle (pipelined)
- TLB miss: 10-50+ cycles (page table walk)

### Bandwidth

- **Fetch**: 4-8 bytes per cycle (default: 8 bytes = 2-4 instructions)
- **Retire**: 1-4 instructions per cycle (default: 2 instructions)
- **Memory**: 1 load or store per cycle (64-bit)
- **D-Cache**: 8 bytes per cycle per port

## Comparison with Rocket

### Similarities

- Based on Rocket architecture
- In-order execution
- 5-stage Rocket pipeline extended to 7-stage
- Shared components: PTW, TLB, FPU modules (adapted)
- TileLink interface
- Similar physical design complexity

### Differences

| Feature | Rocket | Shuttle |
|---------|--------|---------|
| **Issue Width** | Single-issue | Dual/Triple/Quad-issue |
| **Pipeline Stages** | 5 | 7 |
| **Superscalar** | No | Yes |
| **Fetch Width** | 4 bytes | 4-32 bytes (default 8) |
| **Retire Width** | 1 | 2-4 (default 2) |
| **MemALU** | No | Yes (optional) |
| **LateALU** | No | Yes (optional) |
| **SFB Optimization** | No | Yes |
| **IPC Potential** | ~1.0 | ~1.2-1.6 |
| **Complexity** | Lower | Higher (but manageable) |
| **Design Target** | General-purpose | FP/Accelerator-heavy workloads |

## Integration

### SoC Integration

- **Interface**: TileLink for memory, MMIO, coherence
- **Multicore**: Supports multiple Shuttle tiles in SMP configuration
- **Coherence**: TileLink coherence protocol support
- **Interrupts**: PLIC (Platform-Level Interrupt Controller) integration
- **Debug**: JTAG debug transport module support

### Software Stack

- **ISA**: RV64IMAFDCB, optionally RV64GCV
- **OS Support**: Linux, FreeBSD, and other RISC-V operating systems
- **Toolchain**: Standard RISC-V GNU toolchain (gcc, llvm)
- **Firmware**: SBI (Supervisor Binary Interface) compatible

## Versioning History

- **1.0**: Initial 6-stage RV64GC release
- **1.1**: Vector unit integration support
- **1.2**: B-extension support (Zba/Zbb/Zbs)
- **2.0**: 7-stage pipeline (current)

## Usage and Configuration Examples

### Dual-Issue Configuration (Default)

```scala
class DualIssueShuttleConfig extends Config(
  new WithNShuttleCores(1, retireWidth=2) ++
  new WithShuttleFetchWidth(8) ++
  // ... other parameters
)
```

### Quad-Issue Configuration

```scala
class QuadIssueShuttleConfig extends Config(
  new WithNShuttleCores(1, retireWidth=4) ++
  new WithShuttleFetchWidth(16) ++
  // ... other parameters
)
```

### With Vector Support

```scala
class ShuttleVectorConfig extends Config(
  new WithNShuttleCores(1) ++
  new WithShuttleVector(vLen=128) ++
  // ... other parameters
)
```

## Conclusion

Shuttle represents a unique point in the RISC-V design space: a superscalar in-order core optimized for floating-point and accelerator workloads. Its 7-stage pipeline with configurable issue width, combined with optimizations like MemALU and LateALU, provides improved performance over single-issue designs while maintaining in-order semantics and manageable complexity.

The core is particularly well-suited for:
- Scientific computing applications with mixed scalar/FP code
- Embedded AI/ML applications with custom accelerators (RoCC)
- Applications requiring predictable performance
- Designs prioritizing area efficiency over maximum performance

Shuttle demonstrates that superscalar in-order designs remain viable for certain application domains, offering a balance between performance and complexity.

---

**Document Version**: 1.0  
**Date**: 2025-10-31  
**Shuttle Version**: 2.0 (7-stage pipeline)
