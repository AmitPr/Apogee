wit_bindgen_guest_rust::generate!({path:"./wit/filesystem.wit"});

#[cfg(feature="import")]
pub(crate) mod imports {
    wasmtime::component::bindgen!({
        path: "./wit/filesystem.wit",
    });
    
    pub use filesystem::add_to_linker;
}