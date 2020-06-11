/* -----------------------------------------------------------------------------------
 * src/error.rs - Common error type to keep things simple.
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

use std::{convert::Infallible, fmt, ptr, string::FromUtf16Error};
use thiserror::Error;
use winapi::{
    shared::minwindef::DWORD,
    um::{errhandlingapi, winbase::*},
};

/// Win32 functions that are capable of erroring out.
#[derive(Debug, Clone, Copy)]
pub enum Win32Function {
    MultiByteToWideChar,
    WideCharToMultiByte,
    GetModuleHandleExW,
    UnregisterClassW,
    RegisterClassExW,
    GetClassInfoExW,
    CreateWindowExW,
    GetWindowPlacement,
    SetWindowPlacement,
    SetWindowTextW,
    InvalidateRect,
    MoveToEx,
    LineTo,
    SetDCBrushColor,
    SetDCPenColor,
    Arc,
    SetArcDirection,
    Rectangle,
    Ellipse,
    ShowWindow,
    UpdateWindow,
    CreateCompatibleBitmap,
    BeginPaint,
    CreateCompatibleDC,
    CreateBitmap,
    GetObjectW,
    Other(&'static str),
}

impl fmt::Display for Win32Function {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match *self {
                Self::MultiByteToWideChar => "MultiByteToWideChar",
                Self::WideCharToMultiByte => "WideCharToMultiByte",
                Self::GetModuleHandleExW => "GetModuleHandleExW",
                Self::UnregisterClassW => "UnregisterClassW",
                Self::RegisterClassExW => "RegisterClassExW",
                Self::GetClassInfoExW => "GetClassInfoExW",
                Self::CreateWindowExW => "CreateWindowExW",
                Self::GetWindowPlacement => "GetWindowPlacement",
                Self::SetWindowPlacement => "SetWindowPlacement",
                Self::SetWindowTextW => "SetWindowTextW",
                Self::InvalidateRect => "InvalidateRect",
                Self::MoveToEx => "MoveToEx",
                Self::LineTo => "LineTo",
                Self::SetDCBrushColor => "SetDCBrushColor",
                Self::SetDCPenColor => "SetDCPenColor",
                Self::Arc => "Arc",
                Self::SetArcDirection => "SetArcDirection",
                Self::Rectangle => "Rectangle",
                Self::Ellipse => "Ellipse",
                Self::ShowWindow => "ShowWindow",
                Self::UpdateWindow => "UpdateWindow",
                Self::CreateCompatibleBitmap => "CreateCompatibleBitmap",
                Self::BeginPaint => "BeginPaint",
                Self::CreateCompatibleDC => "CreateCompatibleDC",
                Self::CreateBitmap => "CreateBitmap",
                Self::GetObjectW => "GetObjectW",
                Self::Other(s) => s,
            }
        )
    }
}

/// The error used by the Porcupine API.
#[derive(Debug, Error)]
pub enum Error {
    #[error("This error should not have been able to possibly occur.")]
    Unreachable,
    #[error("{0}")]
    StaticMsg(&'static str),
    /// A Win32 error occured.
    #[error("A windows error occurred while running {function}: {message} (Code {code})")]
    Win32 {
        code: DWORD,
        message: String,
        function: Win32Function,
    },
    #[error("Unable to convert UTF-16 to Rust string: {0}")]
    Utf16(#[from] FromUtf16Error),

    /// Attempted to create a wide string with a zero in the middle.
    #[error("Attempted to create wide string with a zero.")]
    WideStringNul,
    /// Unable to get widestring slice.
    #[error("Unable to create slice for WideString")]
    LpcwstrFailed,
    /// Attempted to upgrade a dead Weak pointer.
    #[error("Attempted to upgrade a weak pointer that has since expired")]
    ExpiredWeakPtr,
    #[error("Attempted to set storage on a non-owning device context")]
    NoGDIStorage,
    #[error("Attempted to set storage on a device context that already owned storage")]
    AlreadyHadGDIStorage,
}

impl From<Infallible> for Error {
    fn from(_i: Infallible) -> Error {
        Error::Unreachable
    }
}

impl From<Error> for fmt::Error {
    fn from(_f: Error) -> Self {
        Self
    }
}

/// A result, for conveinence.
pub type Result<T> = std::result::Result<T, Error>;

/// Get the last Win32 error, if applicable.
pub fn win32_error(function: Win32Function) -> Error {
    let error = unsafe { errhandlingapi::GetLastError() };

    if error == 0 {
        return Error::Win32 {
            code: error,
            message: "No error detected".to_string(),
            function,
        };
    }

    const ERROR_BUFFER_SIZE: usize = 256;
    let mut error_buffer = Vec::with_capacity(ERROR_BUFFER_SIZE);

    let len = unsafe {
        FormatMessageW(
            FORMAT_MESSAGE_IGNORE_INSERTS
                | FORMAT_MESSAGE_FROM_SYSTEM
                | FORMAT_MESSAGE_ARGUMENT_ARRAY,
            ptr::null(),
            error,
            0,
            error_buffer.as_mut_ptr(),
            (ERROR_BUFFER_SIZE + 1) as DWORD,
            ptr::null_mut(),
        )
    };

    if len == 0 {
        return Error::Win32 {
            code: error,
            message: "No error message detected".to_string(),
            function,
        };
    }

    unsafe { error_buffer.set_len(len as usize) };

    match {
        match crate::WStr::from_bytes(&error_buffer) {
            Ok(b) => b,
            Err(e) => return e,
        }
    }
    .into_string()
    {
        Ok(s) => Error::Win32 {
            code: error,
            message: s,
            function,
        },
        Err(e) => e,
    }
}