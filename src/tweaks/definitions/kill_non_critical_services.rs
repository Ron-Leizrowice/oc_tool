// src/tweaks/definitions/kill_non_critical_services.rs

use anyhow::Error;
use tracing::{error, info};
use widestring::U16CString;
use windows::{
    core::PCWSTR,
    Win32::System::Services::{
        CloseServiceHandle, ControlService, OpenSCManagerW, OpenServiceW, QueryServiceStatusEx,
        SC_MANAGER_ALL_ACCESS, SC_STATUS_PROCESS_INFO, SERVICE_CONTROL_STOP, SERVICE_QUERY_STATUS,
        SERVICE_STATUS, SERVICE_STATUS_PROCESS, SERVICE_STOP, SERVICE_STOPPED,
    },
};

use crate::tweaks::{TweakId, TweakMethod};

const SERVICES_TO_KILL: &[&str; 106] = &[
    "AdobeARMservice",            // Adobe Acrobat Update Service
    "AdobeFlashPlayerUpdateSvc",  // Adobe Flash Player Update Service
    "AdobeUpdateService",         // Adobe Update Service
    "AeLookupSvc",                // Application Experience
    "AJRouter",                   // AllJoyn Router Service
    "ALG",                        // Application Layer Gateway Service
    "AppIDSvc",                   // Application Identity
    "AppMgmt",                    // Application Management
    "AppReadiness",               // App Readiness
    "AppXSvc",                    // AppX Deployment Service
    "AssignedAccessManagerSvc",   // Assigned Access Manager Service
    "AudioEndpointBuilder",       // Windows Audio Endpoint Builder
    "Audiosrv",                   // Windows Audio
    "autotimesvc",                // Cellular Time
    "AxInstSV",                   // ActiveX Installer
    "BDESVC",                     // BitLocker Drive Encryption Service
    "BluetoothUserService",       // Bluetooth User Support Service
    "BFE",                        // Base Filtering Engine
    "BITS",                       // Background Intelligent Transfer Service
    "Browser",                    // Computer Browser
    "BthAvctpSvc",                // AVCTP service
    "BTAGService",                // Bluetooth Audio Gateway Service
    "camsvc",                     // Capability Access Manager Service
    "CaptureService",             // CaptureService
    "CDPSvc",                     // Connected Devices Platform Service
    "CertPropSvc",                // Certificate Propagation
    "ClipSVC",                    // Client License Service
    "CryptSvc",                   // Cryptographic Services
    "DiagTrack",                  // Connected User Experiences and Telemetry
    "defragsvc",                  // Optimize drives
    "DevQueryBroker",             // Device Query Broker
    "DeviceAssociationService",   // Device Association Service
    "DevicesFlowUserSvc", // Allows ConnectUX and PC Settings to Connect and Pair with WiFi displays and Bluetooth devices.
    "diagnosticshub",     // Microsoft (R) Diagnostics Hub Standard Collector Service
    "DispBrokerDesktopSvc", // Display Policy Service
    "Dhcp",               // DHCP Client
    "Dnscache",           // DNS Client
    "DPS",                // Diagnostic Policy Service
    "DsmSvc",             //
    "DusmSvc",            // Data Usage
    "EFS",                // Encrypting File System
    "EntAppSvc",          // Enterprise App Management Service
    "EventLog",           // Windows Event Log
    "FrameServer",        // Windows Camera Frame Server
    "GraphicsPerfSvc",    // GraphicsPerfSvc
    "hidserv",            // Human Interface Device Service
    "HvHost",             // Hyper-V Host Compute Service
    "icssvc",             // Windows Mobile Hotspot Service
    "IKEEXT",             // IKE and AuthIP IPsec Keying Modules
    "iphlpsvc",           // IP Helper
    "lfsvc",              // Geolocation Service
    "lmhosts",            // TCP/IP NetBIOS Helper
    "InstallService",     // Microsoft Store Install Service
    "irmon",              // Infrared monitor service
    "KeyIso",             // CNG Key Isolation
    "LanmanWorkstation",  // Workstation
    "LanmanServer",       // Server
    "LicenseManager",     // Windows License Manager Service
    "LxpSvc",             // Language Experience Service
    "LSM",                // Local Session Manager
    "MDCoreSvc",          // Microsoft Defender Core Service
    "mpssvc",             // Windows Defender Firewall
    "MSDTC",              // Distributed Transaction Coordinator
    "MSiSCSI",            // Microsoft iSCSI Initiator Service
    "NaturalAuthentication", // Natural Authentication
    "NcbService",         // Network Connection Broker
    "netprofm",           // Network List Service
    "NgcCtnrSvc",         // Microsoft Passport Container
    "NgcSvc",             // Microsoft Passport
    "NPSMSvc",            // Now Playing Media Service
    "nsi",                // Network Store Interface Service
    "NVDisplay",          // NVIDIA Display Driver Service
    "OneSyncSvc",         // Synchronizes mail, contacts, calendar etc.
    "PcaSvc",             // Program Compatibility Assistant Service
    "PhoneSvc",           // Phone Service
    "PimIndexMaintenanceSvc", // Contact Data
    "pla",                // Performance Logs & Alerts
    "PlugPlay",           // Plug and Play
    "PolicyAgent",        // IPsec Policy Agent
    "PrintNotify",        // Printer Extensions and Notifications
    "RasMan",             // Remote Access Connection Manager
    "RmSvc",              // Radio Management Service
    "RtkAudioUniversalService", // Realtek Audio Universal Service
    "SCardSvr",           // Smart Card
    "ScDeviceEnum",       // Smart Card Device Enumeration Service
    "SCPolicySvc",        // Smart Card Removal Policy
    "Schedule",           // Task Scheduler
    "seclogon",           // Secondary Logon
    "SEMgrSvc",           // Payments and NFC/SE Manager
    "SensorDataService",  // Sensor Data Service
    "SensorService",      // Sensor Service
    "SensrSvc",           // Sensor Monitoring Service
    "SessionEnv",         // Remote Desktop Configuration
    "ShellHWDetection",   // Shell Hardware Detection
    "shpamsvc",           // Shared PC Account Manager
    "SSDPSRV",            // SSDP Discovery
    "SysMain",            // Superfetch
    "TextInputManagementService", // Text Input Management Service
    "Themes",             // Themes
    "TokenBroker",        // Web Account Manager
    "tzautoupdate",       // Auto Time Zone Updater
    "WpnUserService",     // Windows Push Notifications User Service
    "OneSyncSvc",         // Syncs mail, contacts, calendar etc.
    "wlidsvc",            // Microsoft Account Sign-in Assistant
    "WinHttpAutoProxySvc", // WinHTTP Web Proxy Auto-Discovery Service
    "Wcmsvc",             // Windows Connection Manager
];

