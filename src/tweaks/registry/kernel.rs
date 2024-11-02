// src/tweaks/registry/kernel.rs
use indexmap::IndexMap;

use super::{
    method::{RegistryModification, RegistryTweak},
    Tweak, TweakCategory, TweakOption,
};
use crate::{tweaks::TweakId, utils::registry::RegistryKeyValue};

pub fn thread_dpc_disable<'a>() -> Tweak<'a> {
    Tweak::registry_tweak(
        "Thread DPC",
        "Controls whether Deferred Procedure Calls (DPCs) are processed in thread context. When disabled:\n\
        • Forces DPCs to run in interrupt context instead of dedicated DPC threads\n\
        • May reduce latency for interrupt processing\n\
        • Could improve performance in certain real-time scenarios",
        TweakCategory::Kernel,
        RegistryTweak {
            id: TweakId::ThreadDpcDisable,
            options: IndexMap::from_iter(vec![
                (
                    TweakOption::Enabled(false),
                    vec![RegistryModification {
                        path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\kernel",
                        key: "ThreadDpcEnable",
                        value: RegistryKeyValue::Dword(1),
                    }],
                ),
                (
                    TweakOption::Enabled(true),
                    vec![RegistryModification {
                        path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\kernel",
                        key: "ThreadDpcEnable",
                        value: RegistryKeyValue::Dword(0),
                    }],
                ),
            ]),
        },
        true, // requires reboot
    )
}

pub fn additional_kernel_worker_threads<'a>() -> Tweak<'a> {
    Tweak::registry_tweak(
        "Kernel Worker Threads",
        "Controls the number of additional worker threads in the Windows kernel thread pool beyond the default allocation. These threads handle:\n\
        • Critical threads: High-priority system operations and I/O requests\n\
        • Delayed threads: Background and maintenance tasks\n\
        Available options:\n\
        • Default (0): Windows manages thread count automatically\n\
        • Per Core: Adds one thread of each type per logical processor\n\
        • Maximum: Adds two threads of each type per logical processor",
        TweakCategory::Kernel,
        RegistryTweak {
            id: TweakId::AdditionalKernelWorkerThreads,
            options: IndexMap::from_iter(vec![
                (
                    TweakOption::Option("Default".to_string()),
                    vec![
                        RegistryModification {
                            path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\Executive",
                            key: "AdditionalCriticalWorkerThreads",
                            value: RegistryKeyValue::Dword(0),
                        },
                        RegistryModification {
                            path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\Executive",
                            key: "AdditionalDelayedWorkerThreads",
                            value: RegistryKeyValue::Dword(0),
                        },
                    ],
                ),
                (
                    TweakOption::Option("Per Core".to_string()),
                    vec![
                        RegistryModification {
                            path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\Executive",
                            key: "AdditionalCriticalWorkerThreads",
                            value: RegistryKeyValue::Dword(1),
                        },
                        RegistryModification {
                            path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\Executive",
                            key: "AdditionalDelayedWorkerThreads",
                            value: RegistryKeyValue::Dword(1),
                        },
                    ],
                ),
                (
                    TweakOption::Option("Maximum".to_string()),
                    vec![
                        RegistryModification {
                            path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\Executive",
                            key: "AdditionalCriticalWorkerThreads",
                            value: RegistryKeyValue::Dword(2),
                        },
                        RegistryModification {
                            path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\Executive",
                            key: "AdditionalDelayedWorkerThreads",
                            value: RegistryKeyValue::Dword(2),
                        },
                    ],
                ),
            ]),
        },
        true, // Actually requires reboot for kernel thread pool changes
    )
}

// TweakOption::Enabled(true)
//
// @Echo Off
// Title Kernel Tweaks
// cd %systemroot%\system32
// call :IsAdmin

