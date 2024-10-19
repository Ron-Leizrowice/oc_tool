// src/tweaks/msr/mod.rs

use method::{MSRTweak, MsrTweakState};

use super::{Tweak, TweakCategory, TweakId};

pub mod method;

pub fn all_msr_tweaks<'a>() -> Vec<(TweakId, Tweak<'a>)> {
    vec![
        (TweakId::DowngradeFp512ToFp256, downgrade_fp512_to_fp256()),
        (
            TweakId::DisableRsmSpecialBusCycle,
            disable_rsm_special_bus_cycle(),
        ),
        (
            TweakId::DisableSmiSpecialBusCycle,
            disable_smi_special_bus_cycle(),
        ),
        (
            TweakId::DisablePredictiveStoreForwarding,
            disable_predictive_store_forwarding(),
        ),
        (
            TweakId::DisableSpeculativeStoreBypass,
            disable_speculative_store_bypass(),
        ),
        (
            TweakId::DisableSingleThreadIndirectBranchPredictor,
            disable_single_thread_indirect_branch_predictor(),
        ),
        (
            TweakId::DisableIndirectBranchRestrictionSpeculation,
            disable_indirect_branch_restriction_speculation(),
        ),
        (
            TweakId::SelectiveBranchPredictorBarrier,
            selective_branch_predictor_barrier(),
        ),
        (
            TweakId::IndirectBranchPredictionBarrier,
            indirect_branch_prediction_barrier(),
        ),
        (TweakId::AutomaticIbrsEnable, automatic_ibrs_enable()),
        (
            TweakId::UpperAddressIgnoreEnable,
            upper_address_ignore_enable(),
        ),
        (
            TweakId::TranslationCacheExtensionEnable,
            translation_cache_extension_enable(),
        ),
        (TweakId::FastFxsaveFrstorEnable, fast_fxsave_frstor_enable()),
        (
            TweakId::DisableSecureVirtualMachine,
            disable_secure_virtual_machine(),
        ),
        (TweakId::DisableNoExecutePage, disable_no_execute_page()),
        (TweakId::LongModeEnable, long_mode_enable()),
        (
            TweakId::SystemCallExtensionEnable,
            system_call_extension_enable(),
        ),
        (
            TweakId::AggressivePrefetchProfile,
            aggressive_prefetch_profile(),
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
            TweakId::DisableHostMultiKeyEncryption,
            disable_host_multi_key_encryption(),
        ),
        (
            TweakId::DisableSecureNestedPaging,
            disable_secure_nested_paging(),
        ),
        (
            TweakId::EnableTopOfMemory2MemoryTypeWriteBack,
            enable_top_of_mem2mem_type_write_back(),
        ),
        (
            TweakId::DisableSecureMemoryEncryption,
            disable_secure_memory_encryption(),
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
            TweakId::EnableMtrrTopOfMemory2,
            enable_mtrr_top_of_memory_2(),
        ),
        (TweakId::EnableMtrrVariableDram, enable_mtrr_variable_dram()),
    ]
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

pub fn downgrade_fp512_to_fp256<'a>() -> Tweak<'a> {
    Tweak::msr_tweak(
        "Downgrade FP512 to FP256",
        "Downgrades FP512 performance to look more like FP256 performance.",
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
        false, // does not require reboot
    )
}

pub fn disable_rsm_special_bus_cycle<'a>() -> Tweak<'a> {
    Tweak::msr_tweak(
        "Disable RSM Special Bus Cycle",
        "Disables the RSM special bus cycle, which is used to read the system management mode (SMM) memory area.",
        TweakCategory::Cpu,
        MSRTweak {
            id: TweakId::DisableRsmSpecialBusCycle,
            msrs: vec![
                MsrTweakState {
                    index: 0xC001_0015,
                    bit: 30,
                    state: true
                }
            ],
            readable: true,
        },
        false, // does not require reboot
    )
}

pub fn disable_smi_special_bus_cycle<'a>() -> Tweak<'a> {
    Tweak::msr_tweak(
        "Disable SMI Special Bus Cycle",
        "Disables the SMI special bus cycle, which is used to read the system management mode (SMM) memory area.",
        TweakCategory::Cpu,
        MSRTweak {
            id: TweakId::DisableSmiSpecialBusCycle,
            msrs: vec![
                MsrTweakState {
                    index: 0xC001_0015,
                    bit: 31,
                    state: true
                }
            ],
            readable: true,
        },
        false, // does not require reboot
    )
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

pub fn disable_predictive_store_forwarding<'a>() -> Tweak<'a> {
    Tweak::msr_tweak(
        "Disable Predictive Store Forwarding",
        "Disables Predictive Store Forwarding (PSFD) to prevent speculative execution of store instructions from forwarding data to subsequent load instructions.",
        TweakCategory::Cpu,
        MSRTweak {
            id: TweakId::DisablePredictiveStoreForwarding,
            msrs : vec![
                MsrTweakState {
                    index: 0x0000_0048,
                    bit: 7,
                    state: true
                }
            ],
            readable: true,
        },
        false, // does not require reboot
    )
}

pub fn disable_speculative_store_bypass<'a>() -> Tweak<'a> {
    Tweak::msr_tweak(
        "Disable Speculative Store Bypass",
        "Disables Speculative Store Bypass (SSBD) to prevent speculative execution of store instructions from bypassing the store buffer.",
        TweakCategory::Cpu,
        MSRTweak {
            id: TweakId::DisableSpeculativeStoreBypass,
            msrs : vec![
                MsrTweakState {
                    index: 0x0000_0048,
                    bit: 2,
                    state: true
                }
            ],
            readable: true,
        },
        false, // does not require reboot
    )
}

pub fn disable_single_thread_indirect_branch_predictor<'a>() -> Tweak<'a> {
    Tweak::msr_tweak(
        "Disable Single Thread Indirect Branch Predictor",
        "Disables the Single Thread Indirect Branch Predictor (STIBP) to prevent indirect branch prediction across threads.",
        TweakCategory::Cpu,
        MSRTweak {
            id: TweakId::DisableSingleThreadIndirectBranchPredictor,
            msrs: vec![
                MsrTweakState {
                    index: 0x0000_0048,
                    bit: 1,
                    state: true
                }
            ],
            readable: true,
        },
        false, // does not require reboot
    )
}

pub fn disable_indirect_branch_restriction_speculation<'a>() -> Tweak<'a> {
    Tweak::msr_tweak(
        "Disable Indirect Branch Restriction Speculation",
        "Disables Indirect Branch Restriction Speculation (IBRS) to prevent speculation of indirect branches.",
        TweakCategory::Cpu,
        MSRTweak {
            id: TweakId::DisableIndirectBranchRestrictionSpeculation,
            msrs: vec![
                MsrTweakState {
                    index: 0x0000_0048,
                    bit: 0,
                    state: true
                }
            ],
            readable: true,
        },
        false, // does not require reboot
    )
}

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

pub fn selective_branch_predictor_barrier<'a>() -> Tweak<'a> {
    Tweak::msr_tweak(
        "Selective Branch Predictor Barrier",
        "Initiates a Selective Branch Predictor Barrier (SBPB) to prevent the branch predictor from being influenced by indirect branches.",
        TweakCategory::Cpu,
        MSRTweak {
            id: TweakId::SelectiveBranchPredictorBarrier,
            msrs: vec![
                MsrTweakState {
                    index: 0x0000_0049,
                    bit: 7,
                    state: true
                }
            ],
            readable: false,
        },
        false, // does not require reboot
    )
}

pub fn indirect_branch_prediction_barrier<'a>() -> Tweak<'a> {
    Tweak::msr_tweak(
        "Indirect Branch Prediction Barrier",
        "Initiates an Indirect Branch Prediction Barrier (IBPB) to prevent the branch predictor from being influenced by indirect branches.",
        TweakCategory::Cpu,
        MSRTweak {
            id: TweakId::IndirectBranchPredictionBarrier,
            msrs: vec![
                MsrTweakState {
                    index: 0x0000_0049,
                    bit: 0,
                    state: true
                }
            ],
            readable: false,
        },
        false, // does not require reboot
    )
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
        "Automatic IBRS Enable",
        "Enables Automatic IBRS (Indirect Branch Restricted Speculation) to automatically enable IBRS protection for processes running at CPL 0 or SEV-SNP.",
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
        false, // does not require reboot
    )
}

pub fn upper_address_ignore_enable<'a>() -> Tweak<'a> {
    Tweak::msr_tweak(
        "Upper Address Ignore Enable",
        "Enables Upper Address Ignore to suppress canonical faults for most data access virtual addresses, allowing software to use the upper bits of a virtual address as tags.",
        TweakCategory::Cpu,
        MSRTweak {
            id: TweakId::UpperAddressIgnoreEnable,
            msrs: vec![
                MsrTweakState {
                    index: 0xC000_0080,
                    bit: 20,
                    state: true
                }
            ],
            readable: true,
        },
        false, // does not require reboot
    )
}

pub fn translation_cache_extension_enable<'a>() -> Tweak<'a> {
    Tweak::msr_tweak(
        "Translation Cache Extension Enable",
        "Enables Translation Cache Extension to invalidate PDC entries related to the linear address of the INVLPG instruction.",
        TweakCategory::Cpu,
        MSRTweak {
            id: TweakId::TranslationCacheExtensionEnable,
            msrs: vec![
                MsrTweakState {
                    index: 0xC000_0080,
                    bit: 15,
                    state: true
                }
            ],
            readable: true,
        },
        false, // does not require reboot
    )
}

pub fn fast_fxsave_frstor_enable<'a>() -> Tweak<'a> {
    Tweak::msr_tweak(
        "Fast FXSAVE/FRSTOR Enable",
        "Enables the fast FXSAVE/FRSTOR mechanism.",
        TweakCategory::Cpu,
        MSRTweak {
            id: TweakId::FastFxsaveFrstorEnable,
            msrs: vec![MsrTweakState {
                index: 0xC000_0080,
                bit: 14,
                state: true,
            }],
            readable: true,
        },
        false, // does not require reboot
    )
}

pub fn disable_secure_virtual_machine<'a>() -> Tweak<'a> {
    Tweak::msr_tweak(
        "Secure Virtual Machine Enable",
        "Enables Secure Virtual Machine (SVM) features.",
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
        false, // does not require reboot
    )
}

pub fn disable_no_execute_page<'a>() -> Tweak<'a> {
    Tweak::msr_tweak(
        "No-Execute Page Enable",
        "Enables the no-execute page protection feature.",
        TweakCategory::Cpu,
        MSRTweak {
            id: TweakId::DisableNoExecutePage,
            msrs: vec![MsrTweakState {
                index: 0xC000_0080,
                bit: 11,
                state: false,
            }],
            readable: true,
        },
        false, // does not require reboot
    )
}

pub fn long_mode_enable<'a>() -> Tweak<'a> {
    Tweak::msr_tweak(
        "Long Mode Enable",
        "Enables long mode.",
        TweakCategory::Cpu,
        MSRTweak {
            id: TweakId::LongModeEnable,
            msrs: vec![MsrTweakState {
                index: 0xC000_0080,
                bit: 8,
                state: true,
            }],
            readable: true,
        },
        false, // does not require reboot
    )
}

pub fn system_call_extension_enable<'a>() -> Tweak<'a> {
    Tweak::msr_tweak(
        "System Call Extension Enable",
        "Enables the SYSCALL and SYSRET instructions.",
        TweakCategory::Cpu,
        MSRTweak {
            id: TweakId::SystemCallExtensionEnable,
            msrs: vec![MsrTweakState {
                index: 0xC000_0080,
                bit: 0,
                state: true,
            }],
            readable: true,
        },
        false, // does not require reboot
    )
}

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

pub fn aggressive_prefetch_profile<'a>() -> Tweak<'a> {
    Tweak::msr_tweak(
        "Prefetch Aggressiveness Profile",
        "Selects a prefetch aggressiveness profile.",
        TweakCategory::Cpu,
        MSRTweak {
            id: TweakId::AggressivePrefetchProfile,
            msrs: vec![
                // master enable
                MsrTweakState {
                    index: 0xC000_0108,
                    bit: 6,
                    state: true,
                },
                // Set PrefetchAggressivenessProfile to Level 3 (most aggressive)
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
        false, // does not require reboot
    )
}

pub fn disable_up_down_prefetcher<'a>() -> Tweak<'a> {
    Tweak::msr_tweak(
        "Disable Up-Down Prefetcher",
        "Disables the prefetcher that uses memory access history to determine whether to fetch the next or previous line into L2 cache for all memory accesses.",
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
        false, // does not require reboot
    )
}

pub fn disable_l2_stream_prefetcher<'a>() -> Tweak<'a> {
    Tweak::msr_tweak(
        "Disable L2 Stream Prefetcher",
        "Disables the prefetcher that uses history of memory access patterns to fetch additional sequential lines into L2 cache.",
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
        false, // does not require reboot
    )
}

pub fn disable_l1_region_prefetcher<'a>() -> Tweak<'a> {
    Tweak::msr_tweak(
        "Disable L1 Region Prefetcher",
        "Disables the prefetcher that uses memory access history to fetch additional lines into L1 cache when the data access for a given instruction tends to be followed by a consistent pattern of other accesses within a localized region.",
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
        false, // does not require reboot
    )
}

pub fn disable_l1_stride_prefetcher<'a>() -> Tweak<'a> {
    Tweak::msr_tweak(
        "Disable L1 Stride Prefetcher",
        "Disables the stride prefetcher that uses memory access history of individual instructions to fetch additional lines into L1 cache when each access is a constant distance from the previous.",
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
        false, // does not require reboot
    )
}

pub fn disable_l1_stream_prefetcher<'a>() -> Tweak<'a> {
    Tweak::msr_tweak(
        "Disable L1 Stream Prefetcher",
        "Disables the stream prefetcher that uses history of memory access patterns to fetch additional sequential lines into L1 cache.",
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
        false, // does not require reboot
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
// Valid encodings are {00000b, Core::X86::Msr::MtrrFix_64K through Core::X86::Msr::MtrrFix_4K_7[2:0]}.
// Other Write values cause a GP(0).

pub fn disable_host_multi_key_encryption<'a>() -> Tweak<'a> {
    Tweak::msr_tweak(
        "Disable Host Multi-Key Encryption",
        "Disables Host Multi-Key Encryption (HMKEE).",
        TweakCategory::Cpu,
        MSRTweak {
            id: TweakId::DisableHostMultiKeyEncryption,
            msrs: vec![MsrTweakState {
                index: 0xC001_0010,
                bit: 26,
                state: false,
            }],
            readable: true,
        },
        false, // does not require reboot
    )
}

pub fn disable_secure_nested_paging<'a>() -> Tweak<'a> {
    Tweak::msr_tweak(
        "Disable Secure Nested Paging",
        "Disables Secure Nested Paging (SNP).",
        TweakCategory::Cpu,
        MSRTweak {
            id: TweakId::DisableSecureNestedPaging,
            msrs: vec![MsrTweakState {
                index: 0xC001_0010,
                bit: 24,
                state: false,
            }],
            readable: true,
        },
        false, // does not require reboot
    )
}

pub fn disable_secure_memory_encryption<'a>() -> Tweak<'a> {
    Tweak::msr_tweak(
        "Disable Secure Memory Encryption",
        "Disables Secure Memory Encryption (SME) and Secure Encrypted Virtualization (SEV) memory encryption.",
        TweakCategory::Cpu,
        MSRTweak {
            id: TweakId::DisableSecureMemoryEncryption,
            msrs: vec![
                MsrTweakState {
                index: 0xC001_0010,
                bit: 23,
                state: false,
            }],
            readable: true,
        },
        false, // does not require reboot
    )
}

pub fn enable_top_of_mem2mem_type_write_back<'a>() -> Tweak<'a> {
    Tweak::msr_tweak(
        "Disable Top of Mem2Mem Type Write Back",
        "Disables the top of memory 2 memory type write back.",
        TweakCategory::Cpu,
        MSRTweak {
            id: TweakId::EnableTopOfMemory2MemoryTypeWriteBack,
            msrs: vec![
                // enable Tom2ForceMemTypeWB
                MsrTweakState {
                    index: 0xC001_0010,
                    bit: 22,
                    state: true,
                },
                // enable MtrrDefTypeEn
                MsrTweakState {
                    index: 0x0000_02FF,
                    bit: 11,
                    state: true,
                },
            ],
            readable: true,
        },
        false, // does not require reboot
    )
}

pub fn enable_mtrr_top_of_memory_2<'a>() -> Tweak<'a> {
    Tweak::msr_tweak(
        "Enable MTRR Top of Memory 2",
        "Enables the MTRR top of memory 2.",
        TweakCategory::Cpu,
        MSRTweak {
            id: TweakId::EnableMtrrTopOfMemory2,
            msrs: vec![MsrTweakState {
                index: 0xC001_0010,
                bit: 21,
                state: true,
            }],
            readable: true,
        },
        false, // does not require reboot
    )
}

pub fn enable_mtrr_variable_dram<'a>() -> Tweak<'a> {
    Tweak::msr_tweak(
        "Enable MTRR Variable DRAM",
        "Enables the MTRR variable DRAM.",
        TweakCategory::Cpu,
        MSRTweak {
            id: TweakId::EnableMtrrVariableDram,
            msrs: vec![MsrTweakState {
                index: 0xC001_0010,
                bit: 20,
                state: true,
            }],
            readable: true,
        },
        false, // does not require reboot
    )
}

pub fn enable_mtrr_fixed_dram_modification<'a>() -> Tweak<'a> {
    Tweak::msr_tweak(
        "Enable MTRR Fixed DRAM Modification",
        "Enables the MTRR fixed RdDram and WrDram modification.",
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
        false, // does not require reboot
    )
}

pub fn enable_mtrr_fixed_dram_attributes<'a>() -> Tweak<'a> {
    Tweak::msr_tweak(
        "Enable MTRR Fixed DRAM Attributes",
        "Enables the MTRR fixed RdDram and WrDram attributes.",
        TweakCategory::Cpu,
        MSRTweak {
            id: TweakId::EnableMtrrFixedDramAttributes,
            msrs: vec![MsrTweakState {
                index: 0xC001_0010,
                bit: 18,
                state: true,
            }],
            readable: true,
        },
        false, // does not require reboot
    )
}