pub struct KillNonCriticalServicesTweak {
    pub id: TweakId,
}

impl TweakMethod for KillNonCriticalServicesTweak {
    fn initial_state(&self) -> Result<bool, Error> {
        // Since this is an action, it doesn't have a state
        Ok(false)
    }

    fn apply(&self) -> Result<(), Error> {
        info!("{:?} -> Killing non-critical services.", self.id);
        let mut unkilled_services: Vec<String> = vec![];

        // Open a handle to the Service Control Manager
        unsafe {
            let scm_handle = OpenSCManagerW(
                None,                  // Local machine
                None,                  // ServicesActive database
                SC_MANAGER_ALL_ACCESS, // Full access to the service control manager
            )
            .map_err(|_| Error::msg("Failed to open Service Control Manager"))?;

            if scm_handle.is_invalid() {
                return Err(Error::msg("Failed to open Service Control Manager"));
            }

            // Try stopping services up to 10 times
            for i in 0..10 {
                tracing::info!("Attempting to stop services - Attempt: {}", i + 1);

                let mut failed_services = vec![];

                for service_name in SERVICES_TO_KILL {
                    // Convert the service name to a wide string format required by Windows API
                    let service_name_w = U16CString::from_str(service_name)
                        .map_err(|_| Error::msg("Failed to convert service name to wide string"))?;

                    // Open the service with permissions to stop and query status
                    let service_handle = match OpenServiceW(
                        scm_handle,
                        PCWSTR(service_name_w.as_ptr()),
                        SERVICE_STOP | SERVICE_QUERY_STATUS,
                    ) {
                        Ok(handle) => handle,
                        Err(_) => {
                            error!("Failed to open service: {}", service_name);
                            continue;
                        }
                    };

                    if service_handle.is_invalid() {
                        error!("Failed to open service: {}", service_name);
                        continue;
                    }

                    // Create a buffer for SERVICE_STATUS_PROCESS to hold the status information
                    let mut buffer = vec![0u8; std::mem::size_of::<SERVICE_STATUS_PROCESS>()];
                    let mut bytes_needed = 0;

                    // Query the current status of the service
                    let result = QueryServiceStatusEx(
                        service_handle,
                        SC_STATUS_PROCESS_INFO,
                        Some(&mut buffer),
                        &mut bytes_needed,
                    );

                    if result.is_err() {
                        error!("Failed to query service status: {}", service_name);
                        if let Err(e) = CloseServiceHandle(service_handle) {
                            error!("Failed to close service handle: {}", e);
                        }
                        continue;
                    }

                    // Interpret the buffer as SERVICE_STATUS_PROCESS structure
                    let status =
                        std::ptr::read_unaligned(buffer.as_ptr() as *const SERVICE_STATUS_PROCESS);

                    if status.dwCurrentState == SERVICE_STOPPED {
                        // If the service is already stopped, log and move on
                        info!("Service already stopped: {}", service_name);
                        if let Err(e) = CloseServiceHandle(service_handle) {
                            error!("Failed to close service handle: {}", e);
                        }
                        continue;
                    }

                    // Send the stop control to the service
                    let mut svc_status: SERVICE_STATUS = std::mem::zeroed();
                    let result =
                        ControlService(service_handle, SERVICE_CONTROL_STOP, &mut svc_status);

                    if result.is_err() {
                        error!("Failed to stop service: {}", service_name);
                        failed_services.push(service_name);
                    } else {
                        info!("Service stopped: {}", service_name);
                    }

                    // Close the handle to the service
                    if let Err(e) = CloseServiceHandle(service_handle) {
                        error!("Failed to close service handle: {}", e);
                    }
                }

                // If all services were stopped successfully, break out of the loop
                if failed_services.is_empty() {
                    break;
                }

                if i == 2 {
                    unkilled_services = failed_services.iter().map(|&s| s.to_string()).collect();
                }

                std::thread::sleep(std::time::Duration::from_secs_f32(0.1));
            }

            // Close the handle to the Service Control Manager
            if let Err(e) = CloseServiceHandle(scm_handle) {
                error!("Failed to close Service Control Manager handle: {}", e);
            }
        }

        if !unkilled_services.is_empty() {
            error!(
                "{:?} -> Failed to stop the following non-critical services: {:?}",
                self.id, unkilled_services
            );
        }

        Ok(())
    }

    fn revert(&self) -> Result<(), Error> {
        // This action cannot be reverted
        Ok(())
    }
}
