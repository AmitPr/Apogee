use std::{fs::File, io::Read, path::Path};

use sdk::filesystem;

pub use sdk::filesystem::imports::add_to_linker;

use crate::RequestCtx;

impl filesystem::imports::filesystem::Filesystem for RequestCtx {
    fn read_file(&mut self, path: String) -> anyhow::Result<Result<Vec<u8>, String>> {
        // TODO: Figure out if any errors should be caught by Wasmtime
        // instead of forwarding them to the guest as Strings.
        Ok(read_file(path))
    }
}

//TODO: Add sandboxing
fn read_file(path: String) -> Result<Vec<u8>, String> {
    let path = Path::new(&path);
    let mut file = File::open(path).map_err(|e| e.to_string())?;
    let mut contents = Vec::new();
    file.read_to_end(&mut contents).map_err(|e| e.to_string())?;
    Ok(contents)
}
