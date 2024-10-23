// src/utils/registry.rs

use std::fmt;

use anyhow::{Context, Result};
use widestring::U16CString;
use winreg::{
    enums::{
        RegType::{REG_BINARY, REG_DWORD, REG_EXPAND_SZ, REG_MULTI_SZ, REG_QWORD, REG_SZ},
        HKEY_CLASSES_ROOT, HKEY_CURRENT_CONFIG, HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, HKEY_USERS,
        KEY_READ, KEY_WRITE,
    },
    RegKey, RegValue,
};

/// Enumeration of supported registry key value types.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum RegistryKeyValue {
    Dword(u32),
    Qword(u64),
    Binary(Vec<u8>),
    MultiString(Vec<String>),
    String(String),
    ExpandString(String),
}

impl fmt::Display for RegistryKeyValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RegistryKeyValue::Dword(v) => write!(f, "Dword({})", v),
            RegistryKeyValue::Qword(v) => write!(f, "Qword({})", v),
            RegistryKeyValue::Binary(v) => write!(f, "Binary({:?})", v),
            RegistryKeyValue::MultiString(v) => write!(f, "MultiString({:?})", v),
            RegistryKeyValue::String(v) => write!(f, "String({})", v),
            RegistryKeyValue::ExpandString(v) => write!(f, "ExpandString({})", v),
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
            REG_QWORD => {
                if value.bytes.len() >= 8 {
                    let qword = u64::from_le_bytes([
                        value.bytes[0],
                        value.bytes[1],
                        value.bytes[2],
                        value.bytes[3],
                        value.bytes[4],
                        value.bytes[5],
                        value.bytes[6],
                        value.bytes[7],
                    ]);
                    Ok(Some(RegistryKeyValue::Qword(qword)))
                } else {
                    anyhow::bail!("REG_QWORD data too small for key '{}'", value_name);
                }
            }
            REG_BINARY => Ok(Some(RegistryKeyValue::Binary(value.bytes.clone()))),
            REG_MULTI_SZ => {
                if value.bytes.len() % 2 != 0 {
                    anyhow::bail!(
                        "Invalid UTF-16 byte length for REG_MULTI_SZ key '{}'. Length must be even.",
                        value_name
                    );
                }

                let u16_vec: Vec<u16> = value
                    .bytes
                    .chunks_exact(2)
                    .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
                    .collect();

                let strings = u16_vecs_to_multi_string(&u16_vec)
                    .context("Failed to convert REG_MULTI_SZ to Vec<String>")?;

                Ok(Some(RegistryKeyValue::MultiString(strings)))
            }
            REG_SZ | REG_EXPAND_SZ => {
                if value.bytes.len() % 2 != 0 {
                    anyhow::bail!(
                        "Invalid UTF-16 byte length for key '{}'. Length must be even.",
                        value_name
                    );
                }

                let u16_vec: Vec<u16> = value
                    .bytes
                    .chunks_exact(2)
                    .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
                    .collect();

                let string = u16_vec_to_string(u16_vec);

                if value.vtype == REG_SZ {
                    Ok(Some(RegistryKeyValue::String(string)))
                } else {
                    Ok(Some(RegistryKeyValue::ExpandString(string)))
                }
            }
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
        RegistryKeyValue::Qword(v) => key
            .set_value(value_name, v)
            .with_context(|| format!("Failed to set QWORD value '{}' to '{}'", value_name, v)),
        RegistryKeyValue::Binary(data) => {
            // Set a Binary value
            key.set_raw_value(
                value_name,
                &RegValue {
                    bytes: data.clone(),
                    vtype: REG_BINARY,
                },
            )
            .with_context(|| {
                format!(
                    "Failed to set Binary value '{}' to '{:?}'",
                    value_name, data
                )
            })
        }
        RegistryKeyValue::MultiString(strings) => {
            // Convert Vec<String> to Vec<u16> using helper function
            let bytes_u16 = multi_string_to_u16_vecs(strings)
                .context("Failed to convert Vec<String> to u16 vector for REG_MULTI_SZ")?;

            // Convert u16_vec to bytes (little-endian)
            let bytes: Vec<u8> = bytes_u16.iter().flat_map(|&u| u.to_le_bytes()).collect();

            key.set_raw_value(
                value_name,
                &RegValue {
                    bytes,
                    vtype: REG_MULTI_SZ,
                },
            )
            .with_context(|| {
                format!(
                    "Failed to set MultiString value '{}' to '{:?}'",
                    value_name, strings
                )
            })
        }
        RegistryKeyValue::String(s) => {
            // Convert String to u16 vector using helper function
            let u16_vec = string_to_u16_vec(s)
                .context("Failed to convert String to u16 vector for REG_SZ")?;

            // Convert u16_vec to bytes (little-endian)
            let bytes: Vec<u8> = u16_vec.iter().flat_map(|&u| u.to_le_bytes()).collect();

            key.set_raw_value(
                value_name,
                &RegValue {
                    bytes,
                    vtype: REG_SZ,
                },
            )
            .with_context(|| format!("Failed to set String value '{}' to '{}'", value_name, s))
        }
        RegistryKeyValue::ExpandString(s) => {
            // Convert String to u16 vector using helper function
            let u16_vec = string_to_u16_vec(s)
                .context("Failed to convert String to u16 vector for REG_EXPAND_SZ")?;

            // Convert u16_vec to bytes (little-endian)
            let bytes: Vec<u8> = u16_vec.iter().flat_map(|&u| u.to_le_bytes()).collect();

            key.set_raw_value(
                value_name,
                &RegValue {
                    bytes,
                    vtype: REG_EXPAND_SZ,
                },
            )
            .with_context(|| {
                format!(
                    "Failed to set ExpandString value '{}' to '{}'",
                    value_name, s
                )
            })
        }
    }
}

