// src/tweaks/msr/mod.rs

use method::{MSRTweak, MsrTweakState};

use super::{Tweak, TweakCategory, TweakId};

pub mod method;

pub fn all_msr_tweaks<'a>() -> Vec<(TweakId, Tweak<'a>)> {
    vec![
        (TweakId::DowngradeFp512ToFp256, downgrade_fp512_to_fp256()),
        (TweakId::AutomaticIbrsEnable, automatic_ibrs_enable()),
        (
            TweakId::EnableUpperAddressIgnore,
            enable_upper_address_ignore(),
        ),
        (
            TweakId::DisableSecureVirtualMachine,
            disable_secure_virtual_machine(),
        ),
        (
            TweakId::AggressivePrefetchProfile,
            enable_aggressive_prefetch_profile(),
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
            set_cpu_speculative_store_modes_more_speculative(),
        ),
        // (
        //     TweakId::DisableMonitorMonitorAndMwait,
        //     disable_monitor_monitor_and_mwait(),
        // ),
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

// MSR0000_0048 [Speculative Control] (Core::X86::Msr::SPEC_CTRL)
// _ccd[11:0]_lthree0_core[15:0]_thread[1:0]; MSR0000_0048
// Bits Description
// 63:8 Reserved.
// 7 PSFD: Predictive Store Forwarding Disable. Read-write. Reset: 0. 1=Disable predictive store forwarding.
// 6:3 Reserved.
// 2 SSBD. Read-write. Reset: 0. Speculative Store Bypass Disable.
// 1 STIBP. Read-write. Reset: 0. Single thread indirect branch predictor.
// 0 IBRS. Read-write. Reset: 0. Indirect branch restriction speculation.

// pub fn disable_predictive_store_forwarding<'a>() -> Tweak<'a> {
//     Tweak::msr_tweak(
//         "Disable Predictive Store Forwarding",
//         "Disables CPU's ability to speculatively forward store data to subsequent loads. While this can mitigate potential security vulnerabilities, it may impact performance in scenarios with frequent store-to-load dependencies.",
//         TweakCategory::Cpu,
//         MSRTweak {
//             id: TweakId::DisablePredictiveStoreForwarding,
//             msrs : vec![
//                 MsrTweakState {
//                     index: 0x0000_0048,
//                     bit: 7,
//                     state: true
//                 }
//             ],
//             readable: true,
//         },
//         false,
//     )
// }

// pub fn disable_speculative_store_bypass<'a>() -> Tweak<'a> {
//     Tweak::msr_tweak(
//         "Disable SSB",
//         "Disables Speculative Store Bypass (SSBD) preventing loads from executing before the addresses of all older stores are known. This mitigates Speculative Store Bypass vulnerabilities but may reduce performance in memory-intensive workloads.",
//         TweakCategory::Cpu,
//         MSRTweak {
//             id: TweakId::DisableSpeculativeStoreBypass,
//             msrs : vec![
//                 MsrTweakState {
//                     index: 0x0000_0048,
//                     bit: 2,
//                     state: true
//                 }
//             ],
//             readable: true,
//         },
//         false,
//     )
// }

// pub fn disable_single_thread_indirect_branch_predictor<'a>() -> Tweak<'a> {
//     Tweak::msr_tweak(
//         "Disable STIBP",
//         "Disables the Single Thread Indirect Branch Predictor (STIBP), preventing one CPU thread from using branch predictions trained by another thread. While this can enhance security against cross-thread attacks, it may impact performance in multi-threaded applications.",
//         TweakCategory::Cpu,
//         MSRTweak {
//             id: TweakId::DisableSingleThreadIndirectBranchPredictor,
//             msrs: vec![
//                 MsrTweakState {
//                     index: 0x0000_0048,
//                     bit: 1,
//                     state: true
//                 }
//             ],
//             readable: true,
//         },
//         false,
//     )
// }

// pub fn disable_indirect_branch_restriction_speculation<'a>() -> Tweak<'a> {
//     Tweak::msr_tweak(
//         "Disable IBRS",
//         "Disables Indirect Branch Restriction Speculation (IBRS), controlling the CPU's ability to speculatively execute indirect branches. While disabling can improve performance, it may expose the system to certain branch target injection attacks.",
//         TweakCategory::Cpu,
//         MSRTweak {
//             id: TweakId::DisableIndirectBranchRestrictionSpeculation,
//             msrs: vec![
//                 MsrTweakState {
//                     index: 0x0000_0048,
//                     bit: 0,
//                     state: true
//                 }
//             ],
//             readable: true,
//         },
//         false,
//     )
// }

// MSR0000_0049 [Prediction Command] (Core::X86::Msr::PRED_CMD)
// Write-only,Error-on-read. Reset: 0000_0000_0000_0000h.
// _ccd[11:0]_lthree0_core[15:0]; MSR0000_0049
// Bits Description
// 63:8 Reserved.
// 7 SBPB: selective branch predictor barrior. Write-only,Error-on-read. Reset: 0. When SBPB is supported
// (Core::X86::Cpuid::FeatureExt2Eax[SBPB]==1), setting this bit initiates a selective branch predictor barrier
// 6:1 Reserved.
// 0 IBPB: indirect branch prediction barrier. Write-only,Error-on-read. Reset: 0. Supported if
// Core::X86::Cpuid::FeatureExtIdEbx[IBPB] == 1.

// pub fn selective_branch_predictor_barrier<'a>() -> Tweak<'a> {
//     Tweak::msr_tweak(
//         "Selective Branch Predictor Barrier",
//         "Initiates a selective flush of the branch predictor to prevent exploitation of branch prediction history. This targeted approach offers better performance than full branch predictor barriers while still providing security benefits.",
//         TweakCategory::Cpu,
//         MSRTweak {
//             id: TweakId::SelectiveBranchPredictorBarrier,
//             msrs: vec![
//                 MsrTweakState {
//                     index: 0x0000_0049,
//                     bit: 7,
//                     state: true
//                 }
//             ],
//             readable: false,
//         },
//         false,
//     )
// }

// pub fn indirect_branch_prediction_barrier<'a>() -> Tweak<'a> {
//     Tweak::msr_tweak(
//         "Indirect Branch Prediction Barrier",
//         "Forces a complete reset of the indirect branch predictor state, preventing cross-process exploitation of branch prediction. While more comprehensive than SBPB, it incurs higher performance overhead.",
//         TweakCategory::Cpu,
//         MSRTweak {
//             id: TweakId::IndirectBranchPredictionBarrier,
//             msrs: vec![
//                 MsrTweakState {
//                     index: 0x0000_0049,
//                     bit: 0,
//                     state: true
//                 }
//             ],
//             readable: false,
//         },
//         false,
//     )
// }

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
        "Enable Automatic IBRS",
        "Automatically enables Indirect Branch Restricted Speculation for kernel-mode code (CPL=0) and SEV-SNP environments. This provides consistent protection against indirect branch attacks with lower overhead than manual IBRS control.",
        TweakCategory::Cpu,
        MSRTweak {
            id: TweakId::AutomaticIbrsEnable,
            msrs: vec![
                MsrTweakState {
                    index: 0xC000_0080,
                    bit: 21,
                    state: true
                }
            ],
            readable: true,
        },
        false,
    )
}

