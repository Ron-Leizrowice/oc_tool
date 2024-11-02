// src/tweaks/msr/mod.rs

use cache::{disable_opcache, disable_tlb_cache, enable_l3_code_data_prioritization};
use indexmap::IndexMap;
use method::{MSRTweak, MsrState};
use prefetch::{
    disable_l1_region_prefetcher, disable_l1_stream_prefetcher, disable_l1_stride_prefetcher,
    disable_l2_stream_prefetcher, disable_up_down_prefetcher, prefetch_aggressiveness_profile,
};

use super::{Tweak, TweakCategory, TweakId, TweakOption};

mod cache;
pub mod method;
mod prefetch;

pub fn all_msr_tweaks<'a>() -> Vec<(TweakId, Tweak<'a>)> {
    vec![
        (TweakId::AutomaticIbrsEnable, automatic_ibrs_enable()),
        (
            TweakId::AggressivePrefetchProfile,
            prefetch_aggressiveness_profile(),
        ),
        (
            TweakId::DisableUpDownPrefetcher,
            disable_up_down_prefetcher(),
        ),
        (
            TweakId::DisableL2StreamPrefetcher,
            disable_l2_stream_prefetcher(),
        ),
        (
            TweakId::DisableL1RegionPrefetcher,
            disable_l1_region_prefetcher(),
        ),
        (
            TweakId::DisableL1StridePrefetcher,
            disable_l1_stride_prefetcher(),
        ),
        (
            TweakId::DisableL1StreamPrefetcher,
            disable_l1_stream_prefetcher(),
        ),
        (
            TweakId::EnableMtrrFixedDramAttributes,
            enable_mtrr_fixed_dram_attributes(),
        ),
        (
            TweakId::EnableMtrrFixedDramModification,
            enable_mtrr_fixed_dram_modification(),
        ),
        (
            TweakId::EnableTranslationCacheExtension,
            translation_cache_extension_enable(),
        ),
        (TweakId::EnableFastFxsaveFrstor, enable_fast_fxsave_frstor()),
        (
            TweakId::DisbleControlFlowEnforcement,
            disable_control_flow_enforcement(),
        ),
        (
            TweakId::EnableInterruptibleWbinvd,
            enable_interruptible_wbinvd(),
        ),
        (
            TweakId::DisableMcaStatusWriteEnable,
            disable_mca_status_write_enable(),
        ),
        (TweakId::DisableTlbCache, disable_tlb_cache()),
        (
            TweakId::EnableL3CodeDataPrioritization,
            enable_l3_code_data_prioritization(),
        ),
        (TweakId::DisableStreamingStores, disable_streaming_stores()),
        (
            TweakId::DisableRedirectForReturn,
            disable_redirect_for_return(),
        ),
        (TweakId::DisableOpCache, disable_opcache()),
        (
            TweakId::SpeculativeStoreModes,
            cpu_speculative_store_modes(),
        ),
        (TweakId::DisableAvx512, disable_avx512()),
        (
            TweakId::DisableFastShortRepMovsb,
            disable_fast_short_rep_movsb(),
        ),
        (
            TweakId::DisableEnhancedRepMovsbStosb,
            disable_enhanced_rep_movsb_stosb(),
        ),
        (
            TweakId::DisableRepMovStosStreaming,
            disable_rep_mov_stos_streaming(),
        ),
        (
            TweakId::DisableCoreWatchdogTimer,
            disable_core_watchdog_timer(),
        ),
        (
            TweakId::DisablePlatformFirstErrorHandling,
            disable_platform_first_error_handling(),
        ),
    ]
}

