use anyhow::{ensure, Context, Result};
use std::{os::windows::ffi::OsStrExt, path::Path};
use windows::{
    core::PCWSTR,
    Win32::{
        Foundation::RPC_E_CHANGED_MODE,
        System::Com::{
            CoInitializeEx, CoTaskMemFree, CoUninitialize, IBindCtx, COINIT_APARTMENTTHREADED,
        },
        UI::Shell::{
            Common::ITEMIDLIST, ILFindLastID, SHOpenFolderAndSelectItems, SHParseDisplayName,
        },
    },
};

struct Apartment(bool);

impl Apartment {
    fn initialize() -> Result<Self> {
        // Tauri may have initialized this worker in a different COM apartment.
        let result = unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) };
        if result == RPC_E_CHANGED_MODE {
            return Ok(Self(false));
        }
        result
            .ok()
            .context("could not initialize the file manager")?;
        Ok(Self(true))
    }
}

impl Drop for Apartment {
    fn drop(&mut self) {
        if self.0 {
            unsafe { CoUninitialize() };
        }
    }
}

struct ShellItem(*mut ITEMIDLIST);

impl Drop for ShellItem {
    fn drop(&mut self) {
        unsafe { CoTaskMemFree(Some(self.0.cast())) };
    }
}

fn parse_item(path: &Path) -> Result<ShellItem> {
    // Shell paths use ordinary drive/UNC syntax instead of Rust's canonical
    // verbatim prefix. Keep the native wide string, including Unicode names.
    let mut name: Vec<u16> = dunce::simplified(path)
        .as_os_str()
        .encode_wide()
        // Project/archive metadata uses forward slashes, which the Shell's
        // display-name parser rejects even though filesystem APIs accept them.
        .map(|character| {
            if character == u16::from(b'/') {
                u16::from(b'\\')
            } else {
                character
            }
        })
        .collect();
    ensure!(
        !name.contains(&0),
        "file manager path contains a null character"
    );
    name.push(0);
    let mut item = std::ptr::null_mut();
    unsafe { SHParseDisplayName(PCWSTR(name.as_ptr()), None::<&IBindCtx>, &mut item, 0, None) }
        .with_context(|| format!("could not locate {} in the file manager", path.display()))?;
    ensure!(!item.is_null(), "file manager returned an empty item");
    Ok(ShellItem(item))
}

pub(crate) fn reveal(path: &Path) -> Result<()> {
    let _apartment = Apartment::initialize()?;
    let item = parse_item(path)?;
    let parent = path.parent().context("file manager path has no parent")?;
    let folder = parse_item(parent)?;
    // Pass the containing folder and an explicit relative child selection.
    // The zero-child shortcut can open the folder without selecting the file
    // on Windows Server. The borrowed child stays valid while `item` is alive.
    let children = [unsafe { ILFindLastID(item.0) }.cast_const()];
    unsafe { SHOpenFolderAndSelectItems(folder.0, Some(&children), 0) }
        .with_context(|| format!("could not show {} in the file manager", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use windows::Win32::UI::Shell::{ILCombine, SHGetPathFromIDListEx, GPFIDL_FLAGS};

    #[test]
    fn shell_items_resolve_exact_files_in_unicode_and_punctuated_directories() {
        let temp = tempfile::tempdir().unwrap();
        let _apartment = Apartment::initialize().unwrap();
        for directory in ["论文相关", "研究 文件, #", "archive", "trash"] {
            let parent = temp.path().join(directory);
            std::fs::create_dir(&parent).unwrap();
            let path = parent.join("笔记, #.md");
            std::fs::write(&path, "note").unwrap();
            // External files arrive with the verbatim prefix after approval.
            let canonical = path.canonicalize().unwrap();
            // Project/archive relative paths can have mixed separators.
            let mixed = temp.path().join(format!("{directory}/笔记, #.md"));
            for candidate in [&path, &canonical, &mixed] {
                let item = parse_item(candidate).unwrap();
                let folder = parse_item(candidate.parent().unwrap()).unwrap();
                let combined =
                    ShellItem(unsafe { ILCombine(Some(folder.0), Some(ILFindLastID(item.0))) });
                assert!(!combined.0.is_null());
                let mut buffer = vec![0_u16; 1024];
                assert!(
                    unsafe { SHGetPathFromIDListEx(combined.0, &mut buffer, GPFIDL_FLAGS(0)) }
                        .as_bool()
                );
                let end = buffer.iter().position(|&value| value == 0).unwrap();
                let resolved =
                    std::path::PathBuf::from(String::from_utf16(&buffer[..end]).unwrap());
                assert_eq!(resolved.canonicalize().unwrap(), canonical);
            }
        }
    }

    #[test]
    fn missing_files_report_an_error_instead_of_opening_a_default_folder() {
        let temp = tempfile::tempdir().unwrap();
        assert!(reveal(&temp.path().join("missing.md")).is_err());
    }
}
