// src/tweaks/msr/prefetch.rs

// MSRC000_0108 [Prefetch Control] (Core::X86::Msr::PrefetchControl)
// Reset: 0000_0000_0000_03C0h.
// _ccd[11:0]_lthree0_core[15:0]; MSRC000_0108
// Bits Description
// 63:10 Reserved.
// 9:7 PrefetchAggressivenessProfile. Read-write. Reset: 7h. When MasterEnable is set, selects a prefetch
// aggressiveness profile.
// ValidValues:
// Value Description
// 0h Level 0, least aggressive prefetch profile.
// 1h Level 1
// 2h Level 2
// 3h Level 3, most aggressive prefetch profile.
// 6h-4h Reserved.
// 7h Default used by hardware. Not software accessible.
// 6 MasterEnable. Read-write. Reset: 1. Enable prefetch aggressiveness profiles.
// 5 UpDown. Read-write. Reset: 0. Disable prefetcher that uses memory access history to determine whether to fetch
// the next or previous line into L2 cache for all memory accesses.
// 4 Reserved.
// 3 L2Stream. Read-write. Reset: 0. Disable prefetcher that uses history of memory access patterns to fetch
// additional sequential lines into L2 cache.
// 2 L1Region. Read-write. Reset: 0. Disable prefetcher that uses memory access history to fetch additional lines into
// L1 cache when the data access for a given instruction tends to be followed by a consistent pattern of other
// accesses within a localized region.
// 1 L1Stride. Read-write. Reset: 0. Disable stride prefetcher that uses memory access history of individual
// instructions to fetch additional lines into L1 cache when each access is a constant distance from the previous.
// 0 L1Stream. Read-write. Reset: 0. Disable stream prefetcher that uses history of memory access patterns to fetch
// additional sequential lines into L1 cache.

use indexmap::IndexMap;

use super::method::{MSRTweak, MsrState};
use crate::tweaks::{Tweak, TweakCategory, TweakId, TweakOption};

pub fn prefetch_aggressiveness_profile<'a>() -> Tweak<'a> {
    Tweak::msr_tweak(
        "Prefetcher Aggressiveness",
        "Controls CPU's cache prefetch aggressiveness. Level 0 is least aggressive (lowest power, may improve benchmarks that benefit from precise cache control), Level 3 is most aggressive (highest performance in sequential workloads). Benchmark your specific workload to find optimal setting.",
        TweakCategory::Cpu,
        MSRTweak {
            id: TweakId::AggressivePrefetchProfile,
            options: IndexMap::from_iter(vec![
                (
                    TweakOption::Option("Level 0".to_string()),
                    vec![
                        MsrState {
                            index: 0xC000_0108,
                            bit: 6,
                            state: true,
                        },
                        MsrState {
                            index: 0xC000_0108,
                            bit: 7,
                            state: false,
                        },
                        MsrState {
                            index: 0xC000_0108,
                            bit: 8,
                            state: false,
                        },
                        MsrState {
                            index: 0xC000_0108,
                            bit: 9,
                            state: false,
                        },
                    ],
                ),
                (
                    TweakOption::Option("Level 1".to_string()),
                    vec![
                        MsrState {
                            index: 0xC000_0108,
                            bit: 6,
                            state: true,
                        },
                        MsrState {
                            index: 0xC000_0108,
                            bit: 7,
                            state: true,
                        },
                        MsrState {
                            index: 0xC000_0108,
                            bit: 8,
                            state: false,
                        },
                        MsrState {
                            index: 0xC000_0108,
                            bit: 9,
                            state: false,
                        },
                    ],
                ),
                (
                    TweakOption::Option("Level 2".to_string()),
                    vec![
                        MsrState {
                            index: 0xC000_0108,
                            bit: 6,
                            state: true,
                        },
                        MsrState {
                            index: 0xC000_0108,
                            bit: 7,
                            state: true,
                        },
                        MsrState {
                            index: 0xC000_0108,
                            bit: 8,
                            state: true,
                        },
                        MsrState {
                            index: 0xC000_0108,
                            bit: 9,
                            state: false,
                        },
                    ],
                ),
                (
                    TweakOption::Option("Level 3".to_string()),
                    vec![
                        MsrState {
                            index: 0xC000_0108,
                            bit: 6,
                            state: true,
                        },
                        MsrState {
                            index: 0xC000_0108,
                            bit: 7,
                            state: true,
                        },
                        MsrState {
                            index: 0xC000_0108,
                            bit: 8,
                            state: true,
                        },
                        MsrState {
                            index: 0xC000_0108,
                            bit: 9,
                            state: true,
                        },
                    ],
                )

            ]),
            readable: true,
        },
    )
}

