// src/utils/registry.rs

use std::fmt;

use anyhow::{Context, Result};
use winreg::{
    enums::{
        RegType::{REG_BINARY, REG_DWORD, REG_SZ},
        HKEY_CLASSES_ROOT, HKEY_CURRENT_CONFIG, HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, HKEY_USERS,
        KEY_READ, KEY_WRITE,
    },
    RegKey, RegValue,
};

/// Enumeration of supported registry key value types.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum RegistryKeyValue {
    Dword(u32),
    Binary(Vec<u8>),
    String(String),
    Deleted, // the key should not exist
}

impl fmt::Display for RegistryKeyValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RegistryKeyValue::Dword(v) => write!(f, "Dword({})", v),
            RegistryKeyValue::Binary(v) => write!(f, "Binary({:?})", v),
            RegistryKeyValue::String(v) => write!(f, "String({})", v),
            RegistryKeyValue::Deleted => write!(f, "None"),
        }
    }
}

/// Reads a registry value from the specified path and key.
///
/// # Parameters
/// - `path`: The registry path (e.g., "HKEY_LOCAL_MACHINE\\Software\\...").
/// - `key_name`: The name of the registry value.
///
/// # Returns
///
/// - `Ok(Some(RegistryKeyValue))` if the value exists.
/// - `Ok(None)` if the value doesn't exist.
/// - `Err(anyhow::Error)` if an error occurs.
pub fn read_registry_value(path: &str, key_name: &str) -> Result<Option<RegistryKeyValue>> {
    let (hive, subkey_path) = parse_registry_path(path)
        .with_context(|| format!("Failed to parse registry path '{}'", path))?;

    let subkey = hive
        .open_subkey_with_flags(&subkey_path, KEY_READ)
        .with_context(|| format!("Failed to open subkey '{}'", subkey_path))?;

    get_registry_key_value(&subkey, key_name)
}

/// Creates or modifies a registry value at the specified path,
/// ensuring that all intermediate keys exist.
///
/// # Parameters
/// - `path`: The registry path (e.g., "HKEY_LOCAL_MACHINE\\Software\\...").
/// - `key_name`: The name of the registry value.
/// - `value`: The `RegistryKeyValue` to set.
///
/// # Returns
/// - `Ok(())` if successful.
/// - `Err(anyhow::Error)` if modification fails.
pub fn create_or_modify_registry_value(
    path: &str,
    key_name: &str,
    value: &RegistryKeyValue,
) -> Result<()> {
    let (hive, subkey_path) = parse_registry_path(path)
        .with_context(|| format!("Failed to parse registry path '{}'", path))?;

    // Use create_subkey which creates all intermediate subkeys if they don't exist
    let (key, _) = hive
        .create_subkey(&subkey_path)
        .with_context(|| format!("Failed to create or open subkey '{}'", subkey_path))?;

    // Now set the value
    set_registry_key_value(&key, key_name, value).with_context(|| {
        format!(
            "Failed to set registry value '{}' in path '{}'",
            key_name, path
        )
    })
}

/// Deletes a registry value at the specified path.
///
/// # Parameters
/// - `path`: The registry path (e.g., "HKEY_LOCAL_MACHINE\\Software\\...").
/// - `key_name`: The name of the registry value to delete.
///
/// # Returns
/// - `Ok(())` if successful or if the value does not exist.
/// - `Err(anyhow::Error)` if deletion fails for reasons other than the value not existing.
pub fn delete_registry_value(path: &str, key_name: &str) -> Result<()> {
    let (hive, subkey_path) = parse_registry_path(path)
        .with_context(|| format!("Failed to parse registry path '{}'", path))?;

    let subkey = hive
        .open_subkey_with_flags(&subkey_path, KEY_WRITE)
        .with_context(|| format!("Failed to open subkey '{}'", subkey_path))?;

    match subkey.delete_value(key_name) {
        Ok(_) => Ok(()),
        Err(e) => {
            if e.kind() == std::io::ErrorKind::NotFound {
                // The value does not exist; treat as success
                Ok(())
            } else {
                Err(anyhow::anyhow!(
                    "Failed to delete registry entry '{}' in '{}': {}",
                    key_name,
                    subkey_path,
                    e
                ))
            }
        }
    }
}

