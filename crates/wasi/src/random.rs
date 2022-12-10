#![allow(unused_variables)]
use rand::RngCore;

use crate::{wasi_random, WasiCtx};

impl wasi_random::WasiRandom for WasiCtx {
    fn getrandom(&mut self, len: u32) -> anyhow::Result<Vec<u8>> {
        let mut buf = vec![0; len as usize];
        let mut rng = rand::thread_rng();
        rng.fill_bytes(&mut buf);
        Ok(buf)
    }
}