pub fn enable_upper_address_ignore<'a>() -> Tweak<'a> {
    Tweak::msr_tweak(
        "Enable Upper Address Ignore",
        "Allows software to use upper bits of virtual addresses as metadata tags by suppressing canonical address checks. This can improve performance in specialized memory management scenarios while maintaining memory protection.",
        TweakCategory::Cpu,
        MSRTweak {
            id: TweakId::EnableUpperAddressIgnore,
            msrs: vec![
                MsrTweakState {
                    index: 0xC000_0080,
                    bit: 20,
                    state: true,
                },
            ],
            readable: true,
        },
        false,
    )
}

pub fn enable_interruptible_wbinvd<'a>() -> Tweak<'a> {
    Tweak::msr_tweak(
        "Enable Interruptible WBINVD",
        "Makes the Write-Back and Invalidate Cache (WBINVD) instruction interruptible, reducing system stall time during cache flushes. This can significantly improve system responsiveness during cache maintenance operations.",
        TweakCategory::Cpu,
        MSRTweak {
            id: TweakId::EnableInterruptibleWbinvd,
            msrs: vec![
                MsrTweakState {
                    index: 0xC000_0080,
                    bit: 18,
                    state: true,
                },
            ],
            readable: true,
        },
        false,
    )
}

pub fn translation_cache_extension_enable<'a>() -> Tweak<'a> {
    Tweak::msr_tweak(
        "Translation Cache Extension Enable",
        "Optimizes INVLPG instruction behavior to only invalidate PDC entries related to the specified linear address instead of all entries. This can improve performance in scenarios with frequent TLB invalidations.",
        TweakCategory::Cpu,
        MSRTweak {
            id: TweakId::EnableTranslationCacheExtension,
            msrs: vec![
                MsrTweakState {
                    index: 0xC000_0080,
                    bit: 15,
                    state: true
                }
            ],
            readable: true,
        },
        false,
    )
}

