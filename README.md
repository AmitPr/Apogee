# Apogee
Apogee is a WIP serverless platform built utilizing WebAssembly Components. This monorepo contains the prototype implementation. Apogee is a personal project and is currently not ready (and may not ever be ready) for production use.

## Crates
This repository contains several crates which come together to form the entire platform. Namely:

| Crate | Description |
| --- | --- |
| `apogee-bindings` | Contains generated bindings from `*.wit` files. |
| `apogee-sdk` | SDK for building WebAssembly Components. This is used within guests. |
| `apogee-macros` | Macro definitions that are exposed by the SDK. |
| `apogee-host` | Host runtime for running WebAssembly Components. |
| `wasmtime-wasi-host` | Contains the bindings for WASI interfaces. |