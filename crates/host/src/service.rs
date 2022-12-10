use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use wasmtime::{component::Component, Engine};

use crate::ctx::RequestCtx;

pub struct Service {
    pub component: Component,
    pub name: String,
    pub directory: PathBuf,
    pub config: ServiceConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfig {
    pub name: String,
    pub wasm: PathBuf,
    pub filesystem: Vec<FilesystemEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilesystemEntry {
    pub path: PathBuf,
    pub target: PathBuf,
}

impl Service {
    pub fn load(directory: PathBuf, engine: &Engine) -> anyhow::Result<Service> {
        let service_file = directory.join("service.toml");
        if !service_file.exists() {
            return Err(anyhow!("Service file not found"));
        }
        let service_config =
            toml::from_str::<ServiceConfig>(std::fs::read_to_string(&service_file)?.as_str())?;

        let wasm_path = directory.join(&service_config.wasm);
        let component = Component::from_file(engine, wasm_path)?;

        Ok(Service {
            component,
            name: service_config.name.clone(),
            directory,
            config: service_config,
        })
    }

    pub fn construct_ctx(&self) -> anyhow::Result<RequestCtx> {
        let mut ctx = RequestCtx::new();
        for entry in &self.config.filesystem {
            ctx.preopen_dir(
                entry
                    .target
                    .to_str()
                    .ok_or_else(|| anyhow!("Invalid target path: {:?}", entry.path))?,
                self.directory.join(&entry.path),
            )?;
        }

        ctx.wasi.set_context(self.name.clone());
        Ok(ctx)
    }
}
