use crate::{wasi_logging, WasiCtx};

impl wasi_logging::WasiLogging for WasiCtx {
    fn log(
        &mut self,
        _level: wasi_logging::Level,
        _context: String,
        message: String,
    ) -> anyhow::Result<()> {
        print!("[{}] {message}", self.logging_context);
        Ok(())
    }
}

impl WasiCtx {
    pub fn set_context(&mut self, context: String) {
        self.logging_context = context;
    }
}