pub fn enable_fast_fxsave_frstor<'a>() -> Tweak<'a> {
    Tweak::msr_tweak(
        "Enable Fast FXSAVE/FRSTOR",
        "Enables optimized handling of FPU and SSE state save/restore operations in 64-bit mode. This can significantly improve context switch performance in applications using floating-point or SIMD instructions.",
        TweakCategory::Cpu,
        MSRTweak {
            id: TweakId::EnableFastFxsaveFrstor,
            msrs: vec![
                MsrTweakState {
                    index: 0xC000_0080,
                    bit: 14,
                    state: true,
                },
            ],
            readable: true,
        },
        false,
    )
}

pub fn disable_secure_virtual_machine<'a>() -> Tweak<'a> {
    Tweak::msr_tweak(
        "Disable Secure Virtual Machine",
        "Disables AMD's Secure Virtual Machine (SVM) technology. While this can reduce virtualization overhead, it should only be used in systems that don't require hardware-assisted virtualization or security features provided by SVM.",
        TweakCategory::Cpu,
        MSRTweak {
            id: TweakId::DisableSecureVirtualMachine,
            msrs: vec![MsrTweakState {
                index: 0xC000_0080,
                bit: 12,
                state: false,
            }],
            readable: true,
        },
        false,
    )
}

// pub fn long_mode_enable<'a>() -> Tweak<'a> {
//     Tweak::msr_tweak(
//         "Enable Long Mode",
//         "Enables 64-bit long mode operation, allowing access to 64-bit instructions, registers, and addressing. This is fundamental for modern 64-bit operating systems and applications.",
//         TweakCategory::Cpu,
//         MSRTweak {
//             id: TweakId::LongModeEnable,
//             msrs: vec![MsrTweakState {
//                 index: 0xC000_0080,
//                 bit: 8,
//                 state: true,
//             }],
//             readable: true,
//         },
//         false,
//     )
// }

// pub fn system_call_extension_enable<'a>() -> Tweak<'a> {
//     Tweak::msr_tweak(
//         "Enable System Call Extension",
//         "Enables fast SYSCALL/SYSRET instructions for efficient system calls in 64-bit mode. This provides lower latency than traditional INT-based system calls, improving overall system performance.",
//         TweakCategory::Cpu,
//         MSRTweak {
//             id: TweakId::SystemCallExtensionEnable,
//             msrs: vec![MsrTweakState {
//                 index: 0xC000_0080,
//                 bit: 0,
//                 state: true,
//             }],
//             readable: true,
//         },
//         false,
//     )
// }

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

pub fn enable_aggressive_prefetch_profile<'a>() -> Tweak<'a> {
    Tweak::msr_tweak(
        "Enable Aggressive Prefetch Profile",
        "Configures the CPU's prefetcher to its most aggressive setting (Level 3) and ensures prefetching is enabled. This can significantly improve performance in applications with predictable memory access patterns, but may increase power consumption and potentially harm performance with random access patterns.",
        TweakCategory::Cpu,
        MSRTweak {
            id: TweakId::AggressivePrefetchProfile,
            msrs: vec![
                MsrTweakState {
                    index: 0xC000_0108,
                    bit: 6,
                    state: true,
                },
                MsrTweakState {
                    index: 0xC000_0108,
                    bit: 7,
                    state: true,
                },
                MsrTweakState {
                    index: 0xC000_0108,
                    bit: 8,
                    state: true,
                },
                MsrTweakState {
                    index: 0xC000_0108,
                    bit: 9,
                    state: false,
                },
            ],
            readable: true,
        },
        false,
    )
}

