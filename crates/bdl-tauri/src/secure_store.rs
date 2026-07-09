use std::fs;
use std::path::PathBuf;

use bdl_core::BdlResult;

#[derive(Debug, Clone)]
pub struct SecureStore {
    path: PathBuf,
}

impl SecureStore {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn load_cookie(&self) -> BdlResult<Option<String>> {
        match fs::read_to_string(&self.path) {
            Ok(cookie) if cookie.trim().is_empty() => Ok(None),
            Ok(cookie) => Ok(Some(cookie)),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(error.into()),
        }
    }

    pub fn save_cookie(&self, cookie: &str) -> BdlResult<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(&self.path, cookie)?;
        Ok(())
    }

    pub fn clear_cookie(&self) -> BdlResult<()> {
        match fs::remove_file(&self.path) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(error.into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;

    #[test]
    fn secure_store_persists_cookie_until_cleared() -> BdlResult<()> {
        let path = temp_store_path();
        let store = SecureStore::new(path.clone());

        store.save_cookie("DedeUserID=42; SESSDATA=session; bili_jct=csrf")?;
        assert_eq!(
            store.load_cookie()?,
            Some("DedeUserID=42; SESSDATA=session; bili_jct=csrf".to_owned())
        );

        store.clear_cookie()?;
        assert_eq!(store.load_cookie()?, None);

        if let Some(parent) = path.parent() {
            let _ = fs::remove_dir_all(parent);
        }

        Ok(())
    }

    fn temp_store_path() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or_default();

        std::env::temp_dir()
            .join(format!("bdl-secure-store-{nanos}"))
            .join("account.cookie")
    }
}