// Reg.exe add "HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\kernel" /v "MaxDynamicTickDuration" /t REG_DWORD /d "10" /f
// Reg.exe add "HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\kernel" /v "MaximumSharedReadyQueueSize" /t REG_DWORD /d "128" /f
// Reg.exe add "HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\kernel" /v "BufferSize" /t REG_DWORD /d "32" /f
// Reg.exe add "HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\kernel" /v "IoQueueWorkItem" /t REG_DWORD /d "32" /f
// Reg.exe add "HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\kernel" /v "IoQueueWorkItemToNode" /t REG_DWORD /d "32" /f
// Reg.exe add "HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\kernel" /v "IoQueueWorkItemEx" /t REG_DWORD /d "32" /f
// Reg.exe add "HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\kernel" /v "IoQueueThreadIrp" /t REG_DWORD /d "32" /f
// Reg.exe add "HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\kernel" /v "ExTryQueueWorkItem" /t REG_DWORD /d "32" /f
// Reg.exe add "HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\kernel" /v "ExQueueWorkItem" /t REG_DWORD /d "32" /f
// Reg.exe add "HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\kernel" /v "IoEnqueueIrp" /t REG_DWORD /d "32" /f
// Reg.exe add "HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\kernel" /v "XMMIZeroingEnable" /t REG_DWORD /d "0" /f
// Reg.exe add "HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\kernel" /v "UseNormalStack" /t REG_DWORD /d "1" /f
// Reg.exe add "HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\kernel" /v "UseNewEaBuffering" /t REG_DWORD /d "1" /f
// Reg.exe add "HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\kernel" /v "StackSubSystemStackSize" /t REG_DWORD /d "65536" /f
// Exit

// :IsAdmin
// Reg.exe query "HKU\S-1-5-19\Environment"
// If Not %ERRORLEVEL% EQU 0 (
//  Cls & Echo You must have administrator rights to continue ...
//  Pause & Exit
// )
// Cls
// goto:eof

//
// TweakOption::Enabled(false)
//
// @Echo Off
// Title Revert Kernel Tweaks
// cd %systemroot%\system32
// call :IsAdmin

// Reg.exe delete "HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\kernel" /v "MaxDynamicTickDuration" /f
// Reg.exe delete "HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\kernel" /v "MaximumSharedReadyQueueSize" /f
// Reg.exe delete "HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\kernel" /v "BufferSize" /f
// Reg.exe delete "HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\kernel" /v "IoQueueWorkItem" /f
// Reg.exe delete "HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\kernel" /v "IoQueueWorkItemToNode" /f
// Reg.exe delete "HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\kernel" /v "IoQueueWorkItemEx" /f
// Reg.exe delete "HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\kernel" /v "IoQueueThreadIrp" /f
// Reg.exe delete "HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\kernel" /v "ExTryQueueWorkItem" /f
// Reg.exe delete "HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\kernel" /v "ExQueueWorkItem" /f
// Reg.exe delete "HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\kernel" /v "IoEnqueueIrp" /f
// Reg.exe delete "HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\kernel" /v "XMMIZeroingEnable" /f
// Reg.exe delete "HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\kernel" /v "UseNormalStack" /f
// Reg.exe delete "HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\kernel" /v "UseNewEaBuffering" /f
// Reg.exe delete "HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\kernel" /v "StackSubSystemStackSize" /f
// Exit

// :IsAdmin
// Reg.exe query "HKU\S-1-5-19\Environment"
// If Not %ERRORLEVEL% EQU 0 (
//     Cls & Echo You must have administrator rights to continue ...
//     Pause & Exit
// )
// Cls
// goto:eof