pub fn disable_up_down_prefetcher<'a>() -> Tweak<'a> {
    Tweak::msr_tweak(
        "Disable Up-Down Prefetcher",
        "Disables the L2 cache prefetcher that predicts whether to fetch the next or previous cache line based on memory access history. This can be beneficial in workloads with random access patterns where prefetching might waste bandwidth.",
        TweakCategory::Cpu,
        MSRTweak {
            id: TweakId::DisableUpDownPrefetcher,
            msrs: vec![MsrTweakState {
                index: 0xC000_0108,
                bit: 5,
                state: true,
            }],
            readable: true,
        },
        false,
    )
}

pub fn disable_l2_stream_prefetcher<'a>() -> Tweak<'a> {
    Tweak::msr_tweak(
        "Disable L2 Stream Prefetcher",
        "Disables the L2 cache's stream prefetcher that analyzes memory access patterns to prefetch sequential cache lines. Can improve performance in workloads with random access patterns or when memory bandwidth is a bottleneck.",
        TweakCategory::Cpu,
        MSRTweak {
            id: TweakId::DisableL2StreamPrefetcher,
            msrs: vec![MsrTweakState {
                index: 0xC000_0108,
                bit: 3,
                state: true,
            }],
            readable: true,
        },
        false,
    )
}

pub fn disable_l1_region_prefetcher<'a>() -> Tweak<'a> {
    Tweak::msr_tweak(
        "Disable L1 Region Prefetcher",
        "Disables the L1 cache prefetcher that detects and prefetches from localized memory regions based on access patterns. This can be beneficial in applications with scattered memory access patterns where spatial locality is poor.",
        TweakCategory::Cpu,
        MSRTweak {
            id: TweakId::DisableL1RegionPrefetcher,
            msrs: vec![MsrTweakState {
                index: 0xC000_0108,
                bit: 2,
                state: true,
            }],
            readable: true,
        },
        false,
    )
}

pub fn disable_l1_stride_prefetcher<'a>() -> Tweak<'a> {
    Tweak::msr_tweak(
        "Disable L1 Stride Prefetcher",
        "Disables the L1 cache prefetcher that detects constant-stride memory access patterns. While this prefetcher typically improves performance in array traversals and matrix operations, disabling it can be beneficial in workloads with irregular memory access patterns or when trying to reduce cache pollution.",
        TweakCategory::Cpu,
        MSRTweak {
            id: TweakId::DisableL1StridePrefetcher,
            msrs: vec![MsrTweakState {
                index: 0xC000_0108,
                bit: 1,
                state: true,
            }],
            readable: true,
        },
        false,
    )
}

pub fn disable_l1_stream_prefetcher<'a>() -> Tweak<'a> {
    Tweak::msr_tweak(
        "Disable L1 Stream Prefetcher",
        "Disables the L1 cache's stream prefetcher that predicts and prefetches sequential memory access patterns. Can improve performance in workloads with random access patterns by preventing unnecessary prefetches and reducing memory bandwidth consumption.",
        TweakCategory::Cpu,
        MSRTweak {
            id: TweakId::DisableL1StreamPrefetcher,
            msrs: vec![MsrTweakState {
                index: 0xC000_0108,
                bit: 0,
                state: true,
            }],
            readable: true,
        },
        false,
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
            msrs: vec![MsrTweakState {
                index: 0xC001_0010,
                bit: 19,
                state: true,
            }],
            readable: true,
        },
        false,
    )
}

pub fn enable_mtrr_fixed_dram_attributes<'a>() -> Tweak<'a> {
    Tweak::msr_tweak(
        "Enable MTRR Fixed DRAM Attributes",
        "Enables the RdDram and WrDram attributes in fixed Memory Type Range Registers (MTRRs). This allows separate read and write caching policies for fixed memory ranges, enabling more sophisticated memory access optimizations.",
        TweakCategory::Cpu,
        MSRTweak {
            id: TweakId::EnableMtrrFixedDramAttributes,
            msrs: vec![
                MsrTweakState {
                    index: 0xC001_0010,
                    bit: 18,
                    state: true,
                },
            ],
            readable: true,
        },
        false,
    )
}