/// Converts a `Vec<u16>` to a `String`.
///
/// # Parameters
///
/// - `u16_vec`: A vector of `u16` values representing a UTF-16LE encoded string.
///
/// # Returns
///
/// - `Ok(String)` if the conversion is successful.
/// - `Err(anyhow::Error)` if the conversion fails.
fn u16_vec_to_string(u16_vec: Vec<u16>) -> String {
    let u16_string = U16CString::from_vec_truncate(u16_vec);
    u16_string.to_string_lossy()
}

/// Converts a `String` to a `Vec<u16>` with a null terminator.
///
/// # Parameters
///
/// - `s`: The string to convert.
///
/// # Returns
///
/// - `Ok(Vec<u16>)` if the conversion is successful.
/// - `Err(anyhow::Error)` if the conversion fails.
fn string_to_u16_vec(s: &str) -> Result<Vec<u16>> {
    let u16_string = U16CString::from_str(s).context("Failed to encode string as UTF-16")?;
    Ok(u16_string.into_vec())
}

/// Converts a flat `Vec<u16>` to a `Vec<String>` for REG_MULTI_SZ.
///
/// # Parameters
///
/// - `u16_vec`: A flat vector of `u16` values representing multiple UTF-16LE encoded strings,
///              separated by nulls and terminated by a double null.
///
/// # Returns
///
/// - `Ok(Vec<String>)` if the conversion is successful.
/// - `Err(anyhow::Error)` if the conversion fails.
fn u16_vecs_to_multi_string(u16_vec: &[u16]) -> Result<Vec<String>> {
    let mut strings = Vec::new();
    let mut current = Vec::new();

    for &code in u16_vec {
        if code == 0 {
            if !current.is_empty() {
                let s = u16_vec_to_string(current.clone());
                strings.push(s);
                current.clear();
            }
        } else {
            current.push(code);
        }
    }

    Ok(strings)
}

