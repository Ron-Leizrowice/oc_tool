// src/tweaks/msr/cache.rs

use indexmap::IndexMap;

use super::method::{MSRTweak, MsrState};
use crate::tweaks::{Tweak, TweakCategory, TweakId, TweakOption};

pub fn disable_opcache<'a>() -> Tweak<'a> {
    Tweak::msr_tweak(
        "Disable OpCache",
        "Disables the CPU's Op Cache, forcing instructions to be decoded from L1 instruction cache. While this typically reduces performance, it can help in specific debugging scenarios or when dealing with self-modifying code.",
        TweakCategory::Cpu,
        MSRTweak {
            id: TweakId::DisableOpCache,
            options: IndexMap::from_iter(
                vec![
                    (TweakOption::Enabled(false), vec![MsrState {
                        index: 0xC001_1021,
                        bit: 5,
                        state: false,
                    }]),
                    (TweakOption::Enabled(true), vec![MsrState {
                        index: 0xC001_1021,
                        bit: 5,
                        state: true,
                    }]),
                ]
            ),
            readable: true,
        },
    )
}

pub fn disable_tlb_cache<'a>() -> Tweak<'a> {
    Tweak::msr_tweak(
        "Disable TLB Cache",
        "Disables the assumption that TLB entries are cached, potentially increasing memory access latency for aggressive memory handling optimizations.",
        TweakCategory::Cpu,
        MSRTweak {
            id: TweakId::DisableTlbCache,
            options: IndexMap::from_iter(
                vec![
                    (TweakOption::Enabled(false), vec![MsrState {
                        index: 0xC001_0015,
                        bit: 3,
                        state: false,
                    }]),
                    (TweakOption::Option("Disabled".to_string()), vec![MsrState {
                        index: 0xC001_0015,
                        bit: 3,
                        state: true,
                    }]),
                ]
            ),
            readable: true,
        },
    )
}

// MSR0000_0C81 [L3 QoS Configuration] (L3::L3CRB::L3QosCfg1)
// Reset: 0000_0000_0000_0000h.
// QOS L3 Cache Allocation CDP mode enable (I vs. D). Contents are copied to ChL2QosCfg1 and ChL3QosCfg1_0.
// _ccd[1:0]_lthree0; MSR0000_0C81
// Bits Description
// 63:1 Reserved.
// 0 CDP. Read-write. Reset: 0. Code and Data Prioritization Technology enable

pub fn enable_l3_code_data_prioritization<'a>() -> Tweak<'a> {
    Tweak::msr_tweak(
        "L3 Cache Code/Data Prioritization",
        "Controls L3 cache partitioning between code and data. Enable for potentially higher benchmark scores by preventing instruction cache thrashing in L3. Most effective in benchmarks with large code footprints or when running multiple intensive benchmarks simultaneously.",
        TweakCategory::Cpu,
        MSRTweak {
            id: TweakId::EnableL3CodeDataPrioritization,
            options: IndexMap::from_iter(
                vec![
                    (TweakOption::Enabled(false), vec![MsrState {
                        index: 0x0000_0C81,
                        bit: 0,
                        state: false,
                    }]),
                    (TweakOption::Enabled(true), vec![MsrState {
                        index: 0x0000_0C81,
                        bit: 0,
                        state: true,
                    }]),
                ]
            ),
            readable: true,
        },
    )
}