// MSRC001_0015 [Hardware Configuration] (Core::X86::Msr::HWCR)
// Reset: 0000_0000_0100_6010h.
// _ccd[11:0]_lthree0_core[15:0]_thread[1:0]; MSRC001_0015
// Bits Description
// 63:36 Reserved.
// 35 CpuidFltEn. Read-write. Reset: 0. 1=Executing CPUID outside of SMM and with CPL > 0 results in #GP.
// 34 DownGradeFp512ToFP256. Read-write. Reset: 0. 1=Downgrade FP512 performance to look more like FP256
// performance.
// 33 SmmPgCfgLock. Read-write. Reset: 0. 1=SMM page config locked. Error-on-write-1 if not in SMM mode. RSM
// unconditionally clears Core::X86::Msr::HWCR[SmmPgCfgLock].
// 32:31 Reserved.
// 30 IRPerfEn: enable instructions retired counter. Read-write. Reset: 0. 1=Enable Core::X86::Msr::IRPerfCount.
// 29:28 Reserved.
// 27 EffFreqReadOnlyLock: read-only effective frequency counter lock. Write-1-only. Reset: 0. Init: BIOS,1.
// 1=Core::X86::Msr::MPerfReadOnly, Core::X86::Msr::APerfReadOnly and Core::X86::Msr::IRPerfCount are
// read-only.
// 26 EffFreqCntMwait: effective frequency counting during mwait. Read-write. Reset: 0. 0=The registers do not
// increment. 1=The registers increment. Specifies whether Core::X86::Msr::MPERF and Core::X86::Msr::APERF
// increment while the core is in the monitor event pending state. See 2.1.6 [Effective Frequency].
// 25 CpbDis: core performance boost disable. Read-write. Reset: 0. 0=CPB is requested to be enabled. 1=CPB is
// disabled. Specifies whether core performance boost is requested to be enabled or disabled. If core performance
// boost is disabled while a core is in a boosted P-state, the core automatically transitions to the highest performance
// non-boosted P-state.
// 24 TscFreqSel: TSC frequency select. Read-only. Reset: 1. 1=The TSC increments at the P0 frequency.
// 23:22 Reserved.
// 21 LockTscToCurrentP0: lock the TSC to the current P0 frequency. Read-write. Reset: 0. 0=The TSC will count
// at the P0 frequency. 1=The TSC frequency is locked to the current P0 frequency at the time this bit is set and
// remains fixed regardless of future changes to the P0 frequency.
// 20 IoCfgGpFault: IO-space configuration causes a GP fault. Read-write. Reset: 0. 1=IO-space accesses to
// configuration space cause a GP fault. The fault is triggered if any part of the IO Read/Write address range is
// between CF8h and CFFh, inclusive. These faults only result from single IO instructions, not to string and REP IO
// instructions. This fault takes priority over the IO trap mechanism described by
// Core::X86::Msr::SMI_ON_IO_TRAP_CTL_STS.
// 19 Reserved.
// 18 McStatusWrEn: machine check status write enable. Read-write. Reset: 0. 0=MCA_STATUS registers are
// readable; writing a non-zero pattern to these registers causes a general protection fault. 1=MCA_STATUS
// registers are Read-write, including Reserved fields; do not cause general protection faults; such writes update all
// implemented bits in these registers; All fields of all threshold registers are Read-write when accessed from MSR
// space, including Locked, except BlkPtr which is always Read-only; McStatusWrEn does not change the access
// type for the thresholding registers accessed via configuration space.
// Description: McStatusWrEn can be used to debug machine check exception and interrupt handlers.
// Independent of the value of this bit, the processor may enforce Write-Ignored behavior on MCA_STATUS
// registers depending on platform settings.
// See 3.1 [Machine Check Architecture].
// 17 Wrap32Dis: 32-bit address wrap disable. Read-write. Reset: 0. 1=Disable 32-bit address wrapping. Software
// can use Wrap32Dis to access physical memory above 4 Gbytes without switching into 64-bit mode. To do so,
// software should write a greater-than 4 Gbyte address to Core::X86::Msr::FS_BASE and
// Core::X86::Msr::GS_BASE. Then it would address ±2 Gbytes from one of those bases using normal memory
// reference instructions with a FS or GS override prefix. However, the INVLPG, FST, and SSE store instructions
// generate 32-bit addresses in legacy mode, regardless of the state of Wrap32Dis.
// 16:15 Reserved.
// 14 RsmSpCycDis: RSM special bus cycle disable. Reset: 1. Init: BIOS,1. 0=A link special bus cycle, SMIACK, is
// generated on a resume from SMI.
// AccessType: Core::X86::Msr::HWCR[SmmLock] ? Read-only : Read-write.
// 13 SmiSpCycDis: SMI special bus cycle disable. Reset: 1. Init: BIOS,1. 0=A link special bus cycle, SMIACK, is
// generated when an SMI interrupt is taken.
// AccessType: Core::X86::Msr::HWCR[SmmLock] ? Read-only : Read-write.
// 12:11 Reserved.
// 10 MonMwaitUserEn: MONITOR/MWAIT user mode enable. Read-write. Reset: 0. 0=The MONITOR and
// MWAIT instructions are supported only in privilege level 0; these instructions in privilege levels 1 to 3 cause a
// #UD exception. 1=The MONITOR and MWAIT instructions are supported in all privilege levels. The state of this
// bit is ignored if MonMwaitDis is set.
// 9 MonMwaitDis: MONITOR and MWAIT disable. Read-write. Reset: 0. 1=The MONITOR, MWAIT,
// MONITORX, and MWAITX opcodes become invalid. This affects what is reported back through
// Core::X86::Cpuid::FeatureIdEcx[Monitor] and Core::X86::Cpuid::FeatureExtIdEcx[MwaitExtended].
// 8 IgnneEm: IGNNE port emulation enable. Read-write. Reset: 0. 1=Enable emulation of IGNNE port.
// 7 AllowFerrOnNe: allow FERR on NE. Read-write. Reset: 0. 0=Disable FERR signalling when generating an x87
// floating point exception (when CR0[NE] is set). 1=FERR is signaled on any x87 floating point exception,
// regardless of CR0[NE].
// 6:5 Reserved.
// 4 INVDWBINVD: INVD to WBINVD conversion. Read,Error-on-write-0. Reset: 1. 1=Convert INVD to
// WBINVD.
// Description: This bit is required to be set for normal operation when any of the following are true:
// • An L2 is shared by multiple threads.
// • An L3 is shared by multiple cores.
// • CC6 is enabled.
// • Probe filter is enabled.
// 3 TlbCacheDis: cacheable memory disable. Read-write. Reset: 0. 1=Disable performance improvement that
// assumes that the PML4, PDP, PDE and PTE entries are in cacheable WB DRAM.
// Description: Operating systems that maintain page tables in any other memory type must set the TlbCacheDis bit
// to insure proper operation. Operating system should do a full TLB flush before and after any changes to this bit
// value.
// • TlbCacheDis does not override the memory type specified by the SMM ASeg and TSeg memory regions
// controlled by Core::X86::Msr::SMMAddr Core::X86::Msr::SMMMask.
// 2:1 Reserved.
// 0 SmmLock: SMM code lock. Read,Write-1-only. Reset: 0. Init: BIOS,1. 1=SMM code in the ASeg and TSeg
// range and the SMM registers are Read-only and SMI interrupts are not intercepted in SVM. See 2.1.13.1.10
// [Locking SMM].

