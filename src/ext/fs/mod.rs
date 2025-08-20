use std::{borrow::Cow, path::Path};

use deno_core::extension;
use deno_permissions::{CheckedPath, PermissionCheckError, PermissionDeniedError};

use super::web::PermissionsContainer;
use crate::ext::ExtensionList;

extension!(
    fs,
    deps = [rustyscript],
    esm_entry_point = "ext:fs/init_fs.js",
    esm = [ dir "src/ext/fs", "init_fs.js" ],
);

pub fn load(extensions: &mut ExtensionList) {
    let options = extensions.options();
    extensions.extend([
        deno_fs::deno_fs::init::<PermissionsContainer>(options.filesystem.clone()),
        fs::init(),
    ]);
}

impl deno_fs::FsPermissions for PermissionsContainer {
    fn check_open<'a>(
        &self,
        path: Cow<'a, Path>,
        access_kind: deno_permissions::OpenAccessKind,
        api_name: &str,
    ) -> Result<CheckedPath<'a>, PermissionCheckError> {
        let read = access_kind.is_read();
        let write = access_kind.is_write();

        let p = self.0.check_open(true, read, write, path, api_name).ok_or(
            PermissionCheckError::PermissionDenied(PermissionDeniedError {
                access: api_name.to_string(),
                name: "open",
            }),
        )?;

        Ok(CheckedPath::unsafe_new(p))
    }

    fn check_open_blind<'a>(
        &self,
        path: Cow<'a, Path>,
        access_kind: deno_permissions::OpenAccessKind,
        display: &str,
        api_name: &str,
    ) -> Result<CheckedPath<'a>, PermissionCheckError> {
        if access_kind.is_read() {
            self.0.check_read_all(Some(api_name))?;
            self.0.check_read_blind(&path, display, api_name)?;
        }

        if access_kind.is_write() {
            self.0.check_write_all(api_name)?;
            self.0.check_write_blind(&path, display, api_name)?;
        }

        Ok(CheckedPath::unsafe_new(path))
    }

    fn check_read_all(&self, api_name: &str) -> Result<(), PermissionCheckError> {
        self.0.check_read_all(Some(api_name))?;
        Ok(())
    }

    fn check_write_partial<'a>(
        &self,
        path: Cow<'a, Path>,
        api_name: &str,
    ) -> Result<CheckedPath<'a>, PermissionCheckError> {
        self.0.check_write_all(api_name)?;
        let p = self.0.check_write_partial(path, api_name)?;

        Ok(CheckedPath::unsafe_new(p))
    }

    fn check_write_all(&self, api_name: &str) -> Result<(), PermissionCheckError> {
        self.0.check_write_all(api_name)?;
        Ok(())
    }
}
