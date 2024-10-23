// src/utils/cpu.rs

use once_cell::sync::Lazy;

#[derive(Debug, Clone, Copy)]
pub struct CpuInfo {
    pub cores: usize,
    pub _threads: usize,
}

impl Default for CpuInfo {
    fn default() -> Self {
        Self::new()
    }
}

impl CpuInfo {
    pub fn new() -> Self {
        CpuInfo {
            cores: num_cpus::get(),
            _threads: num_cpus::get_physical(),
        }
    }
}

pub static CPU_INFO: Lazy<CpuInfo> = Lazy::new(CpuInfo::new);
