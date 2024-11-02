// src/tweaks/definitions/kill_non_critical_services.rs

use std::{
    sync::{mpsc, Arc},
    thread,
    time::Duration,
};

use anyhow::Error;
use tracing::{error, info};
use widestring::U16CString;
use windows::{
    core::PCWSTR,
    Win32::System::Services::{
        CloseServiceHandle, ControlService, OpenSCManagerW, OpenServiceW, QueryServiceStatusEx,
        SC_HANDLE, SC_MANAGER_ALL_ACCESS, SC_STATUS_PROCESS_INFO, SERVICE_CONTROL_STOP,
        SERVICE_QUERY_STATUS, SERVICE_STATUS, SERVICE_STATUS_PROCESS, SERVICE_STOP,
        SERVICE_STOPPED,
    },
};

use crate::tweaks::{TweakId, TweakMethod, TweakOption};

const SERVICES_TO_KILL: &[&str; 102] = &[
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
    "NgcCtnrSvc",         // Microsoft Passport Container
    "NgcSvc",             // Microsoft Passport
    "NPSMSvc",            // Now Playing Media Service
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

// Define a shared handle wrapper
#[derive(Debug, Clone)]
struct SharedHandle(SC_HANDLE);

// Implement Send and Sync for SharedHandle
unsafe impl Send for SharedHandle {}
unsafe impl Sync for SharedHandle {}

// Implement Drop to ensure the handle is closed when the last SharedHandle is dropped
impl Drop for SharedHandle {
    fn drop(&mut self) {
        unsafe {
            if let Err(e) = CloseServiceHandle(self.0) {
                error!("Failed to close service handle: {:?}", e);
            }
        }
    }
}

#[derive(Debug, Clone)]
struct SendSCHandle(Arc<SharedHandle>);

// Implement Send and Sync for SendSCHandle
unsafe impl Send for SendSCHandle {}
unsafe impl Sync for SendSCHandle {}

pub struct KillNonCriticalServicesTweak {
    pub id: TweakId,
}

impl KillNonCriticalServicesTweak {
    pub fn new() -> Self {
        Self {
            id: TweakId::KillAllNonCriticalServices,
        }
    }
    /// Helper function to execute a closure with a timeout.
    fn execute_with_timeout<F, T>(f: F, timeout: Duration) -> Result<T, &'static str>
    where
        F: FnOnce() -> Result<T, &'static str> + Send + 'static,
        T: Send + 'static,
    {
        let (sender, receiver) = mpsc::channel();
        thread::spawn(move || {
            let result = f();
            let _ = sender.send(result);
        });
        match receiver.recv_timeout(timeout) {
            Ok(result) => result,
            Err(_) => Err("Operation timed out"),
        }
    }

    /// Attempts to open the Service Control Manager with a timeout.
    fn open_scm_handle() -> Result<SendSCHandle, &'static str> {
        KillNonCriticalServicesTweak::execute_with_timeout(
            || {
                unsafe {
                    let handle = OpenSCManagerW(
                        None,                  // Local machine
                        None,                  // ServicesActive database
                        SC_MANAGER_ALL_ACCESS, // Full access to the service control manager
                    )
                    .map_err(|_| "Failed to open Service Control Manager")?;

                    if handle.is_invalid() {
                        return Err("Failed to open Service Control Manager");
                    }

                    Ok(SendSCHandle(Arc::new(SharedHandle(handle))))
                }
            },
            Duration::from_secs(1),
        )
    }

    /// Attempts to open a specific service with a timeout.
    fn open_service_handle(
        scm_handle: SendSCHandle,
        service_name: &str,
    ) -> Result<SendSCHandle, &'static str> {
        let service_name_w = U16CString::from_str(service_name)
            .map_err(|_| "Failed to convert service name to wide string")?;
        KillNonCriticalServicesTweak::execute_with_timeout(
            move || {
                unsafe {
                    let sc_handle = scm_handle.0.clone(); // Clone the Arc
                    let handle = OpenServiceW(
                        sc_handle.0,
                        PCWSTR(service_name_w.as_ptr()),
                        SERVICE_STOP | SERVICE_QUERY_STATUS,
                    )
                    .map_err(|_| "Failed to open service handle")?;

                    if handle.is_invalid() {
                        return Err("Failed to open service handle");
                    }

                    Ok(SendSCHandle(Arc::new(SharedHandle(handle))))
                }
            },
            Duration::from_secs(1),
        )
    }

    /// Attempts to query the status of a service with a timeout.
    fn query_service_status(
        service_handle: SendSCHandle,
    ) -> Result<SERVICE_STATUS_PROCESS, &'static str> {
        KillNonCriticalServicesTweak::execute_with_timeout(
            move || {
                unsafe {
                    let sc_handle = service_handle.0.clone();
                    let mut buffer = vec![0u8; std::mem::size_of::<SERVICE_STATUS_PROCESS>()];
                    let mut bytes_needed = 0;

                    let result = QueryServiceStatusEx(
                        sc_handle.0,
                        SC_STATUS_PROCESS_INFO,
                        Some(&mut buffer),
                        &mut bytes_needed,
                    );

                    if result.is_err() {
                        return Err("Failed to query service status");
                    }

                    // Safely interpret the buffer as SERVICE_STATUS_PROCESS
                    let status =
                        std::ptr::read_unaligned(buffer.as_ptr() as *const SERVICE_STATUS_PROCESS);

                    Ok(status)
                }
            },
            Duration::from_secs(1),
        )
    }

    /// Attempts to stop a service with a timeout.
    fn stop_service(service_handle: SendSCHandle) -> Result<(), &'static str> {
        KillNonCriticalServicesTweak::execute_with_timeout(
            move || unsafe {
                let sc_handle = service_handle.0.clone();
                let mut svc_status: SERVICE_STATUS = std::mem::zeroed();
                let result = ControlService(sc_handle.0, SERVICE_CONTROL_STOP, &mut svc_status);

                if result.is_err() {
                    return Err("Failed to stop service");
                }

                Ok(())
            },
            Duration::from_secs(1),
        )
    }
}

