// src/errors.rs

use thiserror::Error;

#[derive(Error, Debug)]
pub enum RegistryTweakError {
    #[error("Invalid registry key format: {0}")]
    InvalidKeyFormat(String),

    #[error("Unsupported registry hive: {0}")]
    UnsupportedHive(String),

    #[error("Failed to open registry key: {0}")]
    KeyOpenError(String),

    #[error("Failed to read registry value: {0}")]
    ReadValueError(String),

    #[error("Failed to set registry value: {0}")]
    SetValueError(String),

    #[error("Failed to create registry key: {0}")]
    CreateError(String),
}

#[derive(Error, Debug)]
pub enum GroupPolicyTweakError {
    #[error("Failed to open group policy key: {0}")]
    KeyOpenError(String),

    #[error("Failed to set group policy value: {0}")]
    SetValueError(String),

    #[error("Failed to delete group policy value: {0}")]
    DeleteValueError(String),

    #[error("Failed to create group policy key: {0}")]
    CreateError(String),

    #[error("Failed to read group policy value: {0}")]
    ReadValueError(String),
}