// MSRC000_0080 [Extended Feature Enable] (Core::X86::Msr::EFER)
// SKINIT Execution: 0000_0000_0000_0000h.
// _ccd[11:0]_lthree0_core[15:0]_thread[1:0]; MSRC000_0080
// Bits Description
// 63:22 Reserved.
// 21 AutomaticIBRSEn: Automatic IBRS Enable. Read-write. Reset: 0. 0=IBRS protection is not enabled unless
// (SPEC_CTRL[IBRS] == 1). 1=IBRS protection is enabled for any process running at (CPL == 0) or ((ASID ==
// 0) && SEV-SNP).
// 20 UAIE: Upper Address Ignore Enable. Read-write. Reset: 0. Upper Address Ignore suppresses canonical faults
// for most data access virtual addresses, which allows software to use the upper bits of a virtual address as tags.
// 19 Reserved.
// 18 IntWbinvdEn. Read-write. Reset: 0. Interruptible wbinvd, wbnoinvd enable.
// 17:16 Reserved.
// 15 TCE: translation cache extension enable. Read-write. Reset: 0. 1=Translation cache extension is enabled. PDC
// entries related to the linear address of the INVLPG instruction are invalidated. If this bit is 0 all PDC entries are
// invalidated by the INVLPG instruction.
// 14 FFXSE: fast FXSAVE/FRSTOR enable. Read-write. Reset: 0. 1=Enables the fast FXSAVE/FRSTOR
// mechanism. A 64-bit operating system may enable the fast FXSAVE/FRSTOR mechanism if
// (Core::X86::Cpuid::FeatureExtIdEdx[FFXSR] == 1). This bit is set once by the operating system and its value is
// not changed afterwards.
// 13 LMSLE: long mode segment limit enable. Read-only,Error-on-write-1. Reset: Fixed,0. 1=Enables the long
// mode segment limit check mechanism.
// 12 SVME: secure virtual machine (SVM) enable. Reset: Fixed,0. 1=SVM features are enabled.
// AccessType: Core::X86::Msr::VM_CR[SvmeDisable] ? Read-only,Error-on-write-1 : Read-write.
// 11 NXE: no-execute page enable. Read-write. Reset: 0. 1=The no-execute page protection feature is enabled.
// 10 LMA: long mode active. Read-only. Reset: 0. 1=Indicates that long mode is active. When writing the EFER
// register the value of this bit must be preserved. Software must read the EFER register to determine the value of
// LMA, change any other bits as required and then write the EFER register. An attempt to write a value that differs
// from the state determined by hardware results in a #GP fault.
// 9 Reserved.
// 8 LME: long mode enable. Read-write. Reset: 0. 1=Long mode is enabled.
// 7:1 Reserved.
// 0 SYSCALL: system call extension enable. Read-write. Reset: 0. 1=SYSCALL and SYSRET instructions are
// enabled. This adds the SYSCALL and SYSRET instructions which can be used in flat addressed operating
// systems as low latency system calls and returns.
pub fn automatic_ibrs_enable<'a>() -> Tweak<'a> {
    Tweak::msr_tweak(
        "Automatic IBRS Control",
        "Controls automatic Indirect Branch Restricted Speculation in kernel mode. Disabling this can reduce CPU overhead from automatic branch prediction restrictions, potentially improving performance in cpu-intensive workloads and benchmarks.",
        TweakCategory::Cpu,
        MSRTweak {
            id: TweakId::AutomaticIbrsEnable,
            options: IndexMap::from_iter(
                vec![
                    (TweakOption::Enabled(false), vec![MsrState {
                        index: 0xC000_0080,
                        bit: 21,
                        state: false,
                    }]),
                    (TweakOption::Enabled(true), vec![MsrState {
                        index: 0xC000_0080,
                        bit: 21,
                        state: true,
                    }]),
                ]
            ),
            readable: true,
        },
    )
}

