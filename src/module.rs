/* -----------------------------------------------------------------------------------
 * src/module.rs - Module-specific info that should only need to be called once.
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

use crate::mutexes::Mutex;
use core::{
    fmt,
    ptr::{self, NonNull},
    sync::atomic::AtomicPtr,
};
use winapi::{
    shared::minwindef::{HINSTANCE__, HMODULE},
    um::libloaderapi,
};

/// Module-specific information.
pub struct ModuleInfo {
    handle: AtomicPtr<HINSTANCE__>,
}

impl fmt::Debug for ModuleInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Win32 Module")
    }
}

impl ModuleInfo {
    /// Create a new instance of module information.
    #[inline]
    pub fn new() -> crate::Result<ModuleInfo> {
        // get the module handle
        let mut handle: HMODULE = ptr::null_mut();

        if unsafe { libloaderapi::GetModuleHandleExA(0, ptr::null(), &mut handle) } == 0 {
            Err(crate::win32_error(crate::Win32Function::GetModuleHandleExA))
        } else {
            debug_assert!(!handle.is_null());
            Ok(Self {
                handle: AtomicPtr::new(handle),
            })
        }
    }

    /// Get the handle to the module.
    #[inline]
    pub fn handle(&mut self) -> NonNull<HINSTANCE__> {
        unsafe { NonNull::new_unchecked(*self.handle.get_mut()) }
    }
}

impl Drop for ModuleInfo {
    fn drop(&mut self) {
        unsafe { libloaderapi::FreeLibrary(*self.handle.get_mut()) };
    }
}

lazy_static::lazy_static! {
    pub static ref MODULE_INFO: Mutex<ModuleInfo> =
        Mutex::new(ModuleInfo::new().expect("Unable to create module info"));
}
