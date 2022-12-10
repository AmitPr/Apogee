use std::{fs::File, io::Read, path::PathBuf};

use path_clean::PathClean;
use apogee_sdk::filesystem;

pub use apogee_sdk::filesystem::imports::add_to_linker;

use crate::RequestCtx;

impl filesystem::imports::filesystem::Filesystem for RequestCtx {
    fn read_file(&mut self, path: String) -> anyhow::Result<Result<Vec<u8>, String>> {
        // TODO: Figure out if any errors should be caught by Wasmtime
        // instead of forwarding them to the guest as Strings.
        Ok(self.read_file(path))
    }
}

impl RequestCtx {
    fn read_file(&mut self, path: String) -> Result<Vec<u8>, String> {
        // Ensure that the path is rooted
        let path = if !path.starts_with('/') {
            format!("/{path}")
        } else {
            path
        };
        let path = PathBuf::from(path).clean();

        //TODO: Files that are inside preopened directories should also be found.
        let key = path.to_str().ok_or("Path not valid UTF-8")?;
        let longest_common_prefix = self
            .preopened_dirs
            .get_longest_common_prefix(key)
            .ok_or("Path not preopened")?;

        let host_path = longest_common_prefix.1.join(
            path.strip_prefix(std::str::from_utf8(longest_common_prefix.0).unwrap())
                .unwrap(),
        );

        let mut file = File::open(host_path).map_err(|e| e.to_string())?;
        let mut contents = Vec::new();
        file.read_to_end(&mut contents).map_err(|e| e.to_string())?;
        Ok(contents)
    }
}