pub fn enable_interruptible_wbinvd<'a>() -> Tweak<'a> {
    Tweak::msr_tweak(
        "Enable Interruptible WBINVD",
        "Makes the Write-Back and Invalidate Cache (WBINVD) instruction interruptible, reducing system stall time during cache flushes. This can significantly improve system responsiveness during cache maintenance operations.",
        TweakCategory::Cpu,
        MSRTweak {
            id: TweakId::EnableInterruptibleWbinvd,
            options: IndexMap::from_iter(
                vec![
                    (TweakOption::Enabled(false), vec![MsrState {
                        index: 0xC000_0080,
                        bit: 18,
                        state: false,
                    }]),
                    (TweakOption::Enabled(true), vec![MsrState {
                        index: 0xC000_0080,
                        bit: 18,
                        state: true,
                    }]),
                ]
            ),
            readable: true,
        },
    )
}

pub fn translation_cache_extension_enable<'a>() -> Tweak<'a> {
    Tweak::msr_tweak(
        "Translation Cache Extension Enable",
        "Optimizes INVLPG instruction behavior to only invalidate PDC entries related to the specified linear address instead of all entries. This can improve performance in scenarios with frequent TLB invalidations.",
        TweakCategory::Cpu,
        MSRTweak {
            id: TweakId::EnableTranslationCacheExtension,
            options: IndexMap::from_iter(
                vec![
                    (TweakOption::Enabled(false), vec![MsrState {
                        index: 0xC000_0080,
                        bit: 15,
                        state: false,
                    }]),
                    (TweakOption::Enabled(true), vec![MsrState {
                        index: 0xC000_0080,
                        bit: 15,
                        state: true,
                    }]),
                ]
            ),
            readable: true,
        },
    )
}

pub fn enable_fast_fxsave_frstor<'a>() -> Tweak<'a> {
    Tweak::msr_tweak(
        "Enable Fast FXSAVE/FRSTOR",
        "Enables optimized handling of FPU and SSE state save/restore operations in 64-bit mode. This can significantly improve context switch performance in applications using floating-point or SIMD instructions.",
        TweakCategory::Cpu,
        MSRTweak {
            id: TweakId::EnableFastFxsaveFrstor,
            options: IndexMap::from_iter(
                vec![
                    (TweakOption::Enabled(false), vec![MsrState {
                        index: 0xC000_0080,
                        bit: 14,
                        state: false,
                    }]),
                    (TweakOption::Enabled(true), vec![MsrState {
                        index: 0xC000_0080,
                        bit: 14,
                        state: true,
                    }]),
                ]
            ),
            readable: true,
        },
    )
}