pub fn alchemy_kernel_tweak<'a>() -> Tweak<'a> {
    Tweak::registry_tweak(
        "Alchemy Kernel Tweak",
        "This tweak applies a variety of kernel settings to improve system responsiveness and performance.",
        TweakCategory::Kernel,
        RegistryTweak {
            id: TweakId::AlchemyKernelTweak,
            options: IndexMap::from_iter(vec![
                (
                    TweakOption::Enabled(false),
                    vec![
                        RegistryModification {
                            path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\kernel",
                            key: "MaxDynamicTickDuration",
                            value: RegistryKeyValue::Dword(10),
                        },
                        RegistryModification {
                            path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\kernel",
                            key: "MaximumSharedReadyQueueSize",
                            value: RegistryKeyValue::Dword(128),
                        },
                        RegistryModification {
                            path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\kernel",
                            key: "BufferSize",
                            value: RegistryKeyValue::Dword(32),
                        },
                        RegistryModification {
                            path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\kernel",
                            key: "IoQueueWorkItem",
                            value: RegistryKeyValue::Dword(32),
                        },
                        RegistryModification {
                            path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\kernel",
                            key: "IoQueueWorkItemToNode",
                            value: RegistryKeyValue::Dword(32),
                        },
                        RegistryModification {
                            path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\kernel",
                            key: "IoQueueWorkItemEx",
                            value: RegistryKeyValue::Dword(32),
                        },
                        RegistryModification {
                            path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\kernel",
                            key: "IoQueueThreadIrp",
                            value: RegistryKeyValue::Dword(32),
                        },
                        RegistryModification {
                            path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\kernel",
                            key: "ExTryQueueWorkItem",
                            value: RegistryKeyValue::Dword(32),
                        },
                        RegistryModification {
                            path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\kernel",
                            key: "ExQueueWorkItem",
                            value: RegistryKeyValue::Dword(32),
                        },
                        RegistryModification {
                            path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\kernel",
                            key: "IoEnqueueIrp",
                            value: RegistryKeyValue::Dword(32),
                        },
                        RegistryModification {
                            path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\kernel",
                            key: "XMMIZeroingEnable",
                            value: RegistryKeyValue::Dword(0),
                        },
                        RegistryModification {
                            path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\kernel",
                            key: "UseNormalStack",
                            value: RegistryKeyValue::Dword(1),
                        },
                        RegistryModification {
                            path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\kernel",
                            key: "UseNewEaBuffering",
                            value: RegistryKeyValue::Dword(1),
                        },
                        RegistryModification {
                            path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\kernel",
                            key: "StackSubSystemStackSize",
                            value: RegistryKeyValue::Dword(65536),
                        },
                    ],),
                    (
                        TweakOption::Enabled(true),
                        vec![
                            RegistryModification {
                                path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\kernel",
                                key: "MaxDynamicTickDuration",
                                value: RegistryKeyValue::Dword(0),
                            },
                            RegistryModification {
                                path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\kernel",
                                key: "MaximumSharedReadyQueueSize",
                                value: RegistryKeyValue::Dword(0),
                            },
                            RegistryModification {
                                path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\kernel",
                                key: "BufferSize",
                                value: RegistryKeyValue::Dword(0),
                            },
                            RegistryModification {
                                path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\kernel",
                                key: "IoQueueWorkItem",
                                value: RegistryKeyValue::Dword(0),
                            },
                            RegistryModification {
                                path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\kernel",
                                key: "IoQueueWorkItemToNode",
                                value: RegistryKeyValue::Dword(0),
                            },
                            RegistryModification {
                                path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\kernel",
                                key: "IoQueueWorkItemEx",
                                value: RegistryKeyValue::Dword(0),
                            },
                            RegistryModification {
                                path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\kernel",
                                key: "IoQueueThreadIrp",
                                value: RegistryKeyValue::Dword(0),
                            },
                            RegistryModification {
                                path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\kernel",
                                key: "ExTryQueueWorkItem",
                                value: RegistryKeyValue::Dword(0),
                            },
                            RegistryModification {
                                path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\kernel",
                                key: "ExQueueWorkItem",
                                value: RegistryKeyValue::Dword(0),
                            },
                            RegistryModification {
                                path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\kernel",
                                key: "IoEnqueueIrp",
                                value: RegistryKeyValue::Dword(0),
                            },
                            RegistryModification {
                                path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\kernel",
                                key: "XMMIZeroingEnable",
                                value: RegistryKeyValue::Dword(1),
                            },
                            RegistryModification {
                                path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\kernel",
                                key: "UseNormalStack",
                                value: RegistryKeyValue::Dword(0),
                            },
                            RegistryModification {
                                path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\kernel",
                                key: "UseNewEaBuffering",
                                value: RegistryKeyValue::Dword(0),
                            },
                            RegistryModification {
                                path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\kernel",
                                key: "StackSubSystemStackSize",
                                value: RegistryKeyValue::Dword(0),
                            },
                        ],
                    ),
                ],
            ),
        },
        true, // requires reboot
    )
}
