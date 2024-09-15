# The Megu Project

<style>
img[src*='#center'] { 
   display: block;
   margin: auto;
}
</style>

![Logo](docs/logo/megu.png#center)

## What's This?
* It contains `megu`, a programming language

* It contains `mokey`, a multilingual infrastructure

* It contains `noda`, an alternative implementation of `MLIR` written for `mokey`

* It is currently written in `Rust` and `Zig`

* It contains a number of libraries that underpin the three projects

* **It is a pleasure to be starred**

## Directory Structure
```
/
├─ megu
│  ├─ "Write Simply, Run Quickly"
│  └─ language: zig
├─ mevil
│  ├─ Build Mokey Projects
│  └─ language: rust
├─ mokey
│  ├─ High Level IR System written by noda
│  └─ language: zig
├─ noda
│  ├─ Yet Another MLIR Implment for programing language
│  └─ language: rust(main) zig(pass&dialect)
└─ bugi 
   ├─ Plugin system for noda
   └─ language: rust
```