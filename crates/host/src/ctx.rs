use std::path::PathBuf;

use patricia_tree::PatriciaMap;
use wasmtime_wasi_host::WasiCtx;


#[derive(Default)]
pub struct RequestCtx {
    pub(crate) wasi: WasiCtx,
    /// Map of <Container Path -> Host Path> for preopened directories
    pub(crate) preopened_dirs: PatriciaMap<PathBuf>,
}

impl RequestCtx {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn preopen_dir(&mut self, container_path: &str, host_path: PathBuf) -> anyhow::Result<()> {
        self.preopened_dirs.insert(container_path, host_path.canonicalize()?);
        Ok(())
    }
}