pub fn downgrade_fp512_to_fp256<'a>() -> Tweak<'a> {
    Tweak::msr_tweak(
        "Downgrade FP512 to FP256",
        "Reduces FP512 (AVX-512) performance to match FP256 (AVX2) levels. This can prevent frequency throttling that typically occurs during AVX-512 operations, potentially improving overall system performance when full AVX-512 throughput isn't critical.",
        TweakCategory::Cpu,
        MSRTweak {
            id: TweakId::DowngradeFp512ToFp256,
            msrs: vec![MsrTweakState {
                index: 0xC001_0015,
                bit: 34,
                state: true,
            }],
            readable: true,
        },
        false,
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
            msrs: vec![
                MsrTweakState {
                    index: 0x0000_0DA0,
                    bit: 12,
                    state: false,
                },
            ],
            readable: true,
        },
        false,
    )
}

pub fn disable_mca_status_write_enable<'a>() -> Tweak<'a> {
    Tweak::msr_tweak(
        "Disable MCA Status Write Enable",
        "Disables write access to MCA status registers to reduce overhead related to error handling for enhanced performance.",
        TweakCategory::Cpu,
        MSRTweak {
            id: TweakId::DisableMcaStatusWriteEnable,
            msrs: vec![
                MsrTweakState {
                    index: 0xC001_0015, // HWCR MSR index
                    bit: 18,
                    state: false, // Disable McStatusWrEn
                },
            ],
            readable: true,
        },
        false, // Does not require reboot
    )
}