// MSRC001_0010 [System Configuration] (Core::X86::Msr::SYS_CFG)
// Reset: 0000_0000_0000_0000h.
// If Core::X86::Msr::SYS_CFG[SecureNestedPagingEn] is set, writes to this register are ignored.
// _ccd[11:0]_lthree0_core[15:0]; MSRC001_0010
// Bits Description
// 63:27 Reserved.
// 26 HMKEE: Host Multi-Key Encryption Enable. Read,Write-1-only. Reset: 0. Used with SYS_CFG[SMEE] to
// select secure memory encryption mode. See SYS_CFG[SMEE] for a table listing the available memory
// encryption modes.
// 25 VmplEn. Reset: 0. VM permission levels enable.
// AccessType: Core::X86::Msr::SYS_CFG[SecureNestedPagingEn] ? Read-only : Read-write.
// 24 SecureNestedPagingEn. Read,Write-1-only. Reset: 0. Enable Secure Nested Paging (SNP).
// 23 SMEE: Secure Memory Encryption Enable. Read,Write-1-only. Reset: 0.
// Description: Used with SYS_CFG[HMKEE] to select secure memory encryption mode. See the table below for
// the available memory encryption modes.
// HMKEE SMEE Description
// 0 0 No encryption.
// 0 1 Enables SME and SEV memory encryption.
// 1 0 Enables SME-HMK memory encryption.
// 1 1 Not supported. Results in #GP.
// 22 Tom2ForceMemTypeWB: top of memory 2 memory type write back. Read-write. Reset: 0. 1=The default
// memory type of memory between 4GB and Core::X86::Msr::TOM2 is write back instead of the memory type
// defined by Core::X86::Msr::MTRRdefType[MemType]. For this bit to have any effect,
// Core::X86::Msr::MTRRdefType[MtrrDefTypeEn] must be 1. MTRRs and PAT can be used to override this
// memory type.
// 21 MtrrTom2En: MTRR top of memory 2 enable. Read-write. Reset: 0. 0=Core::X86::Msr::TOM2 is disabled. 1=
// Core::X86::Msr::TOM2 is enabled.
// 20 MtrrVarDramEn: MTRR variable DRAM enable. Read-write. Reset: 0. Init: BIOS,1.
// 0=Core::X86::Msr::TOP_MEM and IORRs are disabled. 1=These registers are enabled.
// 19 MtrrFixDramModEn: MTRR fixed RdDram and WrDram modification enable. Read-write. Reset: 0.
// 0=Core::X86::Msr::MtrrFix_64K through Core::X86::Msr::MtrrFix_4K_7 [RdDram,WrDram] read values is
// masked 00b; writing does not change the hidden value. 1=Core::X86::Msr::MtrrFix_64K through
// Core::X86::Msr::MtrrFix_4K_7 [RdDram,WrDram] access type is Read-write. Not shared between threads.
// Controls access to Core::X86::Msr::MtrrFix_64K through Core::X86::Msr::MtrrFix_4K_7 [RdDram ,WrDram].
// This bit should be set to 1 during BIOS initialization of the fixed MTRRs, then cleared to 0 for operation.
// 18 MtrrFixDramEn: MTRR fixed RdDram and WrDram attributes enable. Read-write. Reset: 0. Init: BIOS,1.
// 1=Enables the RdDram and WrDram attributes in Core::X86::Msr::MtrrFix_64K through
// Core::X86::Msr::MtrrFix_4K_7.
// 17:0 Reserved.

// MSR0000_02FF [MTRR Default Memory Type] (Core::X86::Msr::MTRRdefType)
// See Core::X86::Msr::MtrrVarBase for general MTRR information.
// _ccd[11:0]_lthree0_core[15:0]; MSR0000_02FF
// Bits Description
// 63:12 Reserved.
// 11 MtrrDefTypeEn: variable and fixed MTRR enable. Read-write. Reset: 0. 0=Fixed and variable MTRRs are not
// enabled. 1=Core::X86::Msr::MtrrVarBase, and Core::X86::Msr::MtrrFix_64K through
// Core::X86::Msr::MtrrFix_4K_7 are enabled.
// 10 MtrrDefTypeFixEn: fixed MTRR enable. Read-write. Reset: 0. 0=Core::X86::Msr::MtrrFix_64K through
// Core::X86::Msr::MtrrFix_4K_7 are not enabled. 1=Core::X86::Msr::MtrrFix_64K through
// Core::X86::Msr::MtrrFix_4K_7 are enabled. This field is ignored (and the fixed MTRRs are not enabled) if
// Core::X86::Msr::MTRRdefType[MtrrDefTypeEn] == 0.
// 9:8 Reserved.
// 7:0 MemType: memory type. Read-write. Reset: 00h.
// Description: If MtrrDefTypeEn == 1 then MemType specifies the memory type for memory space that is not
// specified by either the fixed or variable range MTRRs. If MtrrDefTypeEn == 0 then the default memory type for
// all of memory is UC.
// Valid encodings are {00000b, Core

pub fn enable_mtrr_fixed_dram_modification<'a>() -> Tweak<'a> {
    Tweak::msr_tweak(
        "Enable MTRR Fixed DRAM Modification",
        "Enables modification of fixed Memory Type Range Registers (MTRRs) RdDram and WrDram attributes. This allows customization of cache behavior for fixed memory ranges, typically used during BIOS initialization for optimal memory subsystem configuration.",
        TweakCategory::Cpu,
        MSRTweak {
            id: TweakId::EnableMtrrFixedDramModification,
            options: IndexMap::from_iter(
                vec![
                    (TweakOption::Enabled(false), vec![MsrState {
                        index: 0xC001_0010,
                        bit: 19,
                        state: false,
                    }]),
                    (TweakOption::Enabled(true), vec![MsrState {
                        index: 0xC001_0010,
                        bit: 19,
                        state: true,
                    }]),
                ]
            ),
            readable: true,
        },
    )
}

