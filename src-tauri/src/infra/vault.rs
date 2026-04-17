use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::RwLock;

use ring::aead::{Aad, LessSafeKey, Nonce, UnboundKey, AES_256_GCM};
use ring::rand::{SecureRandom, SystemRandom};

const KEY_FILE: &str = "vault.key";
const ENC_FILE: &str = "vault.enc";
const KEY_LEN: usize = 32;
const NONCE_LEN: usize = 12;
const TAG_LEN: usize = 16;

#[derive(Debug, thiserror::Error)]
pub enum VaultError {
    #[error("vault key file is missing or malformed while vault.enc exists")]
    KeyMissing,
    #[error("vault decrypt failed (tag mismatch or tampered ciphertext)")]
    DecryptFailed,
    #[error("vault file is corrupted: {0}")]
    Corrupted(&'static str),
    #[error("vault write failed: {0}")]
    WriteFailed(#[from] std::io::Error),
    #[error("random source failed")]
    RandomFailed,
    #[error("vault cryptographic operation failed: {0}")]
    CryptoFailed(&'static str),
    #[error("serde failed: {0}")]
    Serde(#[from] serde_json::Error),
}

pub trait VaultKeyProvider: Send + Sync {
    fn load(&self, app_data_dir: &Path) -> Result<Option<[u8; KEY_LEN]>, VaultError>;
    fn create(&self, app_data_dir: &Path) -> Result<[u8; KEY_LEN], VaultError>;
}

pub struct MachineLocalKeyProvider;

impl VaultKeyProvider for MachineLocalKeyProvider {
    fn load(&self, app_data_dir: &Path) -> Result<Option<[u8; KEY_LEN]>, VaultError> {
        let path = app_data_dir.join(KEY_FILE);
        if !path.exists() {
            return Ok(None);
        }

        let raw = fs::read(path)?;
        if raw.len() != KEY_LEN {
            return Err(VaultError::KeyMissing);
        }

        let key: [u8; KEY_LEN] = raw.try_into().map_err(|_| VaultError::KeyMissing)?;
        Ok(Some(key))
    }

    fn create(&self, app_data_dir: &Path) -> Result<[u8; KEY_LEN], VaultError> {
        let mut key = [0u8; KEY_LEN];
        SystemRandom::new()
            .fill(&mut key)
            .map_err(|_| VaultError::RandomFailed)?;
        write_atomic(&app_data_dir.join(KEY_FILE), &key)?;
        Ok(key)
    }
}

pub struct Vault {
    key: LessSafeKey,
    enc_path: PathBuf,
    entries: RwLock<HashMap<String, String>>,
}

impl Vault {
    pub fn open<P: VaultKeyProvider>(
        app_data_dir: &Path,
        provider: &P,
    ) -> Result<Self, VaultError> {
        let enc_path = app_data_dir.join(ENC_FILE);
        let enc_exists = enc_path.exists();

        let raw_key = match (provider.load(app_data_dir)?, enc_exists) {
            (Some(raw), _) => raw,
            (None, true) => return Err(VaultError::KeyMissing),
            (None, false) => provider.create(app_data_dir)?,
        };

        let unbound = UnboundKey::new(&AES_256_GCM, &raw_key)
            .map_err(|_| VaultError::CryptoFailed("invalid key material"))?;
        let key = LessSafeKey::new(unbound);

        let entries = if enc_exists {
            Self::decrypt_file(&key, &enc_path)?
        } else {
            HashMap::new()
        };

        Ok(Self {
            key,
            enc_path,
            entries: RwLock::new(entries),
        })
    }

    pub fn get(&self, key: &str) -> Option<String> {
        self.entries.read().ok().and_then(|v| v.get(key).cloned())
    }

    pub fn set(&self, key: &str, plaintext: &str) -> Result<(), VaultError> {
        let mut entries = self
            .entries
            .write()
            .map_err(|_| VaultError::Corrupted("poisoned lock"))?;

        let prev = entries.insert(key.to_string(), plaintext.to_string());
        if let Err(error) = Self::encrypt_and_persist(&self.key, &self.enc_path, &entries) {
            if let Some(old) = prev {
                entries.insert(key.to_string(), old);
            } else {
                entries.remove(key);
            }
            return Err(error);
        }

        Ok(())
    }

    fn encrypt_and_persist(
        key: &LessSafeKey,
        path: &Path,
        entries: &HashMap<String, String>,
    ) -> Result<(), VaultError> {
        let mut nonce_bytes = [0u8; NONCE_LEN];
        SystemRandom::new()
            .fill(&mut nonce_bytes)
            .map_err(|_| VaultError::RandomFailed)?;

        let mut payload = serde_json::to_vec(entries)?;
        key.seal_in_place_append_tag(
            Nonce::assume_unique_for_key(nonce_bytes),
            Aad::empty(),
            &mut payload,
        )
        .map_err(|_| VaultError::CryptoFailed("encryption failed"))?;

        let mut out = Vec::with_capacity(NONCE_LEN + payload.len());
        out.extend_from_slice(&nonce_bytes);
        out.extend_from_slice(&payload);
        write_atomic(path, &out)
    }

    fn decrypt_file(key: &LessSafeKey, path: &Path) -> Result<HashMap<String, String>, VaultError> {
        let bytes = fs::read(path)?;
        if bytes.len() < NONCE_LEN + TAG_LEN {
            return Err(VaultError::Corrupted("truncated"));
        }

        let (nonce_raw, ciphertext_raw) = bytes.split_at(NONCE_LEN);
        let nonce_arr: [u8; NONCE_LEN] = nonce_raw
            .try_into()
            .map_err(|_| VaultError::Corrupted("bad nonce"))?;
        let nonce = Nonce::assume_unique_for_key(nonce_arr);

        let mut ciphertext = ciphertext_raw.to_vec();
        let plaintext = key
            .open_in_place(nonce, Aad::empty(), &mut ciphertext)
            .map_err(|_| VaultError::DecryptFailed)?;

        serde_json::from_slice(plaintext).map_err(VaultError::from)
    }
}

fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), VaultError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let tmp = path.with_extension("tmp");
    {
        let mut file = File::create(&tmp)?;
        file.write_all(bytes)?;
        file.sync_all()?;
    }
    fs::rename(tmp, path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::sync::Arc;

    use uuid::Uuid;

    use super::{MachineLocalKeyProvider, Vault, VaultError, ENC_FILE, KEY_FILE};

    struct TestDir {
        path: PathBuf,
    }

    impl TestDir {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!("mirage-vault-test-{}", Uuid::new_v4()));
            std::fs::create_dir_all(&path).unwrap();
            Self { path }
        }
    }

    impl Drop for TestDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.path);
        }
    }

    #[test]
    fn roundtrip_persists_across_reopen() {
        let dir = TestDir::new();

        let vault = Vault::open(&dir.path, &MachineLocalKeyProvider).unwrap();
        vault.set("llm_api_key", "sk-123").unwrap();
        assert_eq!(vault.get("llm_api_key").as_deref(), Some("sk-123"));

        let reopened = Vault::open(&dir.path, &MachineLocalKeyProvider).unwrap();
        assert_eq!(reopened.get("llm_api_key").as_deref(), Some("sk-123"));
    }

    #[test]
    fn tampered_ciphertext_is_rejected() {
        let dir = TestDir::new();

        let vault = Vault::open(&dir.path, &MachineLocalKeyProvider).unwrap();
        vault.set("llm_api_key", "sk-123").unwrap();

        let enc_path = dir.path.join(ENC_FILE);
        let mut bytes = std::fs::read(&enc_path).unwrap();
        bytes[15] ^= 0xAA;
        std::fs::write(&enc_path, &bytes).unwrap();

        let reopened = Vault::open(&dir.path, &MachineLocalKeyProvider);
        assert!(matches!(reopened, Err(VaultError::DecryptFailed)));
    }

    #[test]
    fn key_missing_is_hard_failure_when_ciphertext_exists() {
        let dir = TestDir::new();

        let vault = Vault::open(&dir.path, &MachineLocalKeyProvider).unwrap();
        vault.set("llm_api_key", "sk-123").unwrap();
        std::fs::remove_file(dir.path.join(KEY_FILE)).unwrap();

        let reopened = Vault::open(&dir.path, &MachineLocalKeyProvider);
        assert!(matches!(reopened, Err(VaultError::KeyMissing)));
    }

    #[test]
    fn first_open_creates_key_file() {
        let dir = TestDir::new();

        let vault = Vault::open(&dir.path, &MachineLocalKeyProvider).unwrap();
        assert!(dir.path.join(KEY_FILE).exists());
        assert_eq!(vault.get("missing"), None);
    }

    #[test]
    fn supports_parallel_sets_for_distinct_keys() {
        let dir = TestDir::new();
        let vault = Arc::new(Vault::open(&dir.path, &MachineLocalKeyProvider).unwrap());

        let mut handles = Vec::new();
        for idx in 0..8 {
            let shared = Arc::clone(&vault);
            handles.push(std::thread::spawn(move || {
                shared
                    .set(format!("k{idx}").as_str(), format!("v{idx}").as_str())
                    .unwrap();
            }));
        }
        for handle in handles {
            handle.join().unwrap();
        }

        for idx in 0..8 {
            assert_eq!(
                vault.get(format!("k{idx}").as_str()),
                Some(format!("v{idx}"))
            );
        }
    }

    #[test]
    fn concurrent_writes_to_same_key_keep_complete_value() {
        let dir = TestDir::new();
        let vault = Arc::new(Vault::open(&dir.path, &MachineLocalKeyProvider).unwrap());
        let candidates: Vec<String> = (0..12).map(|idx| format!("token-{idx}")).collect();

        let mut handles = Vec::new();
        for value in candidates.clone() {
            let shared = Arc::clone(&vault);
            handles.push(std::thread::spawn(move || {
                for _ in 0..20 {
                    shared.set("shared", value.as_str()).unwrap();
                }
            }));
        }
        for handle in handles {
            handle.join().unwrap();
        }

        let final_value = vault.get("shared").expect("shared key should exist");
        assert!(candidates.iter().any(|candidate| candidate == &final_value));

        let reopened = Vault::open(&dir.path, &MachineLocalKeyProvider).unwrap();
        let persisted = reopened
            .get("shared")
            .expect("shared key should exist after reopen");
        assert_eq!(persisted, final_value);
    }

    #[test]
    fn set_rolls_back_existing_value_when_persist_fails() {
        let dir = TestDir::new();
        let vault = Vault::open(&dir.path, &MachineLocalKeyProvider).unwrap();
        vault.set("llm_api_key", "old-value").unwrap();

        std::fs::create_dir_all(dir.path.join("vault.tmp")).unwrap();
        let result = vault.set("llm_api_key", "new-value");

        assert!(matches!(result, Err(VaultError::WriteFailed(_))));
        assert_eq!(vault.get("llm_api_key").as_deref(), Some("old-value"));
    }

    #[test]
    fn set_rolls_back_new_key_when_persist_fails() {
        let dir = TestDir::new();
        let vault = Vault::open(&dir.path, &MachineLocalKeyProvider).unwrap();

        std::fs::create_dir_all(dir.path.join("vault.tmp")).unwrap();
        let result = vault.set("llm_api_key", "new-value");

        assert!(matches!(result, Err(VaultError::WriteFailed(_))));
        assert_eq!(vault.get("llm_api_key"), None);
    }
}