pub fn disable_up_down_prefetcher<'a>() -> Tweak<'a> {
    Tweak::msr_tweak(
        "Up/Down Prefetcher Control",
        "Disables L2 cache next/previous line prefetching. May improve benchmark scores in memory-sensitive tests by preventing speculative cache fills. Most effective when running benchmarks with non-sequential memory access patterns.",
        TweakCategory::Cpu,
        MSRTweak {
            id: TweakId::DisableUpDownPrefetcher,
            options: IndexMap::from_iter(
                vec![
                    (TweakOption::Enabled(false), vec![MsrState {
                        index: 0xC000_0108,
                        bit: 5,
                        state: false,
                    }]),
                    (TweakOption::Enabled(true), vec![MsrState {
                        index: 0xC000_0108,
                        bit: 5,
                        state: true,
                    }]),
                ]
            ),
            readable: true,
        },
    )
}

pub fn disable_l2_stream_prefetcher<'a>() -> Tweak<'a> {
    Tweak::msr_tweak(
        "L2 Stream Prefetcher Control",
        "Controls L2 cache sequential line prefetching. Disabling can boost benchmark scores by reducing memory bandwidth usage and preventing cache pollution in tests that don't benefit from sequential prefetch.",
        TweakCategory::Cpu,
        MSRTweak {
            id: TweakId::DisableL2StreamPrefetcher,
            options: IndexMap::from_iter(
                vec![
                    (TweakOption::Enabled(false), vec![MsrState {
                        index: 0xC000_0108,
                        bit: 3,
                        state: false,
                    }]),
                    (TweakOption::Enabled(true), vec![MsrState {
                        index: 0xC000_0108,
                        bit: 3,
                        state: true,
                    }]),
                ]
            ),
            readable: true,
        },
    )
}

pub fn disable_l1_region_prefetcher<'a>() -> Tweak<'a> {
    Tweak::msr_tweak(
        "L1 Region Prefetcher Control",
        "Controls L1 region-based prefetching. Disabling can improve benchmark performance by preventing unwanted cache fills in memory-intensive tests, especially effective in benchmarks with irregular access patterns.",
        TweakCategory::Cpu,
        MSRTweak {
            id: TweakId::DisableL1RegionPrefetcher,
            options: IndexMap::from_iter(
                vec![
                    (TweakOption::Enabled(false), vec![MsrState {
                        index: 0xC000_0108,
                        bit: 2,
                        state: false,
                    }]),
                    (TweakOption::Enabled(true), vec![MsrState {
                        index: 0xC000_0108,
                        bit: 2,
                        state: true,
                    }]),
                ]
            ),
            readable: true,
        },
    )
}

pub fn disable_l1_stride_prefetcher<'a>() -> Tweak<'a> {
    Tweak::msr_tweak(
        "L1 Stride Prefetcher Control",
        "Controls L1 stride-pattern prefetching. Disable for potentially higher benchmark scores in tests where precise cache control is beneficial. Most effective in benchmarks that don't rely on regular stride patterns.",
        TweakCategory::Cpu,
        MSRTweak {
            id: TweakId::DisableL1StridePrefetcher,
            options: IndexMap::from_iter(
                vec![
                    (TweakOption::Enabled(false), vec![MsrState {
                        index: 0xC000_0108,
                        bit: 1,
                        state: false,
                    }]),
                    (TweakOption::Enabled(true), vec![MsrState {
                        index: 0xC000_0108,
                        bit: 1,
                        state: true,
                    }]),
                ]
            ),
            readable: true,
        },
    )
}

pub fn disable_l1_stream_prefetcher<'a>() -> Tweak<'a> {
    Tweak::msr_tweak(
        "L1 Stream Prefetcher Control",
        "Controls L1 sequential prefetching. Disabling may increase benchmark scores by reducing cache pollution and memory bandwidth usage. Particularly effective for memory benchmarks where maximum control over cache behavior is desired.",
        TweakCategory::Cpu,
        MSRTweak {
            id: TweakId::DisableL1StreamPrefetcher,
            options: IndexMap::from_iter(
                vec![
                    (TweakOption::Enabled(false), vec![MsrState {
                        index: 0xC000_0108,
                        bit: 0,
                        state: false,
                    }]),
                    (TweakOption::Enabled(true), vec![MsrState {
                        index: 0xC000_0108,
                        bit: 0,
                        state: true,
                    }]),
                ]
            ),
            readable: true,
        },
    )
}
