/* -----------------------------------------------------------------------------------
 * src/window.rs - A wrapper around a GUI window.
 * porcupine - Safe wrapper around the graphical parts of Win32.
 * Copyright © 2020 not_a_seagull
 *
 * This project is licensed under either the Apache 2.0 license or the MIT license, at
 * your option. For more information, please consult the LICENSE-APACHE or LICENSE-MIT
 * files in the repository root.
 * -----------------------------------------------------------------------------------
 * MIT License:
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the “Software”), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in
 * all copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED “AS IS”, WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
 * THE SOFTWARE.
 * -----------------------------------------------------------------------------------
 * Apache 2.0 License Declaration:
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 * ----------------------------------------------------------------------------------
 */

use crate::{DeviceContext, WStr, WString};
use euclid::default::Rect;
use std::{
    fmt, mem,
    os::raw::c_int,
    ptr::{self, NonNull},
    sync::{atomic::AtomicPtr, Arc, Mutex, Weak},
};
use winapi::{
    shared::{
        minwindef::{DWORD, FALSE, TRUE, UINT},
        windef::HWND__,
    },
    um::winuser::{self, WINDOWPLACEMENT, WNDCLASSEXW, WNDPROC},
};

static LOCK_MUTEX_PANIC: &'static str = "Unable to lock module info mutex";
static HWND_MUTEX_PANIC: &'static str = "Unable to achieve lock on window handle";

/// An owned, modifyable window class.
#[derive(Clone)]
pub struct OwnedWindowClass {
    inner: WNDCLASSEXW,
    is_registered: bool,
    class_name: Option<WString>,
}

impl fmt::Debug for OwnedWindowClass {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let name = format!("OwnedWindowClass (\"{}\")", self.class_name()?);
        f.debug_struct(&name)
            .field("inner", &format_args!("Window Class Structure")) // TODO: fix to use Debug trait
            .field("is_registered", &self.is_registered)
            .field("class_name", &self.class_name)
            .finish()
    }
}

impl OwnedWindowClass {
    /// Create a new WindowClass that has yet to be initialized.
    pub fn new() -> Self {
        let mut wc = Self {
            inner: unsafe { mem::zeroed() },
            is_registered: false,
            class_name: None,
        };

        wc.inner.cbSize = mem::size_of::<WNDCLASSEXW>() as UINT;
        wc.inner.lpfnWndProc = Some(winuser::DefWindowProcW);
        wc.inner.hInstance = unsafe {
            crate::MODULE_INFO
                .lock()
                .expect(LOCK_MUTEX_PANIC)
                .handle()
                .as_mut()
        };

        wc
    }

    /// Get the name of the class.
    pub fn class_name_ws(&self) -> &WStr {
        unsafe { WStr::from_ptr(self.inner.lpszClassName) }
    }

    /// Get the name of the class.
    pub fn class_name(&self) -> crate::Result<String> {
        self.class_name_ws().into_string()
    }

    /// Set the name of the class.
    pub fn set_class_name(&mut self, name: WString) -> crate::Result<()> {
        self.inner.lpszClassName = name.as_ptr();
        self.class_name = Some(name); // make sure name isn't dropped

        Ok(())
    }

    /// Set the window procedure for the class.
    pub fn set_window_proc(&mut self, wndproc: WNDPROC) {
        self.inner.lpfnWndProc = wndproc;
    }

    /// Get the style for the window class.
    pub fn style(&self) -> UINT {
        self.inner.style
    }

    /// Set the style for the window class.
    pub fn set_style(&mut self, style: UINT) {
        self.inner.style = style;
    }

    /// Register this class. This function will unregister, then re-register the class
    /// if it is already registered.
    pub fn register(&mut self) -> crate::Result<()> {
        // if this is an already registered class, unregister it
        if self.is_registered {
            if unsafe {
                winuser::UnregisterClassW(
                    self.class_name_ws().as_ptr(),
                    crate::MODULE_INFO
                        .lock()
                        .expect(LOCK_MUTEX_PANIC)
                        .handle()
                        .as_mut(),
                )
            } == 0
            {
                return Err(crate::win32_error(crate::Win32Function::UnregisterClassW));
            } else {
                #[cfg(debug_assertions)]
                {
                    self.is_registered = false; // in the unlikely event of an error
                }
            }
        }

        // register the class
        if unsafe { winuser::RegisterClassExW(&self.inner) } == 0 {
            Err(crate::win32_error(crate::Win32Function::RegisterClassExW))
        } else {
            self.is_registered = true;
            Ok(())
        }
    }
}

/// A window class; either a reference to a window class or a full, owned window class.
pub trait WindowClass {
    /// Convert this item into the name of the class.
    fn identifier(&self) -> &WStr;
}

impl WindowClass for OwnedWindowClass {
    fn identifier(&self) -> &WStr {
        self.class_name_ws()
    }
}

impl WindowClass for WString {
    fn identifier(&self) -> &WStr {
        &*self
    }
}

impl WindowClass for WStr {
    fn identifier(&self) -> &WStr {
        self
    }
}

