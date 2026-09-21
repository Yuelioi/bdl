use std::fmt;
#[cfg(any(target_os = "macos", test))]
use std::fs;
use std::path::Path;
use std::sync::Arc;

use bdl_core::{BdlError, BdlResult};
#[cfg(any(target_os = "macos", test))]
use ring::aead::{Aad, CHACHA20_POLY1305, LessSafeKey, Nonce, UnboundKey};
#[cfg(any(target_os = "macos", test))]
use ring::rand::{SecureRandom, SystemRandom};
#[cfg(any(target_os = "macos", test))]
use std::path::PathBuf;

#[cfg(not(target_os = "macos"))]
const SERVICE_NAME: &str = "bdl-downloader";
#[cfg(not(target_os = "macos"))]
const ACCOUNT_COOKIE_USER: &str = "bilibili-account-cookie";

#[cfg(target_os = "macos")]
const CREDENTIAL_DIR_NAME: &str = "credentials";
#[cfg(any(target_os = "macos", test))]
const CREDENTIAL_KEY_FILE_NAME: &str = "credentials.key";
#[cfg(any(target_os = "macos", test))]
const ACCOUNT_COOKIE_FILE_NAME: &str = "account.cookie.enc";
#[cfg(any(target_os = "macos", test))]
const ENCRYPTED_COOKIE_MAGIC: &[u8; 8] = b"BDLCRED1";
#[cfg(any(target_os = "macos", test))]
const KEY_LEN: usize = 32;
#[cfg(any(target_os = "macos", test))]
const NONCE_LEN: usize = 12;

#[derive(Clone)]
pub struct SecureStore {
    backend: Arc<dyn CredentialBackend>,
}

impl fmt::Debug for SecureStore {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SecureStore")
            .finish_non_exhaustive()
    }
}

impl SecureStore {
    pub fn new(data_dir: impl AsRef<Path>) -> Self {
        #[cfg(target_os = "macos")]
        let backend: Arc<dyn CredentialBackend> = Arc::new(EncryptedFileCredentialBackend::new(
            data_dir.as_ref().join(CREDENTIAL_DIR_NAME),
        ));

        #[cfg(not(target_os = "macos"))]
        let backend: Arc<dyn CredentialBackend> = {
            let _ = data_dir;
            Arc::new(KeyringCredentialBackend {
                service: SERVICE_NAME,
                user: ACCOUNT_COOKIE_USER,
            })
        };

        Self { backend }
    }

    pub fn load_cookie(&self) -> BdlResult<Option<String>> {
        self.backend.load_cookie()
    }

    pub fn save_cookie(&self, cookie: &str) -> BdlResult<()> {
        self.backend.save_cookie(cookie)
    }

    pub fn clear_cookie(&self) -> BdlResult<()> {
        self.backend.clear_cookie()
    }

    #[cfg(test)]
    pub(crate) fn in_memory() -> Self {
        Self {
            backend: Arc::new(InMemoryCredentialBackend::default()),
        }
    }
}

trait CredentialBackend: Send + Sync {
    fn load_cookie(&self) -> BdlResult<Option<String>>;
    fn save_cookie(&self, cookie: &str) -> BdlResult<()>;
    fn clear_cookie(&self) -> BdlResult<()>;
}

#[cfg(not(target_os = "macos"))]
#[derive(Debug)]
struct KeyringCredentialBackend {
    service: &'static str,
    user: &'static str,
}

#[cfg(not(target_os = "macos"))]
impl KeyringCredentialBackend {
    fn entry(&self) -> Result<keyring::Entry, keyring::Error> {
        keyring::Entry::new(self.service, self.user)
    }
}

#[cfg(not(target_os = "macos"))]
impl CredentialBackend for KeyringCredentialBackend {
    fn load_cookie(&self) -> BdlResult<Option<String>> {
        let entry = self
            .entry()
            .map_err(|error| keyring_error("初始化系统凭据项", error))?;
        match entry.get_password() {
            Ok(cookie) if cookie.trim().is_empty() => Ok(None),
            Ok(cookie) => Ok(Some(cookie)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(error) => Err(keyring_error("读取账户 Cookie", error)),
        }
    }

    fn save_cookie(&self, cookie: &str) -> BdlResult<()> {
        let entry = self
            .entry()
            .map_err(|error| keyring_error("初始化系统凭据项", error))?;
        entry
            .set_password(cookie)
            .map_err(|error| keyring_error("保存账户 Cookie", error))
    }

    fn clear_cookie(&self) -> BdlResult<()> {
        let entry = self
            .entry()
            .map_err(|error| keyring_error("初始化系统凭据项", error))?;
        match entry.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(error) => Err(keyring_error("删除账户 Cookie", error)),
        }
    }
}

#[cfg(not(target_os = "macos"))]
fn keyring_error(action: &str, error: keyring::Error) -> BdlError {
    #[cfg(target_os = "linux")]
    let hint = "。请确认桌面会话的 Secret Service 凭据服务（例如 GNOME Keyring）已启动且已解锁。";
    #[cfg(not(target_os = "linux"))]
    let hint = "";
    BdlError::Account {
        message: format!("{action}失败：{error}{hint}"),
    }
}