pub fn enable_mtrr_fixed_dram_attributes<'a>() -> Tweak<'a> {
    Tweak::msr_tweak(
        "Enable MTRR Fixed DRAM Attributes",
        "Enables the RdDram and WrDram attributes in fixed Memory Type Range Registers (MTRRs). This allows separate read and write caching policies for fixed memory ranges, enabling more sophisticated memory access optimizations.",
        TweakCategory::Cpu,
        MSRTweak {
            id: TweakId::EnableMtrrFixedDramAttributes,
            options: IndexMap::from_iter(
                vec![
                    (TweakOption::Enabled(false), vec![MsrState {
                        index: 0xC001_0010,
                        bit: 18,
                        state: false,
                    }]),
                    (TweakOption::Enabled(true), vec![MsrState {
                        index: 0xC001_0010,
                        bit: 18,
                        state: true,
                    }]),
                ]
            ),
            readable: true,
        },
    )
}

// MSR0000_0DA0 [Extended Supervisor State] (Core::X86::Msr::XSS)
// _ccd[11:0]_lthree0_core[15:0]_thread[1:0]; MSR0000_0DA0
// Bits Description
// 63:13 Reserved.
// 12 CET_S. Read-write. Reset: 0. System Control-flow Enforcement Technology.
// 11 CET_U. Read-write. Reset: 0. User Control-flow Enforcement Technology.
// 10:0 Reserved.

pub fn disable_control_flow_enforcement<'a>() -> Tweak<'a> {
    Tweak::msr_tweak(
        "Disable Control-Flow Enforcement",
        "Disables Control-flow Enforcement Technology (CET). While this removes protection against control-flow hijacking attacks, it eliminates the performance overhead associated with shadow stacks and indirect branch tracking.",
        TweakCategory::Cpu,
        MSRTweak {
            id: TweakId::DisbleControlFlowEnforcement,
            options: IndexMap::from_iter(
                vec![
                    (TweakOption::Enabled(false), vec![MsrState {
                        index: 0x0000_0DA0,
                        bit: 12,
                        state: false,
                    }]),
                    (TweakOption::Option("Disabled".to_string()), vec![MsrState {
                        index: 0x0000_0DA0,
                        bit: 12,
                        state: true,
                    }]),
                ]
            ),
            readable: true,
        },
    )
}

pub fn disable_mca_status_write_enable<'a>() -> Tweak<'a> {
    Tweak::msr_tweak(
        "Disable MCA Status Write Enable",
        "Disables write access to MCA status registers to reduce overhead related to error handling for enhanced performance.",
        TweakCategory::Cpu,
        MSRTweak {
            id: TweakId::DisableMcaStatusWriteEnable,
            options: IndexMap::from_iter(
                vec![
                    (TweakOption::Enabled(false), vec![MsrState {
                        index: 0xC001_0015,
                        bit: 18,
                        state: false,
                    }]),
                    (TweakOption::Option("Disabled".to_string()), vec![MsrState {
                        index: 0xC001_0015,
                        bit: 18,
                        state: true,
                    }]),
                ]
            ),
            readable: true,
        },
    )
}

// Setting                             MSRs       Bit    Write