bitflags::bitflags! {
    pub struct WindowStyle : DWORD {
        const NONE = 0;
        const ACTIVE_CAPTION = winuser::WS_ACTIVECAPTION;
        const BORDER = winuser::WS_BORDER;
        const CAPTION = winuser::WS_CAPTION;
        const CHILD = winuser::WS_CHILD;
        const CHILD_WINDOW = winuser::WS_CHILDWINDOW;
        const CLIP_CHILDREN = winuser::WS_CLIPCHILDREN;
        const CLIP_SIBLINGS = winuser::WS_CLIPSIBLINGS;
        const DISABLED = winuser::WS_DISABLED;
        const DLG_FRAME = winuser::WS_DLGFRAME;
        const GROUP = winuser::WS_GROUP;
        const HSCROLL = winuser::WS_HSCROLL;
        const ICONIC = winuser::WS_ICONIC;
        const MAXIMIZE = winuser::WS_MAXIMIZE;
        const MAXIMIZE_BOX = winuser::WS_MAXIMIZEBOX;
        const MINIMIZE = winuser::WS_MINIMIZE;
        const MINIMIZE_BOX = winuser::WS_MINIMIZEBOX;
        const OVERLAPPED = winuser::WS_OVERLAPPED;
        const OVERLAPPED_WINDOW = winuser::WS_OVERLAPPEDWINDOW;
        const POPUP = winuser::WS_POPUP;
        const POPUP_WINDOW = winuser::WS_POPUPWINDOW;
        const SIZEBOX = winuser::WS_SIZEBOX;
        const SYSMENU = winuser::WS_SYSMENU;
        const TAB_STOP = winuser::WS_TABSTOP;
        const THICK_FRAME = winuser::WS_THICKFRAME;
        const TILED = winuser::WS_TILED;
        const TILED_WINDOW = winuser::WS_TILEDWINDOW;
        const VISIBLE = winuser::WS_VISIBLE;
        const VSCROLL = winuser::WS_VSCROLL;
    }
}

bitflags::bitflags! {
    pub struct ExtendedWindowStyle : DWORD {
        const NONE = 0;
        const ACCEPT_FILES = winuser::WS_EX_ACCEPTFILES;
        const APP_WINDOW = winuser::WS_EX_APPWINDOW;
        const CLIENT_EDGE = winuser::WS_EX_CLIENTEDGE;
        const COMPOSITED = winuser::WS_EX_COMPOSITED;
        const CONTEXT_HELP = winuser::WS_EX_CONTEXTHELP;
        const CONTROL_PARENT = winuser::WS_EX_CONTROLPARENT;
        const DLG_MODAL_FRAME = winuser::WS_EX_DLGMODALFRAME;
        const LAYERED = winuser::WS_EX_LAYERED;
        const LAYOUT_RTL = winuser::WS_EX_LAYOUTRTL;
        const LEFT = winuser::WS_EX_LEFT;
        const LEFT_SCROLL_BAR = winuser::WS_EX_LEFTSCROLLBAR;
        const LTR_READING = winuser::WS_EX_LTRREADING;
        const MDI_CHILD = winuser::WS_EX_MDICHILD;
        const NO_ACTIVATE = winuser::WS_EX_NOACTIVATE;
        const NO_INHERIT_LAYOUT = winuser::WS_EX_NOINHERITLAYOUT;
        const NO_PARENT_NOTIFY = winuser::WS_EX_NOPARENTNOTIFY;
        const NO_REDIRECTION_BITMAP = winuser::WS_EX_NOREDIRECTIONBITMAP;
        const OVERLAPPED_WINDOW = winuser::WS_EX_OVERLAPPEDWINDOW;
        const PALETTE_WINDOW = winuser::WS_EX_PALETTEWINDOW;
        const RIGHT = winuser::WS_EX_RIGHT;
        const RIGHT_SCROLL_BAR = winuser::WS_EX_RIGHTSCROLLBAR;
        const RTL_READING = winuser::WS_EX_RTLREADING;
        const STATIC_EDGE = winuser::WS_EX_STATICEDGE;
        const TOOL_WINDOW = winuser::WS_EX_TOOLWINDOW;
        const TOPMOST = winuser::WS_EX_TOPMOST;
        const TRANSPARENT = winuser::WS_EX_TRANSPARENT;
        const WINDOW_EDGE = winuser::WS_EX_WINDOWEDGE;
    }
}

/// Ways to show a window.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum CmdShow {
    Hide = winuser::SW_HIDE,
    ForceMinimize = winuser::SW_FORCEMINIMIZE,
    Maximize = winuser::SW_MAXIMIZE,
    Minimize = winuser::SW_MINIMIZE,
    Show = winuser::SW_SHOW,
    ShowDefault = winuser::SW_SHOWDEFAULT,
    ShowMinimized = winuser::SW_SHOWMINIMIZED,
    ShowNA = winuser::SW_SHOWNA,
    ShowNoActivate = winuser::SW_SHOWNOACTIVATE,
    ShowNormal = winuser::SW_SHOWNORMAL,
}