#[cfg(any(target_os = "macos", test))]
#[derive(Debug)]
struct EncryptedFileCredentialBackend {
    directory: PathBuf,
    key_path: PathBuf,
    cookie_path: PathBuf,
}

#[cfg(any(target_os = "macos", test))]
impl EncryptedFileCredentialBackend {
    fn new(directory: PathBuf) -> Self {
        Self {
            key_path: directory.join(CREDENTIAL_KEY_FILE_NAME),
            cookie_path: directory.join(ACCOUNT_COOKIE_FILE_NAME),
            directory,
        }
    }

    fn load_key(&self) -> BdlResult<[u8; KEY_LEN]> {
        let key =
            fs::read(&self.key_path).map_err(|error| credential_io_error("读取凭据密钥", error))?;
        key.try_into().map_err(|_| BdlError::Account {
            message: "凭据密钥格式无效，请退出登录后重新登录。".to_owned(),
        })
    }

    fn load_or_create_key(&self) -> BdlResult<[u8; KEY_LEN]> {
        match fs::read(&self.key_path) {
            Ok(key) => key.try_into().map_err(|_| BdlError::Account {
                message: "凭据密钥格式无效，请退出登录后重新登录。".to_owned(),
            }),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                fs::create_dir_all(&self.directory)
                    .map_err(|error| credential_io_error("创建凭据目录", error))?;
                set_private_directory_permissions(&self.directory)?;

                let mut key = [0_u8; KEY_LEN];
                SystemRandom::new()
                    .fill(&mut key)
                    .map_err(|_| credential_crypto_error("生成凭据密钥"))?;
                write_private_file(&self.key_path, &key)?;
                Ok(key)
            }
            Err(error) => Err(credential_io_error("读取凭据密钥", error)),
        }
    }

    fn cipher(key: &[u8; KEY_LEN]) -> BdlResult<LessSafeKey> {
        let key = UnboundKey::new(&CHACHA20_POLY1305, key)
            .map_err(|_| credential_crypto_error("初始化凭据加密"))?;
        Ok(LessSafeKey::new(key))
    }
}

#[cfg(any(target_os = "macos", test))]
impl CredentialBackend for EncryptedFileCredentialBackend {
    fn load_cookie(&self) -> BdlResult<Option<String>> {
        let payload = match fs::read(&self.cookie_path) {
            Ok(payload) => payload,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(credential_io_error("读取加密账户凭据", error)),
        };

        let header_len = ENCRYPTED_COOKIE_MAGIC.len() + NONCE_LEN;
        if payload.len() <= header_len || !payload.starts_with(ENCRYPTED_COOKIE_MAGIC) {
            return Err(BdlError::Account {
                message: "加密账户凭据格式无效，请退出登录后重新登录。".to_owned(),
            });
        }

        let key = self.load_key()?;
        let cipher = Self::cipher(&key)?;
        let nonce_bytes: [u8; NONCE_LEN] = payload[ENCRYPTED_COOKIE_MAGIC.len()..header_len]
            .try_into()
            .map_err(|_| credential_crypto_error("读取凭据随机数"))?;
        let mut encrypted = payload[header_len..].to_vec();
        let plain = cipher
            .open_in_place(
                Nonce::assume_unique_for_key(nonce_bytes),
                Aad::from(&ENCRYPTED_COOKIE_MAGIC[..]),
                &mut encrypted,
            )
            .map_err(|_| BdlError::Account {
                message: "账户凭据无法解密，请退出登录后重新登录。".to_owned(),
            })?;
        let cookie = std::str::from_utf8(plain).map_err(|_| BdlError::Account {
            message: "账户凭据内容无效，请退出登录后重新登录。".to_owned(),
        })?;

        if cookie.trim().is_empty() {
            Ok(None)
        } else {
            Ok(Some(cookie.to_owned()))
        }
    }

    fn save_cookie(&self, cookie: &str) -> BdlResult<()> {
        fs::create_dir_all(&self.directory)
            .map_err(|error| credential_io_error("创建凭据目录", error))?;
        set_private_directory_permissions(&self.directory)?;

        let key = self.load_or_create_key()?;
        let cipher = Self::cipher(&key)?;
        let mut nonce_bytes = [0_u8; NONCE_LEN];
        SystemRandom::new()
            .fill(&mut nonce_bytes)
            .map_err(|_| credential_crypto_error("生成凭据随机数"))?;

        let mut encrypted = cookie.as_bytes().to_vec();
        cipher
            .seal_in_place_append_tag(
                Nonce::assume_unique_for_key(nonce_bytes),
                Aad::from(&ENCRYPTED_COOKIE_MAGIC[..]),
                &mut encrypted,
            )
            .map_err(|_| credential_crypto_error("加密账户凭据"))?;

        let mut payload =
            Vec::with_capacity(ENCRYPTED_COOKIE_MAGIC.len() + NONCE_LEN + encrypted.len());
        payload.extend_from_slice(ENCRYPTED_COOKIE_MAGIC);
        payload.extend_from_slice(&nonce_bytes);
        payload.extend_from_slice(&encrypted);
        write_private_file(&self.cookie_path, &payload)
    }