// Streaming Stores Disabled                       |   0xC0011020  |   28  |   1
// RedirectForReturnDis                            |   0xC0011029  |   14  |   1
// Disable Global C-States                         |   0xC001100C  |   12  |   0
//                                                 |   0xC001100C  |   15  |   0
//                                                 |   0xC001100C  |   16  |   0
//                                                 |   0xC001100C  |   17  |   0
//                                                 |   0xC001100C  |   18  |   0
//                                                 |   0xC001100C  |   20  |   0
// Disable OpCache                                 |   0xC0011021  |   5   |   1
// SVM Disable                                     |   0xC0010114  |   4   |   1
// CPU Speculative Store Modes (Balanced)          |   0xC00110EC  |   0   |   0
// CPU Speculative Store Modes (Less Speculative)  |   0xC00110E5  |   26  |   0
// CPU Speculative Store Modes (More Speculative)  |   0xC00110EC  |   0   |   1
//                                                 |   0xC00110E5  |   26  |   1
// MONITOR MONITOR and MWAIT disable               |   0xC0010015  |   9   |   1
// Disable AVX512                                  |   0xC0011022  |   16  |   0
//                                                 |   0xC0011022  |   17  |   0
//                                                 |   0xC0011022  |   21  |   0
//                                                 |   0xC0011022  |   28  |   0
//                                                 |   0xC0011022  |   30  |   0
//                                                 |   0xC0011022  |   31  |   0
// Fast Short REP MOVSB (Disable)                  |   0xC00110DF  |   36  |   0
// Enhanced REP MOBSB/STOSB (Disable)              |   0xC0011002  |   9   |   0
// REP-MOV/STOS Streaming (Disable)                |   0xC0011000  |   15  |   1
// Disable PSS                                     |   0xC00102B1  |   0   |   0
// Core Watchdog Timer Disable                     |   0xC0010074  |   0   |   0
//                                                 |   0xC0010074  |   3   |   0
// Platform First Error Handling                   |   0xC0000410  |   5   |   0
//                                                 |   0xC0000410  |   12  |   0

pub fn disable_streaming_stores<'a>() -> Tweak<'a> {
    Tweak::msr_tweak(
        "Disable Streaming Stores",
        "Disables CPU streaming store operations (non-temporal stores). While this may increase memory bandwidth usage due to cache pollution, it can improve performance in scenarios where data is likely to be reused soon after writing.",
        TweakCategory::Cpu,
        MSRTweak {
            id: TweakId::DisableStreamingStores,
            options: IndexMap::from_iter(
                vec![
                    (TweakOption::Enabled(false), vec![MsrState {
                        index: 0xC001_1020,
                        bit: 28,
                        state: false,
                    }]),
                    (TweakOption::Option("Disabled".to_string()), vec![MsrState {
                        index: 0xC001_1020,
                        bit: 28,
                        state: true,
                    }]),
                ]
            ),
            readable: true,
        },
    )
}

pub fn disable_redirect_for_return<'a>() -> Tweak<'a> {
    Tweak::msr_tweak(
        "Disable Redirect for Return",
        "Disables CPU's Return Stack Buffer (RSB) redirect mechanism. This can improve performance in specific workloads by preventing unnecessary speculation redirects, but may slightly impact branch prediction accuracy.",
        TweakCategory::Cpu,
        MSRTweak {
            id: TweakId::DisableRedirectForReturn,
            options: IndexMap::from_iter(
                vec![
                    (TweakOption::Enabled(false), vec![MsrState {
                        index: 0xC001_1029,
                        bit: 14,
                        state: false,
                    }]),
                    (TweakOption::Enabled(true), vec![MsrState {
                        index: 0xC001_1029,
                        bit: 14,
                        state: true,
                    }]),
                ]
            ),
            readable: true,
        },
    )
}

