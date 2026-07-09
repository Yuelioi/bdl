use std::fmt;
use std::fs;
use std::path::Path;
use std::sync::Arc;

use bdl_core::{BdlError, BdlResult};

const SERVICE_NAME: &str = "bdl-downloader";
const ACCOUNT_COOKIE_USER: &str = "bilibili-account-cookie";

#[derive(Clone)]
pub struct SecureStore {
    backend: Arc<dyn CredentialBackend>,
}

impl fmt::Debug for SecureStore {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SecureStore")
            .field("service", &SERVICE_NAME)
            .field("user", &ACCOUNT_COOKIE_USER)
            .finish_non_exhaustive()
    }
}

impl SecureStore {
    pub fn new() -> Self {
        Self {
            backend: Arc::new(KeyringCredentialBackend {
                service: SERVICE_NAME,
                user: ACCOUNT_COOKIE_USER,
            }),
        }
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

    pub fn migrate_legacy_cookie_file(&self, path: impl AsRef<Path>) -> BdlResult<Option<String>> {
        let path = path.as_ref();
        let cookie = match fs::read_to_string(path) {
            Ok(cookie) => cookie.trim().to_owned(),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(error.into()),
        };

        if cookie.is_empty() {
            remove_legacy_cookie_file(path)?;
            return Ok(None);
        }

        self.save_cookie(&cookie)?;
        remove_legacy_cookie_file(path)?;
        Ok(Some(cookie))
    }

    pub fn clear_legacy_cookie_file(path: impl AsRef<Path>) -> BdlResult<()> {
        remove_legacy_cookie_file(path.as_ref())
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

#[derive(Debug)]
struct KeyringCredentialBackend {
    service: &'static str,
    user: &'static str,
}

impl KeyringCredentialBackend {
    fn entry(&self) -> Result<keyring::Entry, keyring::Error> {
        keyring::Entry::new(self.service, self.user)
    }
}

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

fn keyring_error(action: &str, error: keyring::Error) -> BdlError {
    BdlError::Account {
        message: format!("{action}失败：{error}"),
    }
}

fn remove_legacy_cookie_file(path: &Path) -> BdlResult<()> {
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
    fn secure_store_migrates_legacy_cookie_file_into_backend() -> BdlResult<()> {
        let path = temp_store_path();
        let parent = path.parent().expect("temp path should have parent");
        fs::create_dir_all(parent)?;
        fs::write(&path, "DedeUserID=42; SESSDATA=session; bili_jct=csrf\n")?;
        let store = SecureStore::in_memory();

        let migrated = store.migrate_legacy_cookie_file(&path)?;

        assert_eq!(
            migrated,
            Some("DedeUserID=42; SESSDATA=session; bili_jct=csrf".to_owned())
        );
        assert_eq!(
            store.load_cookie()?,
            Some("DedeUserID=42; SESSDATA=session; bili_jct=csrf".to_owned())
        );
        assert!(!path.exists());

        let _ = fs::remove_dir_all(parent);
        Ok(())
    }

    fn temp_store_path() -> std::path::PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or_default();

        std::env::temp_dir()
            .join(format!("bdl-secure-store-{nanos}"))
            .join("account.cookie")
    }
}
