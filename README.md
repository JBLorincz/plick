# plick
A PL/1 Compiler for Microcomputers (Micro PL/1).

Plick is a LLVM Frontend Compiler for a very much work-in-progress dialect of [PL/1](https://en.wikipedia.org/wiki/PL/I) called Micro PL/1.


# Install

## Dependencies
1. LLVM version 14.0
2. A C compiler for linking (tested to work with both GCC and Clang)




## Windows Setup

To make a working copy of plick from source, you will require a copy of LLVM 14.0. Plick's tests also requires C/C++ development tooling, such as MSVC or Mingw-w64. Because Rust also uses LLVM you already have one of these - by default, MSVC.


- Follow [these instructions](https://llvm.org/docs/CMake.html) to build LLVM from source and install on your machine.
- Set the LLVM_SYS_140_PREFIX environment variable to the path of your built LLVM root to get the [llvm-sys](https://lib.rs/crates/llvm-sys) dependency working.

Once LLVM is set up, plick should be able to run. Note that by default the outputted object files will share the target of your current Rust toolchain. For most people on Windows, that means the object files will be for the x86_64-pc-windows-msvc target. 

**NOTE**: `cargo test` on the MSVC toolchain assumes command-line access to Microsoft's `cl` compiler. The easiest way to provide access is to run `cargo test` from a Visual Studio Developer Command Prompt.

- Compile a pli file from the project with `cargo run -- <PATH TO FILE HERE>`

- Link the file to create the executable with the following command: `cl <file name here> /link msvcrt.lib legacy_stdio_definitions.lib`

- You should now have an executable ready to run


