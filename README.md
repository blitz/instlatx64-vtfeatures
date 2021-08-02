# Intel VT Feature Parsing

This repo contains a tiny tool to parse CPUID dumps from
[InstLatx64](https://github.com/InstLatx64/InstLatx64).

This tool is somewhat work-in-progress.

## Building

After [installing Rust](https://www.rust-lang.org/tools/install), this
tool can be built using:

```
% cargo build
```

## Usage

```
# Somewhere else...
% git clone https://github.com/InstLatx64/InstLatx64

# In this repository
% cargo run < $PATH_TO_INSTLATX64_REPO/GenuineIntel/GenuineIntel00406C3_Braswell_CPUID.txt
EPT                         : Y
Unrestricted Guest          : Y
VMCS Shadowing              : N
APIC-register virtualization: N
Virtual-interrupt delivery  : N
VMX Preemption Timer        : Y
Process posted interrupts   : N

```