/// Parses the full registry path into hive and subkey path.
///
/// # Parameters
///
/// - `path`: The full registry path (e.g., "HKEY_LOCAL_MACHINE\\Software\\...").
///
/// # Returns
///
/// - `Ok((RegKey, String))` with the parsed hive and subkey path.
/// - `Err(anyhow::Error)` if parsing fails.
pub fn parse_registry_path(path: &str) -> Result<(RegKey, String)> {
    let components: Vec<&str> = path.split('\\').collect();
    if components.len() < 2 {
        anyhow::bail!(
            "Invalid registry path: '{}'. Expected format 'HKEY_*\\Subkey\\...'",
            path
        );
    }
    let hive = match components[0].to_uppercase().as_str() {
        "HKEY_LOCAL_MACHINE" => HKEY_LOCAL_MACHINE,
        "HKEY_CURRENT_USER" => HKEY_CURRENT_USER,
        "HKEY_CLASSES_ROOT" => HKEY_CLASSES_ROOT,
        "HKEY_USERS" => HKEY_USERS,
        "HKEY_CURRENT_CONFIG" => HKEY_CURRENT_CONFIG,
        other => anyhow::bail!("Unsupported registry hive: '{}'", other),
    };
    let key = components[1..].join("\\");
    Ok((RegKey::predef(hive), key))
}

/// Gets a registry value.
///
/// # Parameters
///
/// - `key`: The `RegKey` to read from.
/// - `value_name`: The name of the registry value.
///
/// # Returns
///
/// - `Ok(Some(RegistryKeyValue))` if the value exists.
/// - `Ok(None)` if the value doesn't exist.
/// - `Err(anyhow::Error)` if an error occurs.
fn get_registry_key_value(key: &RegKey, value_name: &str) -> Result<Option<RegistryKeyValue>> {
    match key.get_raw_value(value_name) {
        Ok(value) => match value.vtype {
            REG_DWORD => {
                if value.bytes.len() >= 4 {
                    let dword = u32::from_le_bytes([
                        value.bytes[0],
                        value.bytes[1],
                        value.bytes[2],
                        value.bytes[3],
                    ]);
                    Ok(Some(RegistryKeyValue::Dword(dword)))
                } else {
                    anyhow::bail!("REG_DWORD data too small for key '{}'", value_name);
                }
            }

            REG_BINARY => Ok(Some(RegistryKeyValue::Binary(value.bytes.clone()))),

            REG_SZ => Ok(Some(RegistryKeyValue::String(
                String::from_utf16_lossy(
                    &value
                        .bytes
                        .chunks_exact(2)
                        .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
                        .collect::<Vec<u16>>(),
                )
                .to_string(),
            ))),

            _ => anyhow::bail!("Unsupported registry value type: {:?}", value.vtype),
        },
        Err(e) => {
            if e.kind() == std::io::ErrorKind::NotFound {
                Ok(None)
            } else {
                Err(anyhow::anyhow!(
                    "Failed to read value '{}': {}",
                    value_name,
                    e
                ))
            }
        }
    }
}