pub fn cpu_speculative_store_modes<'a>() -> Tweak<'a> {
    Tweak::msr_tweak(
        "Speculative Store Modes",
        "Enables more aggressive store-to-load forwarding and memory disambiguation. Can improve performance in memory-intensive workloads at the cost of potentially higher power consumption and slightly increased risk of memory ordering violations.",
        TweakCategory::Cpu,
        MSRTweak {
            id: TweakId::SpeculativeStoreModes,
            options: IndexMap::from_iter(
                vec![
                    (TweakOption::Option("Balanced".to_string()), vec![
                        MsrState {
                            index: 0xC001_10EC,
                            bit: 0,
                            state: false,
                        },
                        MsrState {
                            index: 0xC001_10E5,
                            bit: 26,
                            state: false,
                        },
                    ]),
                    (TweakOption::Option("Less Speculative".to_string()), vec![
                        MsrState {
                            index: 0xC001_10EC,
                            bit: 0,
                            state: false,
                        },
                        MsrState {
                            index: 0xC001_10E5,
                            bit: 26,
                            state: true,
                        },
                    ]),
                    (TweakOption::Option("More Speculative".to_string()), vec![
                        MsrState {
                            index: 0xC001_10EC,
                            bit: 0,
                            state: true,
                        },
                        MsrState {
                            index: 0xC001_10E5,
                            bit: 26,
                            state: true,
                        },
                    ]),
                ]
            ),
            readable: true,
        },
    )
}

pub fn disable_avx512<'a>() -> Tweak<'a> {
    Tweak::msr_tweak(
        "Disable AVX512",
        "Disables AVX-512 instruction set extensions. This prevents frequency downclocking that occurs during AVX-512 operations and can improve overall system performance when AVX-512 instructions are not crucial for workload performance.",
        TweakCategory::Cpu,
        MSRTweak {
            id: TweakId::DisableAvx512,
            options: IndexMap::from_iter(
                vec![
                    (TweakOption::Enabled(false), vec![
                        MsrState {
                            index: 0xC001_1022,
                            bit: 16,
                            state: false,
                        },
                        MsrState {
                            index: 0xC001_1022,
                            bit: 17,
                            state: false,
                        },
                        MsrState {
                            index: 0xC001_1022,
                            bit: 21,
                            state: false,
                        },
                        MsrState {
                            index: 0xC001_1022,
                            bit: 28,
                            state: false,
                        },
                        MsrState {
                            index: 0xC001_1022,
                            bit: 30,
                            state: false,
                        },
                        MsrState {
                            index: 0xC001_1022,
                            bit: 31,
                            state: false,
                        },
                    ]),
                    (TweakOption::Enabled(true), vec![
                        MsrState {
                            index: 0xC001_1022,
                            bit: 16,
                            state: true,
                        },
                        MsrState {
                            index: 0xC001_1022,
                            bit: 17,
                            state: true,
                        },
                        MsrState {
                            index: 0xC001_1022,
                            bit: 21,
                            state: true,
                        },
                        MsrState {
                            index: 0xC001_1022,
                            bit: 28,
                            state: true,
                        },
                        MsrState {
                            index: 0xC001_1022,
                            bit: 30,
                            state: true,
                        },
                        MsrState {
                            index: 0xC001_1022,
                            bit: 31,
                            state: true,
                        },
                    ]),
                ]
            ),
            readable: true,
        },
    )
}

pub fn disable_fast_short_rep_movsb<'a>() -> Tweak<'a> {
    Tweak::msr_tweak(
        "Disable Fast Short REP MOVSB",
        "Disables optimized handling of short REP MOVSB instructions. While this optimization typically improves small memory copy operations, disabling it can be beneficial when the overhead of enabling the optimization exceeds its benefits in specific workloads.",
        TweakCategory::Cpu,
        MSRTweak {
            id: TweakId::DisableFastShortRepMovsb,
            options: IndexMap::from_iter(
                vec![
                    (TweakOption::Enabled(false), vec![MsrState {
                        index: 0xC001_10DF,
                        bit: 36,
                        state: false,
                    }]),
                    (TweakOption::Enabled(true), vec![MsrState {
                        index: 0xC001_10DF,
                        bit: 36,
                        state: true,
                    }]),
                ]
            ),
            readable: true,
        },
    )
}

