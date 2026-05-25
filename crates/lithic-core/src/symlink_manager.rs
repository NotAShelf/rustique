#[cfg(unix)]
use tokio::fs::symlink;

#[cfg(windows)]
use tokio::fs::{symlink_dir, symlink_file};

use crate::errors::LithicError;
use std::fs;
use std::path::Path;

pub struct SymlinkManager;

impl SymlinkManager {
   /// Manage symlink creation
   pub async fn create(target: impl AsRef<Path>, link: impl AsRef<Path>) -> Result<(), LithicError> {
      let (target, link) = (target.as_ref(), link.as_ref());
      #[cfg(unix)]
      symlink(target, link)
         .await
         .map_err(|e| LithicError::SimpleError(e.to_string()))?;

      #[cfg(windows)]
      if target.is_dir() {
         symlink_dir(target, link)
            .await
            .map_err(|e| LithicError::SimpleError(e.to_string()))?;
      } else {
         symlink_file(target, link)
            .await
            .map_err(|e| LithicError::SimpleError(e.to_string()))?;
      }

      Ok(())
   }

   pub fn remove(path: impl AsRef<Path>) -> Result<(), LithicError> {
      fs::remove_file(path.as_ref()).map_err(|e| LithicError::SimpleError(e.to_string()))?;

      Ok(())
   }

   /// Checks if `path` is a symlink
   pub fn exists(path: impl AsRef<Path>) -> bool {
      path.as_ref().is_symlink()
   }
}
