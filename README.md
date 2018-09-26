# Filecoin Proving Subsystem (FPS)

The **Filecoin Proving Subsystem** provides the storage proofs required by the Filecoin protocol.  It is implemented entirely in Rust, as a series of partially inter-dependent crates – some of which export C bindings to the supported API. This decomposition into distinct crates/modules is relatively recent, and in some cases current code has not been fully refactored to reflect the intended eventual organization.
  
There are currently four different crates:

- [**Storage Proofs (storage-proofs)**](./storage-proofs) 
    A library for constructing storage proofs – including non-circuit proofs, corresponding SNARK circuits, and a method of combining them.  
      
    `storage-proofs` is intended to serve as a reference implementation for _**Proof-of-Replication**_, while also performing the heavy lifting for `filecoin-proofs`.

     Primary Components:
     -   **PoR** (**_Proof-of-Retrievability_**: Merkle inclusion proof)
     -   **DrgPoRep** (_Depth Robust Graph_ **_Proof-of-Replication_**)
     -   **ZigZagDrgPoRep** (implemented as a specialized **LayeredDrgPoRep**)
     -   **PoSt** (Proof-of-Spacetime)


- [Filecoin Proofs (filecoin-proofs)](./filecoin-proofs) 
  A wrapper around `storage-proofs`, providing an FFI-exported API callable from C (and in practice called by [go-filecoin](https://github.com/filecoin-project/go-filecoin') via cgo). Filecoin-specific values of setup parameters are included here, and circuit parameters generated by Filecoin’s (future) trusted setup will also live here.

- [Sector Base (sector-base)](./sector-base) 
  A sector database abstracting away underlying storage considerations. This abstraction will allow for alternate implementations mapping logical sectors to physical storage – facilitating both support for miner specialization, and configurable adaptation to a given miner’s physical hardware and preferences.  

- [Storage Backend (storage-backend)](./storage-backend) 
  The `storage-backend` crate is intended to contain abstractions and implementations of non-filecoin-specific storage mechanisms require by `storage-proofs`. However, for the sake of simplicity, it is currently an empty placeholder.
    
    ![FPS crate dependencies](/img/fps-dependencies.png?raw=true)  
    
## Design Notes

Earlier in the design process, we considered implementing what has become the **FPS** in Go – as a wrapper around potentially multiple SNARK circuit libraries. We eventually decided to use [bellman](https://github.com/zkcrypto/bellman) – a library developed by ZCash, which supports efficient pedersen hashing inside of SNARKs. Having made that decision, it was natural and efficient to implement the entire subsystem in Rust. We considered the benefits (self-contained code base, ability to rely on static typing across layers) and costs (developer ramp-up, sometimes unwieldiness of borrow-checker) as part of that larger decision and determined that the overall project benefits (in particular ability to build on Zcash’s work) outweighed the costs.
      
We also considered whether the **FPS** should be implemented as a standalone binary accessed from [**go-filecoin**](https://github.com/filecoin-project/go-filecoin) either as a single-invocation CLI or as a long-running daemon process. We chose to bundle the **FPS** as an FFI dependency for both the simplicity of having a Filecoin node be deliverable as a single monolithic binary, and for the (perceived) relative development simplicity of the API implementation.
  
If at any point it were to become clear that the FFI approach is irredeemably problematic, the option of moving to a standalone **FPS** remains. However, we have now already solved the majority of technical problems associated with calling from Go into Rust even while allowing for a high degree of runtime configurability. Therefore, it seems most likely that we continue down the path in which we have already invested and begin to reap reward.
    
## API Reference
The **FPS** is accessed from [**go-filecoin**](https://github.com/filecoin-project/go-filecoin) via FFI calls to its API, which is the union of the APIs of its constituents: [**filecoin-proofs**](https://github.com/filecoin-project/rust-proofs/blob/master/filecoin-proofs/src/api/mod.rs) and [**sector-base**](https://github.com/filecoin-project/rust-proofs/blob/master/sector-base/src/api/mod.rs). The Rust source code serves as the source of truth defining the **FPS** APIs and will be the eventual source of generated documentation.
  
For more information about the significance of API functions, please see [PLACEHOLDER LINK]().  
- [Go implementation of filecoin-proofs API](https://github.com/filecoin-project/go-filecoin/blob/master/proofs/rustprover.go) and [associated interface structures](https://github.com/filecoin-project/go-filecoin/blob/master/proofs/interface.go).
- [Go implementation of sector-base API](https://github.com/filecoin-project/go-filecoin/blob/master/proofs/disk_backed_sector_store.go).  


## Install and configure Rust

[Install Rust.](https://www.rust-lang.org/en-US/install.html)

Configure to use nightly:

```
> rustup default nightly
```

## Build

```
> cargo build --release --all
```

## Test

```
> cargo test --all
```


## License

MIT or Apache 2.0