/// Converts a `Vec<String>` to a flat `Vec<u16>` for REG_MULTI_SZ.
///
/// # Parameters
///
/// - `strings`: A vector of strings to convert.
///
/// # Returns
///
/// - `Ok(Vec<u16>)` if the conversion is successful.
/// - `Err(anyhow::Error)` if the conversion fails.
fn multi_string_to_u16_vecs(strings: &[String]) -> Result<Vec<u16>> {
    let mut u16_vec = Vec::new();

    for s in strings {
        let mut s_u16 = string_to_u16_vec(s)
            .context("Failed to convert string to u16 vector in REG_MULTI_SZ")?;
        // Remove the null terminator added by `string_to_u16_vec`
        if let Some(&last) = s_u16.last() {
            if last == 0 {
                s_u16.pop();
            }
        }
        u16_vec.extend_from_slice(&s_u16);
        // Add a single null to separate strings
        u16_vec.push(0);
    }

    // Add an additional null to terminate the sequence
    u16_vec.push(0);

    Ok(u16_vec)
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
    fn test_create_modify_read_delete_qword() {
        let _lock = TEST_MUTEX.lock().unwrap();

        let path = get_test_path();
        let key_name = "TestQword";
        let test_value = RegistryKeyValue::Qword(1234567890123456789);

        // Ensure clean state
        let _ = delete_registry_value(&path, key_name);

        // Create or modify
        create_or_modify_registry_value(&path, key_name, &test_value)
            .expect("Failed to set QWORD value");

        // Read back
        let read_value = read_registry_value(&path, key_name).expect("Failed to read QWORD value");
        assert_eq!(read_value, Some(test_value.clone()));

        // Clean up
        delete_registry_value(&path, key_name).expect("Failed to delete QWORD value");

        // Verify deletion
        let read_deleted =
            read_registry_value(&path, key_name).expect("Failed to read deleted QWORD value");
        assert_eq!(read_deleted, None);
    }

    #[test]
    fn test_create_modify_read_delete_string() {
        let _lock = TEST_MUTEX.lock().unwrap();

        let path = get_test_path();
        let key_name = "TestString";
        let test_value = RegistryKeyValue::String("Hello, Registry!".to_string());

        // Ensure clean state
        let _ = delete_registry_value(&path, key_name);

        // Create or modify
        create_or_modify_registry_value(&path, key_name, &test_value)
            .expect("Failed to set String value");

        // Read back
        let read_value = read_registry_value(&path, key_name).expect("Failed to read String value");
        assert_eq!(read_value, Some(test_value.clone()));

        // Clean up
        delete_registry_value(&path, key_name).expect("Failed to delete String value");

        // Verify deletion
        let read_deleted =
            read_registry_value(&path, key_name).expect("Failed to read deleted String value");
        assert_eq!(read_deleted, None);
    }

    #[test]
    fn test_create_modify_read_delete_expand_string() {
        let _lock = TEST_MUTEX.lock().unwrap();

        let path = get_test_path();
        let key_name = "TestExpandString";
        let test_value = RegistryKeyValue::ExpandString("Path\\to\\%USERNAME%".to_string());

        // Ensure clean state
        let _ = delete_registry_value(&path, key_name);

        // Create or modify
        create_or_modify_registry_value(&path, key_name, &test_value)
            .expect("Failed to set ExpandString value");

        // Read back
        let read_value =
            read_registry_value(&path, key_name).expect("Failed to read ExpandString value");
        assert_eq!(read_value, Some(test_value.clone()));

        // Clean up
        delete_registry_value(&path, key_name).expect("Failed to delete ExpandString value");

        // Verify deletion
        let read_deleted = read_registry_value(&path, key_name)
            .expect("Failed to read deleted ExpandString value");
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
    fn test_create_modify_read_delete_multi_string() {
        let _lock = TEST_MUTEX.lock().unwrap();

        let path = get_test_path();
        let key_name = "TestMultiString";
        let test_value = RegistryKeyValue::MultiString(vec![
            "First".to_string(),
            "Second".to_string(),
            "Third".to_string(),
        ]);

        // Ensure clean state
        let _ = delete_registry_value(&path, key_name);

        // Create or modify
        create_or_modify_registry_value(&path, key_name, &test_value)
            .expect("Failed to set MultiString value");

        // Read back
        let read_value =
            read_registry_value(&path, key_name).expect("Failed to read MultiString value");
        assert_eq!(read_value, Some(test_value.clone()));

        // Clean up
        delete_registry_value(&path, key_name).expect("Failed to delete MultiString value");

        // Verify deletion
        let read_deleted =
            read_registry_value(&path, key_name).expect("Failed to read deleted MultiString value");
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

    #[test]
    fn test_set_and_get_multiple_values() {
        let _lock = TEST_MUTEX.lock().unwrap();

        let path = get_test_path();
        let key_name1 = "TestValue1";
        let key_name2 = "TestValue2";
        let value1 = RegistryKeyValue::Dword(100);
        let value2 = RegistryKeyValue::String("Multiple Values".to_string());

        // Ensure clean state
        let _ = delete_registry_value(&path, key_name1);
        let _ = delete_registry_value(&path, key_name2);

        // Set multiple values
        create_or_modify_registry_value(&path, key_name1, &value1)
            .expect("Failed to set TestValue1");
        create_or_modify_registry_value(&path, key_name2, &value2)
            .expect("Failed to set TestValue2");

        // Read back
        let read_value1 = read_registry_value(&path, key_name1).expect("Failed to read TestValue1");
        let read_value2 = read_registry_value(&path, key_name2).expect("Failed to read TestValue2");

        assert_eq!(read_value1, Some(value1.clone()));
        assert_eq!(read_value2, Some(value2.clone()));

        // Clean up
        delete_registry_value(&path, key_name1).expect("Failed to delete TestValue1");
        delete_registry_value(&path, key_name2).expect("Failed to delete TestValue2");
    }

    #[test]
    fn test_overwrite_value() {
        let _lock = TEST_MUTEX.lock().unwrap();

        let path = get_test_path();
        let key_name = "TestOverwrite";
        let initial_value = RegistryKeyValue::String("Initial".to_string());
        let new_value = RegistryKeyValue::String("Overwritten".to_string());

        // Ensure clean state
        let _ = delete_registry_value(&path, key_name);

        // Set initial value
        create_or_modify_registry_value(&path, key_name, &initial_value)
            .expect("Failed to set initial value");

        // Overwrite with new value
        create_or_modify_registry_value(&path, key_name, &new_value)
            .expect("Failed to overwrite value");

        // Read back
        let read_value =
            read_registry_value(&path, key_name).expect("Failed to read overwritten value");
        assert_eq!(read_value, Some(new_value.clone()));

        // Clean up
        delete_registry_value(&path, key_name).expect("Failed to delete overwritten value");
    }
}