    fn clear_cookie(&self) -> BdlResult<()> {
        remove_file_if_exists(&self.cookie_path)?;
        remove_file_if_exists(&self.key_path)?;
        Ok(())
    }
}

#[cfg(any(target_os = "macos", test))]
fn credential_io_error(action: &str, error: std::io::Error) -> BdlError {
    BdlError::Account {
        message: format!("{action}失败：{error}"),
    }
}

#[cfg(any(target_os = "macos", test))]
fn credential_crypto_error(action: &str) -> BdlError {
    BdlError::Account {
        message: format!("{action}失败。"),
    }
}

#[cfg(all(any(target_os = "macos", test), unix))]
fn write_private_file(path: &Path, contents: &[u8]) -> BdlResult<()> {
    use std::io::Write;
    use std::os::unix::fs::OpenOptionsExt;

    let mut file = fs::OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .mode(0o600)
        .open(path)
        .map_err(|error| credential_io_error("写入账户凭据", error))?;
    file.write_all(contents)
        .map_err(|error| credential_io_error("写入账户凭据", error))?;
    file.sync_all()
        .map_err(|error| credential_io_error("同步账户凭据", error))
}

#[cfg(all(any(target_os = "macos", test), not(unix)))]
fn write_private_file(path: &Path, contents: &[u8]) -> BdlResult<()> {
    fs::write(path, contents).map_err(|error| credential_io_error("写入账户凭据", error))
}

#[cfg(all(any(target_os = "macos", test), unix))]
fn set_private_directory_permissions(path: &Path) -> BdlResult<()> {
    use std::os::unix::fs::PermissionsExt;

    fs::set_permissions(path, fs::Permissions::from_mode(0o700))
        .map_err(|error| credential_io_error("设置凭据目录权限", error))
}

#[cfg(all(any(target_os = "macos", test), not(unix)))]
fn set_private_directory_permissions(_path: &Path) -> BdlResult<()> {
    Ok(())
}

#[cfg(any(target_os = "macos", test))]
fn remove_file_if_exists(path: &Path) -> BdlResult<()> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.into()),
    }
}

#[cfg(test)]
#[derive(Default)]
struct InMemoryCredentialBackend {
    cookie: std::sync::Mutex<Option<String>>,
}

#[cfg(test)]
impl CredentialBackend for InMemoryCredentialBackend {
    fn load_cookie(&self) -> BdlResult<Option<String>> {
        Ok(self
            .cookie
            .lock()
            .map_err(|_| BdlError::Account {
                message: "测试凭据存储锁已损坏。".to_owned(),
            })?
            .clone())
    }

    fn save_cookie(&self, cookie: &str) -> BdlResult<()> {
        *self.cookie.lock().map_err(|_| BdlError::Account {
            message: "测试凭据存储锁已损坏。".to_owned(),
        })? = Some(cookie.to_owned());
        Ok(())
    }

    fn clear_cookie(&self) -> BdlResult<()> {
        *self.cookie.lock().map_err(|_| BdlError::Account {
            message: "测试凭据存储锁已损坏。".to_owned(),
        })? = None;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;

    #[test]
    fn secure_store_persists_cookie_until_cleared() -> BdlResult<()> {
        let store = SecureStore::in_memory();

        store.save_cookie("DedeUserID=42; SESSDATA=session; bili_jct=csrf")?;
        assert_eq!(
            store.load_cookie()?,
            Some("DedeUserID=42; SESSDATA=session; bili_jct=csrf".to_owned())
        );

        store.clear_cookie()?;
        assert_eq!(store.load_cookie()?, None);

        Ok(())
    }

    #[test]
    fn encrypted_file_backend_round_trips_without_plaintext_on_disk() -> BdlResult<()> {
        let directory = temp_store_dir();
        let backend = EncryptedFileCredentialBackend::new(directory.clone());
        let cookie = "DedeUserID=42; SESSDATA=session; bili_jct=csrf";

        backend.save_cookie(cookie)?;

        let encrypted = fs::read(directory.join(ACCOUNT_COOKIE_FILE_NAME))?;
        assert!(
            !encrypted
                .windows(cookie.len())
                .any(|window| window == cookie.as_bytes())
        );
        assert_eq!(backend.load_cookie()?, Some(cookie.to_owned()));

        let reloaded = EncryptedFileCredentialBackend::new(directory.clone());
        assert_eq!(reloaded.load_cookie()?, Some(cookie.to_owned()));

        reloaded.clear_cookie()?;
        assert!(!directory.join(ACCOUNT_COOKIE_FILE_NAME).exists());
        assert!(!directory.join(CREDENTIAL_KEY_FILE_NAME).exists());

        let _ = fs::remove_dir_all(directory);
        Ok(())
    }

    fn temp_store_dir() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or_default();

        std::env::temp_dir().join(format!("bdl-secure-store-{nanos}"))
    }
}