/// Sets a registry value.
///
/// # Parameters
///
/// - `key`: The `RegKey` to modify.
/// - `value_name`: The name of the registry value.
/// - `value`: The `RegistryKeyValue` to set.
///
/// # Returns
///
/// - `Ok(())` if successful.
/// - `Err(anyhow::Error)` if setting fails.
pub fn set_registry_key_value(
    key: &RegKey,
    value_name: &str,
    value: &RegistryKeyValue,
) -> Result<()> {
    match value {
        RegistryKeyValue::Dword(v) => key
            .set_value(value_name, v)
            .with_context(|| format!("Failed to set DWORD value '{}' to '{}'", value_name, v)),
        RegistryKeyValue::Binary(data) => key
            .set_raw_value(
                value_name,
                &RegValue {
                    bytes: data.clone(),
                    vtype: winreg::enums::RegType::REG_BINARY,
                },
            )
            .with_context(|| {
                format!(
                    "Failed to set Binary value '{}' to '{:?}'",
                    value_name, data
                )
            }),
        RegistryKeyValue::String(s) => key
            .set_raw_value(
                value_name,
                &RegValue {
                    bytes: s.encode_utf16().flat_map(|c| c.to_le_bytes()).collect(),
                    vtype: REG_SZ,
                },
            )
            .with_context(|| format!("Failed to set String value '{}' to '{}'", value_name, s)),
        RegistryKeyValue::Deleted => match key.delete_value(value_name) {
            Ok(_) => Ok(()),
            Err(e) => {
                if e.kind() == std::io::ErrorKind::NotFound {
                    Ok(())
                } else {
                    Err(anyhow::anyhow!(
                        "Failed to delete value '{}': {}",
                        value_name,
                        e
                    ))
                }
            }
        },
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use lazy_static::lazy_static;

    use super::*;

    lazy_static! {
        static ref TEST_MUTEX: Mutex<()> = Mutex::new(());
    }

    const TEST_SUBKEY: &str = "Software\\WinregUtilsTest";

    fn get_test_path() -> String {
        format!("HKEY_CURRENT_USER\\{}", TEST_SUBKEY)
    }

    #[test]
    fn test_create_modify_read_delete_dword() {
        let _lock = TEST_MUTEX.lock().unwrap(); // Ensure tests run serially

        let path = get_test_path();
        let key_name = "TestDword";
        let test_value = RegistryKeyValue::Dword(42);

        // Ensure clean state
        let _ = delete_registry_value(&path, key_name);

        // Create or modify
        create_or_modify_registry_value(&path, key_name, &test_value)
            .expect("Failed to set DWORD value");

        // Read back
        let read_value = read_registry_value(&path, key_name).expect("Failed to read DWORD value");
        assert_eq!(read_value, Some(test_value.clone()));

        // Clean up
        delete_registry_value(&path, key_name).expect("Failed to delete DWORD value");

        // Verify deletion
        let read_deleted =
            read_registry_value(&path, key_name).expect("Failed to read deleted DWORD value");
        assert_eq!(read_deleted, None);
    }

    #[test]
    fn test_create_modify_read_delete_binary() {
        let _lock = TEST_MUTEX.lock().unwrap();

        let path = get_test_path();
        let key_name = "TestBinary";
        let test_value = RegistryKeyValue::Binary(vec![0xDE, 0xAD, 0xBE, 0xEF]);

        // Ensure clean state
        let _ = delete_registry_value(&path, key_name);

        // Create or modify
        create_or_modify_registry_value(&path, key_name, &test_value)
            .expect("Failed to set Binary value");

        // Read back
        let read_value = read_registry_value(&path, key_name).expect("Failed to read Binary value");
        assert_eq!(read_value, Some(test_value.clone()));

        // Clean up
        delete_registry_value(&path, key_name).expect("Failed to delete Binary value");

        // Verify deletion
        let read_deleted =
            read_registry_value(&path, key_name).expect("Failed to read deleted Binary value");
        assert_eq!(read_deleted, None);
    }

    #[test]
    fn test_nonexistent_value() {
        let _lock = TEST_MUTEX.lock().unwrap(); // Ensure tests run serially

        let path = get_test_path(); // "HKEY_CURRENT_USER\\Software\\WinregUtilsTest"
        let key_name = "NonExistentValue";

        // Ensure the key exists by creating it (if not already existing)
        create_or_modify_registry_value(&path, "TempValue", &RegistryKeyValue::Dword(1))
            .expect("Failed to set temporary value");

        // Ensure 'NonExistentValue' does not exist
        delete_registry_value(&path, key_name)
            .expect("Failed to delete 'NonExistentValue' (if it exists)");

        // Attempt to read the non-existent value, expecting Ok(None)
        let result =
            read_registry_value(&path, key_name).expect("Failed to read value from existing key");
        assert_eq!(result, None);

        // Clean up the temporary value
        delete_registry_value(&path, "TempValue").expect("Failed to delete temporary value");
    }

    #[test]
    fn test_invalid_registry_path() {
        let _lock = TEST_MUTEX.lock().unwrap();

        let invalid_paths = vec![
            "",
            "INVALID_HIVE\\Software",
            "HKEY_UNKNOWN\\Software",
            "HKEY_CURRENT_USER", // Missing subkey
        ];

        for path in invalid_paths {
            let result = parse_registry_path(path);
            assert!(result.is_err(), "Path '{}' should be invalid", path);
        }
    }
}
