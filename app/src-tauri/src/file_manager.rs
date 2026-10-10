use anyhow::{ensure, Context, Result};
use std::{
    os::windows::ffi::OsStrExt,
    path::Path,
    time::{Duration, Instant},
};
use windows::{
    core::{Interface, PCWSTR},
    Win32::{
        Foundation::RPC_E_CHANGED_MODE,
        System::{
            Com::{
                CoCreateInstance, CoInitializeEx, CoTaskMemFree, CoUninitialize, IBindCtx,
                IServiceProvider, CLSCTX_ALL, COINIT_APARTMENTTHREADED,
            },
            Variant::VARIANT,
        },
        UI::Shell::{
            Common::ITEMIDLIST, IFolderView2, ILFindLastID, ILIsEqual, IPersistFolder2,
            IShellBrowser, IShellView, IShellWindows, SHOpenFolderAndSelectItems,
            SHParseDisplayName, SID_STopLevelBrowser, ShellWindows, SVSI_DESELECTOTHERS,
            SVSI_ENSUREVISIBLE, SVSI_FOCUSED, SVSI_SELECT,
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
    // The borrowed child stays valid while `item` is alive.
    let children = [unsafe { ILFindLastID(item.0) }.cast_const()];
    unsafe { SHOpenFolderAndSelectItems(folder.0, Some(&children), 0) }
        .with_context(|| format!("could not show {} in the file manager", path.display()))?;
    // Opening a folder can report success before its asynchronous view has
    // applied the selection. Confirm it in the matching view and repair it there.
    let windows: IShellWindows = unsafe { CoCreateInstance(&ShellWindows, None, CLSCTX_ALL) }?;
    let deadline = Instant::now() + Duration::from_secs(5);
    let mut last_error = None;
    loop {
        match select_in_matching_view(&windows, folder.0, children[0]) {
            Ok(true) => return Ok(()),
            Ok(false) => {}
            Err(error) => last_error = Some(error),
        }
        if Instant::now() >= deadline {
            let message = format!(
                "file manager did not confirm selection of {}",
                path.display()
            );
            return Err(last_error.unwrap_or_else(|| anyhow::anyhow!(message.clone())))
                .context(message);
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}

fn select_in_matching_view(
    windows: &IShellWindows,
    parent: *const ITEMIDLIST,
    child: *const ITEMIDLIST,
) -> Result<bool> {
    for index in 0..unsafe { windows.Count() }? {
        // A window can close or navigate during enumeration. Retry rather than
        // touching another directory or failing because an unrelated view vanished.
        let matching_view = (|| -> Result<Option<(IShellView, IFolderView2)>> {
            let dispatch = unsafe { windows.Item(&VARIANT::from(index)) }?;
            let provider = dispatch.cast::<IServiceProvider>()?;
            let browser: IShellBrowser = unsafe { provider.QueryService(&SID_STopLevelBrowser) }?;
            let view = unsafe { browser.QueryActiveShellView() }?;
            let folder_view = view.cast::<IFolderView2>()?;
            let folder: IPersistFolder2 = unsafe { folder_view.GetFolder() }?;
            let current = ShellItem(unsafe { folder.GetCurFolder() }?);
            if current.0.is_null() || !unsafe { ILIsEqual(current.0, parent) }.as_bool() {
                return Ok(None);
            }
            Ok(Some((view, folder_view)))
        })();
        let Ok(Some((view, folder_view))) = matching_view else {
            continue;
        };
        let selected_flag = SVSI_SELECT.0 as u32;
        if unsafe { folder_view.GetSelectionState(child) }
            .context("could not read the target selection")?
            & selected_flag
            == 0
        {
            let flags =
                (SVSI_SELECT.0 | SVSI_DESELECTOTHERS.0 | SVSI_ENSUREVISIBLE.0 | SVSI_FOCUSED.0)
                    as u32;
            unsafe { view.SelectItem(child, flags) }.context("could not select the target file")?;
        }
        return Ok(unsafe { folder_view.GetSelectionState(child) }
            .context("could not verify the target selection")?
            & selected_flag
            != 0);
    }
    Ok(false)
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