impl TweakMethod for KillNonCriticalServicesTweak {
    fn initial_state(&self) -> Result<TweakOption, Error> {
        // Since this is an action, it doesn't have a state
        Ok(TweakOption::Enabled(false))
    }

    fn apply(&self, _option: TweakOption) -> Result<(), Error> {
        info!("{:?} -> Killing non-critical services.", self.id);
        let mut failed_services: Vec<String> = vec![];

        // Attempt to open the Service Control Manager with a timeout
        let scm_handle = match KillNonCriticalServicesTweak::open_scm_handle() {
            Ok(handle) => handle,
            Err(e) => {
                error!("Failed to open Service Control Manager: {}", e);
                // If SCM handle can't be opened, all services are considered failed
                failed_services.extend(SERVICES_TO_KILL.iter().map(|&s| s.to_string()));
                // Early return since we can't proceed without SCM
                return Err(Error::msg(format!(
                    "Failed to open Service Control Manager: {}",
                    e
                )));
            }
        };

        // Try stopping services up to 5 times
        for attempt in 1..=5 {
            tracing::info!("Attempting to stop services - Attempt: {}", attempt);

            for &service_name in SERVICES_TO_KILL.iter() {
                // Open the service with a timeout
                let service_handle = match KillNonCriticalServicesTweak::open_service_handle(
                    scm_handle.clone(),
                    service_name,
                ) {
                    Ok(handle) => handle,
                    Err(e) => {
                        error!(
                            "Attempt {}: Failed to open service '{}': {}",
                            attempt, service_name, e
                        );
                        continue;
                    }
                };

                // Query the service status with a timeout
                let status = match KillNonCriticalServicesTweak::query_service_status(
                    service_handle.clone(),
                ) {
                    Ok(status) => status,
                    Err(e) => {
                        error!(
                            "Attempt {}: Failed to query status for service '{}': {}",
                            attempt, service_name, e
                        );
                        continue;
                    }
                };

                if status.dwCurrentState == SERVICE_STOPPED {
                    // Service is already stopped
                    info!("Service '{}' is already stopped.", service_name);
                    continue;
                }

                // Attempt to stop the service with a timeout
                match KillNonCriticalServicesTweak::stop_service(service_handle.clone()) {
                    Ok(_) => {
                        info!("Service '{}' stopped successfully.", service_name);
                    }
                    Err(e) => {
                        error!(
                            "Attempt {}: Failed to stop service '{}': {}",
                            attempt, service_name, e
                        );
                    }
                }
            }

            thread::sleep(Duration::from_millis(500));
        }

        if !failed_services.is_empty() {
            error!(
                "{:?} -> Failed to stop the following non-critical services: {:?}",
                self.id, failed_services
            );
            return Err(Error::msg(format!(
                "Failed to stop services: {:?}",
                failed_services
            )));
        }

        Ok(())
    }

    fn revert(&self) -> Result<(), Error> {
        // This action cannot be reverted
        Ok(())
    }
}
