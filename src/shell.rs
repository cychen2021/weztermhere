//! The COM side: an `IExplorerCommand` that Windows 11's new context menu calls. All `unsafe` FFI lives here.

use std::ffi::c_void;
use std::path::PathBuf;
use std::process::Command;

use windows::Win32::Foundation::CLASS_E_CLASSNOTAVAILABLE;
use windows::Win32::Foundation::CLASS_E_NOAGGREGATION;
use windows::Win32::Foundation::E_FAIL;
use windows::Win32::Foundation::E_NOTIMPL;
use windows::Win32::Foundation::S_FALSE;
use windows::Win32::System::Com::CoTaskMemFree;
use windows::Win32::System::Com::IBindCtx;
use windows::Win32::System::Com::IClassFactory;
use windows::Win32::System::Com::IClassFactory_Impl;
use windows::Win32::UI::Shell::ECF_DEFAULT;
use windows::Win32::UI::Shell::ECS_ENABLED;
use windows::Win32::UI::Shell::IEnumExplorerCommand;
use windows::Win32::UI::Shell::IExplorerCommand;
use windows::Win32::UI::Shell::IExplorerCommand_Impl;
use windows::Win32::UI::Shell::IShellItemArray;
use windows::Win32::UI::Shell::SHStrDupW;
use windows::Win32::UI::Shell::SIGDN_FILESYSPATH;
use windows::core::BOOL;
use windows::core::GUID;
use windows::core::HRESULT;
use windows::core::Interface;
use windows::core::PWSTR;
use windows::core::Ref;
use windows::core::Result;
use windows::core::implement;
use windows::core::w;

use crate::launch;

/// The CLSID of the command; it must match the `Clsid` in `packaging/AppxManifest.xml`.
pub const CLSID_WEZTERM_HERE: GUID = GUID::from_u128(0x7c1b6e52_3f0a_4d6b_9a52_5e8d2c41b7a3);

/// The context menu entry "Open WezTerm here".
#[implement(IExplorerCommand)]
struct WezTermHere;

impl IExplorerCommand_Impl for WezTermHere_Impl {
    fn GetTitle(&self, _items: Ref<IShellItemArray>) -> Result<PWSTR> {
        unsafe { SHStrDupW(w!("在此处打开 WezTerm")) }
    }

    fn GetIcon(&self, _items: Ref<IShellItemArray>) -> Result<PWSTR> {
        match launch::locate() {
            Some(exe) => {
                let wide: Vec<u16> = format!("{},0", exe.display())
                    .encode_utf16()
                    .chain([0])
                    .collect();
                unsafe { SHStrDupW(windows::core::PCWSTR(wide.as_ptr())) }
            }
            None => Err(E_NOTIMPL.into()),
        }
    }

    fn GetToolTip(&self, _items: Ref<IShellItemArray>) -> Result<PWSTR> {
        Err(E_NOTIMPL.into())
    }

    fn GetCanonicalName(&self) -> Result<GUID> {
        Ok(CLSID_WEZTERM_HERE)
    }

    fn GetState(&self, _items: Ref<IShellItemArray>, _slow: BOOL) -> Result<u32> {
        Ok(ECS_ENABLED.0 as u32)
    }

    fn Invoke(&self, items: Ref<IShellItemArray>, _bind: Ref<IBindCtx>) -> Result<()> {
        let dir = folder_of(items.as_ref())?;
        let exe = launch::locate().ok_or_else(|| windows::core::Error::from(E_FAIL))?;
        Command::new(exe)
            .args(launch::wezterm_args(&dir))
            .current_dir(&dir)
            .spawn()
            .map_err(|_| windows::core::Error::from(E_FAIL))?;
        Ok(())
    }

    fn GetFlags(&self) -> Result<u32> {
        Ok(ECF_DEFAULT.0 as u32)
    }

    fn EnumSubCommands(&self) -> Result<IEnumExplorerCommand> {
        Err(E_NOTIMPL.into())
    }
}

/// Returns the file system path of the first item (a folder, or the folder whose background was clicked).
fn folder_of(items: Option<&IShellItemArray>) -> Result<PathBuf> {
    let items = items.ok_or_else(|| windows::core::Error::from(E_FAIL))?;
    unsafe {
        let item = items.GetItemAt(0)?;
        let name = item.GetDisplayName(SIGDN_FILESYSPATH)?;
        let path = name.to_string();
        CoTaskMemFree(Some(name.0 as *const c_void));
        path.map(PathBuf::from)
            .map_err(|_| windows::core::Error::from(E_FAIL))
    }
}

/// Hands out [`WezTermHere`] instances to the shell.
#[implement(IClassFactory)]
struct Factory;

impl IClassFactory_Impl for Factory_Impl {
    fn CreateInstance(
        &self,
        outer: Ref<windows::core::IUnknown>,
        iid: *const GUID,
        object: *mut *mut c_void,
    ) -> Result<()> {
        if outer.is_some() {
            return Err(CLASS_E_NOAGGREGATION.into());
        }
        let command: IExplorerCommand = WezTermHere.into();
        unsafe { command.query(iid, object).ok() }
    }

    fn LockServer(&self, _lock: BOOL) -> Result<()> {
        Ok(())
    }
}

/// The COM entry point: returns the class factory for [`CLSID_WEZTERM_HERE`].
///
/// # Safety
///
/// Called by COM with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "system" fn DllGetClassObject(
    clsid: *const GUID,
    iid: *const GUID,
    object: *mut *mut c_void,
) -> HRESULT {
    unsafe {
        if clsid.is_null() || iid.is_null() || object.is_null() {
            return E_FAIL;
        }
        if *clsid != CLSID_WEZTERM_HERE {
            return CLASS_E_CLASSNOTAVAILABLE;
        }
        let factory: IClassFactory = Factory.into();
        factory.query(iid, object)
    }
}

/// The COM entry point that says whether the DLL may be unloaded. The shell keeps it loaded, so the answer is no.
#[unsafe(no_mangle)]
pub extern "system" fn DllCanUnloadNow() -> HRESULT {
    S_FALSE
}
