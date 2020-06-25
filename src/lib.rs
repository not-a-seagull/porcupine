/* -----------------------------------------------------------------------------------
 * src/lib.rs - Root of the Porcupine library.
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

#![cfg(windows)]

//! This is intended to be a safe Rust wrapper around the Win32 API, with an emphasis on the graphical WinUser
//! part of the API.

pub mod bitmap;
pub mod commctrl;
pub mod dc;
mod error;
pub mod module;
pub mod window;

pub use bitmap::*;
pub use commctrl::*;
pub use dc::*;
pub use error::*;
pub use module::*;
pub use window::*;

/// Utility function to convert Rust bool to Win32 BOOL
#[inline]
pub fn wboolify(rbool: bool) -> winapi::shared::minwindef::BOOL {
    use winapi::shared::minwindef::{FALSE, TRUE};
    if rbool {
        TRUE
    } else {
        FALSE
    }
}

/// Utility function to convert a Euclid rect to a Windows rect.
#[inline]
pub fn eurect_to_winrect(
    eurect: euclid::default::Rect<std::os::raw::c_int>,
) -> winapi::shared::windef::RECT {
    winapi::shared::windef::RECT {
        left: eurect.origin.x,
        top: eurect.origin.y,
        right: eurect.origin.x + eurect.size.width,
        bottom: eurect.origin.y + eurect.size.height,
    }
}