pub fn disable_tlb_cache<'a>() -> Tweak<'a> {
    Tweak::msr_tweak(
        "Disable TLB Cache",
        "Disables the assumption that TLB entries are cached, potentially increasing memory access latency for aggressive memory handling optimizations.",
        TweakCategory::Cpu,
        MSRTweak {
            id: TweakId::DisableTlbCache,
            msrs: vec![
                MsrTweakState {
                    index: 0xC001_0015, // HWCR MSR index
                    bit: 3,
                    state: true, // Disable TlbCacheDis
                },
            ],
            readable: true,
        },
        false, // Does not require reboot
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
        "Enable L3 Code-Data Prioritization",
        "Activates Code and Data Prioritization Technology (CDP) in the L3 cache, allowing separate control over code and data placement. This can improve performance by optimizing cache utilization based on workload characteristics and preventing code/data contention.",
        TweakCategory::Cpu,
        MSRTweak {
            id: TweakId::EnableL3CodeDataPrioritization,
            msrs: vec![
                MsrTweakState {
                    index: 0x0000_0C81,
                    bit: 0,
                    state: true,
                },
            ],
            readable: true,
        },
        false,
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
            msrs: vec![MsrTweakState {
                index: 0xC001_1020,
                bit: 28,
                state: true,
            }],
            readable: true,
        },
        false,
    )
}

pub fn disable_redirect_for_return<'a>() -> Tweak<'a> {
    Tweak::msr_tweak(
        "Disable Redirect for Return",
        "Disables CPU's Return Stack Buffer (RSB) redirect mechanism. This can improve performance in specific workloads by preventing unnecessary speculation redirects, but may slightly impact branch prediction accuracy.",
        TweakCategory::Cpu,
        MSRTweak {
            id: TweakId::DisableRedirectForReturn,
            msrs: vec![
                MsrTweakState {
                    index: 0xC001_1029,
                    bit: 14,
                    state: true,
                },
            ],
            readable: true,
        },
        false,
    )
}

pub fn disable_opcache<'a>() -> Tweak<'a> {
    Tweak::msr_tweak(
        "Disable OpCache",
        "Disables the CPU's Op Cache, forcing instructions to be decoded from L1 instruction cache. While this typically reduces performance, it can help in specific debugging scenarios or when dealing with self-modifying code.",
        TweakCategory::Cpu,
        MSRTweak {
            id: TweakId::DisableOpCache,
            msrs: vec![MsrTweakState {
                index: 0xC001_1021,
                bit: 5,
                state: true,
            }],
            readable: true,
        },
        false,
    )
}

pub fn set_cpu_speculative_store_modes_more_speculative<'a>() -> Tweak<'a> {
    Tweak::msr_tweak(
        "CPU Speculative Store Modes",
        "Enables more aggressive store-to-load forwarding and memory disambiguation. Can improve performance in memory-intensive workloads at the cost of potentially higher power consumption and slightly increased risk of memory ordering violations.",
        TweakCategory::Cpu,
        MSRTweak {
            id: TweakId::SpeculativeStoreModes,
            msrs: vec![
                MsrTweakState {
                    index: 0xC001_10EC,
                    bit: 0,
                    state: true,
                },
                MsrTweakState {
                    index: 0xC001_10E5,
                    bit: 26,
                    state: true,
                },
            ],
            readable: true,
        },
        false,
    )
}

pub fn disable_avx512<'a>() -> Tweak<'a> {
    Tweak::msr_tweak(
        "Disable AVX512",
        "Disables AVX-512 instruction set extensions. This prevents frequency downclocking that occurs during AVX-512 operations and can improve overall system performance when AVX-512 instructions are not crucial for workload performance.",
        TweakCategory::Cpu,
        MSRTweak {
            id: TweakId::DisableAvx512,
            msrs: vec![
                MsrTweakState {
                    index: 0xC001_1022,
                    bit: 16,
                    state: false,
                },
                MsrTweakState {
                    index: 0xC001_1022,
                    bit: 17,
                    state: false,
                },
                MsrTweakState {
                    index: 0xC001_1022,
                    bit: 21,
                    state: false,
                },
                MsrTweakState {
                    index: 0xC001_1022,
                    bit: 28,
                    state: false,
                },
                MsrTweakState {
                    index: 0xC001_1022,
                    bit: 30,
                    state: false,
                },
                MsrTweakState {
                    index: 0xC001_1022,
                    bit: 31,
                    state: false,
                },
            ],
            readable: true,
        },
        false,
    )
}