pub fn disable_enhanced_rep_movsb_stosb<'a>() -> Tweak<'a> {
    Tweak::msr_tweak(
        "Disable Enhanced REP MOVSB/STOSB",
        "Disables enhanced string operation optimizations for REP MOVSB/STOSB instructions. Can improve performance in workloads where the overhead of enabling these optimizations exceeds their benefits, particularly with small or unaligned memory operations.",
        TweakCategory::Cpu,
        MSRTweak {
            id: TweakId::DisableEnhancedRepMovsbStosb,
            options: IndexMap::from_iter(
                vec![
                    (TweakOption::Enabled(false), vec![MsrState {
                        index: 0xC001_1002,
                        bit: 9,
                        state: false,
                    }]),
                    (TweakOption::Enabled(true), vec![MsrState {
                        index: 0xC001_1002,
                        bit: 9,
                        state: true,
                    }]),
                ]
            ),
            readable: true,
        },
    )
}

pub fn disable_rep_mov_stos_streaming<'a>() -> Tweak<'a> {
    Tweak::msr_tweak(
        "Disable REP-MOV/STOS Streaming",
        "Disables streaming optimization for REP MOV/STOS instructions. This can improve performance in scenarios where memory operations need to maintain cache coherency or when data is likely to be immediately reused.",
        TweakCategory::Cpu,
        MSRTweak {
            id: TweakId::DisableRepMovStosStreaming,
            options: IndexMap::from_iter(
                vec![
                    (TweakOption::Enabled(false), vec![MsrState {
                        index: 0xC001_1000,
                        bit: 15,
                        state: false,
                    }]),
                    (TweakOption::Enabled(false), vec![MsrState {
                        index: 0xC001_1000,
                        bit: 15,
                        state: true,
                    }]),
                ]
            ),
            readable: true,
        },
    )
}

pub fn disable_core_watchdog_timer<'a>() -> Tweak<'a> {
    Tweak::msr_tweak(
        "Disable Core Watchdog Timer",
        "Disables the CPU core watchdog timer mechanism. This can reduce overhead from periodic timer interrupts and improve performance in scenarios where system stability monitoring is not critical.",
        TweakCategory::Cpu,
        MSRTweak {
            id: TweakId::DisableCoreWatchdogTimer,
            options: IndexMap::from_iter(
                vec![
                    (TweakOption::Enabled(false), vec![
                        MsrState {
                            index: 0xC001_0074,
                            bit: 0,
                            state: false,
                        },
                        MsrState {
                            index: 0xC001_0074,
                            bit: 3,
                            state: false,
                        },
                    ]),
                    (TweakOption::Enabled(false), vec![
                        MsrState {
                            index: 0xC001_0074,
                            bit: 0,
                            state: true,
                        },
                        MsrState {
                            index: 0xC001_0074,
                            bit: 3,
                            state: true,
                        },
                    ]),
                ]
            ),
            readable: true,
        },
    )
}

pub fn disable_platform_first_error_handling<'a>() -> Tweak<'a> {
    Tweak::msr_tweak(
        "Disable Platform First Error Handling",
        "Disables the platform's first error handling mechanism. This reduces overhead from error checking and handling routines, potentially improving performance in stable systems where comprehensive error handling is not critical.",
        TweakCategory::Cpu,
        MSRTweak {
            id: TweakId::DisablePlatformFirstErrorHandling,
            options: IndexMap::from_iter(
                vec![
                    (TweakOption::Enabled(false), vec![
                        MsrState {
                            index: 0xC000_0410,
                            bit: 5,
                            state: false,
                        },
                        MsrState {
                            index: 0xC000_0410,
                            bit: 12,
                            state: false,
                        },
                    ]),
                    (TweakOption::Enabled(false), vec![
                        MsrState {
                            index: 0xC000_0410,
                            bit: 5,
                            state: true,
                        },
                        MsrState {
                            index: 0xC000_0410,
                            bit: 12,
                            state: true,
                        },
                    ]),
                ]
            ),
            readable: true,
        },
    )
}