impl CmdShow {
    /// For consistency.
    #[allow(non_snake_case)]
    #[inline]
    pub fn ShowMaximize() -> Self {
        Self::Maximize
    }
}

/// A wrapper around the Win32 HWND.
pub struct Window {
    hwnd: Arc<Mutex<AtomicPtr<HWND__>>>,
}

impl Window {
    /// Create a new window class.
    pub fn new<WC: WindowClass>(
        window_class: &WC,
        window_name: &WStr,
        style: WindowStyle,
        extended_style: ExtendedWindowStyle,
        bounds: Rect<c_int>,
        parent: &Self,
    ) -> crate::Result<Self> {
        let hwnd = unsafe {
            winuser::CreateWindowExW(
                extended_style.bits(),
                window_class.identifier().as_ptr(),
                window_name.as_ptr(),
                style.bits(),
                bounds.origin.x,
                bounds.origin.y,
                bounds.size.width,
                bounds.size.height,
                parent.hwnd().as_mut(),
                ptr::null_mut(),
                crate::MODULE_INFO
                    .lock()
                    .expect(LOCK_MUTEX_PANIC)
                    .handle()
                    .as_mut(),
                ptr::null_mut(),
            )
        };

        if hwnd.is_null() {
            Err(crate::win32_error(crate::Win32Function::CreateWindowExW))
        } else {
            Ok(Self {
                hwnd: Arc::new(Mutex::new(AtomicPtr::new(hwnd))),
            })
        }
    }

    /// Get the raw handle to this window.
    ///
    /// # Panics
    ///
    /// This function will panic if the internal mutex is unable to be locked.
    pub fn hwnd(&self) -> NonNull<HWND__> {
        let mut p = self.hwnd.lock().expect(HWND_MUTEX_PANIC);
        let ptr = p.get_mut();
        unsafe { NonNull::new_unchecked(*ptr) }
    }

    pub fn weak_reference(&self) -> Weak<Mutex<AtomicPtr<HWND__>>> {
        Arc::downgrade(&self.hwnd)
    }

    /// Change the bounds of this window.
    pub fn reshape(&self, rect: Rect<c_int>) -> crate::Result<()> {
        // create the window placement struct
        let mut wp: WINDOWPLACEMENT = unsafe { mem::zeroed() };
        let mut hwnd = self.hwnd();

        if unsafe { winuser::GetWindowPlacement(hwnd.as_mut(), &mut wp) } == 0 {
            return Err(crate::win32_error(crate::Win32Function::GetWindowPlacement));
        }

        wp.rcNormalPosition.left = rect.origin.x;
        wp.rcNormalPosition.top = rect.origin.y;
        wp.rcNormalPosition.right = rect.origin.x + rect.size.width;
        wp.rcNormalPosition.bottom = rect.origin.y + rect.size.height;

        if unsafe { winuser::SetWindowPlacement(hwnd.as_mut(), &wp) } == 0 {
            Err(crate::win32_error(crate::Win32Function::SetWindowPlacement))
        } else {
            Ok(())
        }
    }

    /// Enable or unenable this window.
    #[inline]
    pub fn enable(&self, do_display: bool) {
        unsafe { winuser::EnableWindow(self.hwnd().as_mut(), crate::wboolify(do_display)) };
    }

    /// Show the window.
    #[inline]
    pub fn show(&self, cmd_show: CmdShow) {
        unsafe { winuser::ShowWindow(self.hwnd().as_mut(), cmd_show as c_int) };
    }

    /// Update the window.
    #[inline]
    pub fn update(&self) -> crate::Result<()> {
        if unsafe { winuser::UpdateWindow(self.hwnd().as_mut()) } == 0 {
            Err(crate::win32_error(crate::Win32Function::UpdateWindow))
        } else {
            Ok(())
        }
    }

    /// Set the text value of this window.
    #[inline]
    pub fn set_text(&self, text: &WStr) -> crate::Result<()> {
        // note: i've personally tested this in C. You can delete the actual
        // allocated memory if you've already run SetWindowText.
        if unsafe { winuser::SetWindowTextW(self.hwnd().as_mut(), text.as_ptr()) } == 0 {
            Err(crate::win32_error(crate::Win32Function::SetWindowTextW))
        } else {
            Ok(())
        }
    }

    /// Invalid this window and force a redraw.
    #[inline]
    pub fn invalidate(&self, invalidated_rect: Option<Rect<c_int>>) -> crate::Result<()> {
        let rect = invalidated_rect.map(crate::eurect_to_winrect);
        if unsafe {
            winuser::InvalidateRect(
                self.hwnd().as_mut(),
                match rect {
                    Some(ref r) => r,
                    None => ptr::null(),
                },
                TRUE,
            )
        } == 0
        {
            Err(crate::win32_error(crate::Win32Function::InvalidateRect))
        } else {
            Ok(())
        }
    }

    /// Begin painting ops on this window.
    #[inline]
    pub fn begin_paint(&self) -> crate::Result<DeviceContext> {
        DeviceContext::begin_paint(self)
    }
}