pub fn disable_fast_short_rep_movsb<'a>() -> Tweak<'a> {
    Tweak::msr_tweak(
        "Disable Fast Short REP MOVSB",
        "Disables optimized handling of short REP MOVSB instructions. While this optimization typically improves small memory copy operations, disabling it can be beneficial when the overhead of enabling the optimization exceeds its benefits in specific workloads.",
        TweakCategory::Cpu,
        MSRTweak {
            id: TweakId::DisableFastShortRepMovsb,
            msrs: vec![
                MsrTweakState {
                    index: 0xC001_10DF,
                    bit: 36,
                    state: false,
                },
            ],
            readable: true,
        },
        false,
    )
}

pub fn disable_enhanced_rep_movsb_stosb<'a>() -> Tweak<'a> {
    Tweak::msr_tweak(
        "Disable Enhanced REP MOVSB/STOSB",
        "Disables enhanced string operation optimizations for REP MOVSB/STOSB instructions. Can improve performance in workloads where the overhead of enabling these optimizations exceeds their benefits, particularly with small or unaligned memory operations.",
        TweakCategory::Cpu,
        MSRTweak {
            id: TweakId::DisableEnhancedRepMovsbStosb,
            msrs: vec![
                MsrTweakState {
                    index: 0xC001_1002,
                    bit: 9,
                    state: false,
                },
            ],
            readable: true,
        },
        false,
    )
}

pub fn disable_rep_mov_stos_streaming<'a>() -> Tweak<'a> {
    Tweak::msr_tweak(
        "Disable REP-MOV/STOS Streaming",
        "Disables streaming optimization for REP MOV/STOS instructions. This can improve performance in scenarios where memory operations need to maintain cache coherency or when data is likely to be immediately reused.",
        TweakCategory::Cpu,
        MSRTweak {
            id: TweakId::DisableRepMovStosStreaming,
            msrs: vec![MsrTweakState {
                index: 0xC001_1000,
                bit: 15,
                state: true,
            }],
            readable: true,
        },
        false,
    )
}

// pub fn disable_pss<'a>() -> Tweak<'a> {
//     Tweak::msr_tweak(
//         "Disable PSS",
//         "Disables Performance Supported States (PSS) ACPI functionality. This can reduce latency in frequency transitions by limiting the available P-states, potentially improving performance in workloads sensitive to frequency scaling delays.",
//         TweakCategory::Cpu,
//         MSRTweak {
//             id: TweakId::DisablePss,
//             msrs: vec![
//                 MsrTweakState {
//                     index: 0xC001_02B1,
//                     bit: 0,
//                     state: false,
//                 },
//             ],
//             readable: true,
//         },
//         false,
//     )
// }

pub fn disable_core_watchdog_timer<'a>() -> Tweak<'a> {
    Tweak::msr_tweak(
        "Disable Core Watchdog Timer",
        "Disables the CPU core watchdog timer mechanism. This can reduce overhead from periodic timer interrupts and improve performance in scenarios where system stability monitoring is not critical.",
        TweakCategory::Cpu,
        MSRTweak {
            id: TweakId::DisableCoreWatchdogTimer,
            msrs: vec![
                MsrTweakState {
                    index: 0xC001_0074,
                    bit: 0,
                    state: false,
                },
                MsrTweakState {
                    index: 0xC001_0074,
                    bit: 3,
                    state: false,
                },
            ],
            readable: true,
        },
        false,
    )
}

pub fn disable_platform_first_error_handling<'a>() -> Tweak<'a> {
    Tweak::msr_tweak(
        "Disable Platform First Error Handling",
        "Disables the platform's first error handling mechanism. This reduces overhead from error checking and handling routines, potentially improving performance in stable systems where comprehensive error handling is not critical.",
        TweakCategory::Cpu,
        MSRTweak {
            id: TweakId::DisablePlatformFirstErrorHandling,
            msrs: vec![
                MsrTweakState {
                    index: 0xC000_0410,
                    bit: 5,
                    state: false,
                },
                MsrTweakState {
                    index: 0xC000_0410,
                    bit: 12,
                    state: false,
                },
            ],
            readable: true,
        },
        false,
    )
}
