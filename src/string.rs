/* -----------------------------------------------------------------------------------
 * src/string.rs - Handle some of the tricky string semantics in Win32.
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

use std::{borrow::Borrow, boxed::Box, convert::TryFrom, fmt, mem, ops, ptr, slice};
use winapi::shared::ntdef::WCHAR;

/// A slice of a wide, UTF-16 string.
#[repr(transparent)]
pub struct WStr([WCHAR]);

// helper function: length of a wide string
unsafe fn wide_strlen(raw: *const WCHAR) -> usize {
    // offset the pointer until a zero is encountered
    let mut p = raw;
    let mut c = 0;

    while *p != 0 {
        p = p.offset(1);
        c += 1;
    }

    c
}

/// A container for a wide, UTF-16 string.
#[derive(Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct WString {
    inner: Box<[WCHAR]>,
}

impl WString {
    /// Create a new WString from a vector of wide chars, without checking for zeroes.
    #[inline]
    pub unsafe fn from_vec_unchecked(mut v: Vec<WCHAR>) -> Self {
        v.reserve_exact(1);
        v.push(0);
        Self {
            inner: v.into_boxed_slice(),
        }
    }

    /// Create a new WString.
    pub fn new<T: Into<Vec<WCHAR>>>(t: T) -> Result<Self, crate::Error> {
        // some traits for optimizations
        /*trait IntoWideVec {
            fn into_wide_vec(self) -> Vec<WCHAR>;
        }

        impl<T: Into<Vec<WCHAR>>> IntoWideVec for T {
            default fn into_wide_vec(self) -> Vec<WCHAR> {
                self.into()
            }
        }

        // this should help avoid reallocations
        impl<'a> IntoWideVec for &'a [WCHAR] {
            fn into_wide_vec(self) -> Vec<WCHAR> {
                let mut v = Vec::with_capacity(self.len() + 1);
                v.extend(self);
                v
            }
        }

        impl<'a> IntoWideVec for &'a str {
            fn into_wide_vec(self) -> Vec<WCHAR> {
                let mut v = Vec::with_capacity(self.len() + 1);
                v.extend(self.encode_utf16());
                v
            }
        }

        // convert to a vector
        let vec = IntoWideVec::into_wide_vec(t);*/

        let vec: Vec<WCHAR> = t.into();

        // check if there is a zero in the string
        // search from the back since that's where zeroes tend to be
        if vec.iter().rev().any(|x| *x == 0) {
            Err(crate::Error::WideStringNul)
        } else {
            Ok(unsafe { Self::from_vec_unchecked(vec) })
        }
    }

    /// Creates a buffer for a wide string.
    #[inline]
    pub fn buffer(len: usize) -> Self {
        let mut v = Vec::with_capacity(len);
        v.extend([32].iter().cycle().take(len)); // note: 32 is UTF-16 for ' '
        unsafe { Self::from_vec_unchecked(v) }
    }

    /// Creates this item from a pointer to a wide string. This should only be used if
    /// the pointer was created by the into_raw() function. Otherwise, shenanigans may occur.
    #[inline]
    pub unsafe fn from_raw(ptr: *mut WCHAR) -> Self {
        // we need a fat pointer to cast from
        let len = wide_strlen(ptr);
        let slice = slice::from_raw_parts_mut(ptr, len);

        Self {
            inner: Box::from_raw(slice as *mut [WCHAR]),
        }
    }

    /// Convert this item into a raw wide string.
    #[inline]
    pub fn into_raw(self) -> *mut WCHAR {
        Box::into_raw(self.into_boxed_slice()) as *mut WCHAR
    }

    /// Convert this item into a boxed slice containing the data.
    #[inline]
    pub fn into_boxed_slice(self) -> Box<[WCHAR]> {
        let b = mem::ManuallyDrop::new(self);
        unsafe { ptr::read(&b.inner) }
    }

    /// Convert this item into a vector containing bytes, including the null byte.
    #[inline]
    pub fn into_bytes(self) -> Vec<WCHAR> {
        self.into_boxed_slice().into_vec()
    }

    /// Convert this item into a vector containing bytes, excluding the null byte.
    #[inline]
    pub fn into_bytes_no_nul(self) -> Vec<WCHAR> {
        let mut v = self.into_bytes();
        let _nul = v.pop();
        debug_assert_eq!(_nul, Some(0u16));
        v
    }

    /// Get the bytes, including the null byte.
    #[inline]
    pub fn as_bytes(&self) -> &[WCHAR] {
        &self.inner
    }

    /// Get the bytes, excluding the null byte.
    #[inline]
    pub fn as_bytes_no_nul(&self) -> &[WCHAR] {
        &self.inner[..self.inner.len() - 1]
    }
}

impl Drop for WString {
    fn drop(&mut self) {
        // convert this string to a zero string
        unsafe { *self.inner.get_unchecked_mut(0) = 0 };
    }
}

impl fmt::Debug for WString {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}

impl ops::Deref for WString {
    type Target = WStr;

    #[inline]
    fn deref(&self) -> &WStr {
        unsafe { WStr::from_bytes_unchecked(self.as_bytes()) }
    }
}

impl From<WString> for Vec<u16> {
    fn from(w: WString) -> Self {
        w.into_bytes_no_nul()
    }
}

impl Default for &WStr {
    fn default() -> Self {
        const EMPTY: &'static [WCHAR] = &[0];
        unsafe { WStr::from_ptr(EMPTY.as_ptr()) }
    }
}

impl Default for WString {
    fn default() -> Self {
        let c: &WStr = Default::default();
        c.to_owned()
    }
}

impl From<&WStr> for Box<WStr> {
    fn from(w: &WStr) -> Box<WStr> {
        let boxed: Box<[WCHAR]> = Box::from(w.to_bytes());
        unsafe { Box::from_raw(Box::into_raw(boxed) as *mut WStr) }
    }
}

impl Borrow<WStr> for WString {
    #[inline]
    fn borrow(&self) -> &WStr {
        self
    }
}

impl fmt::Debug for WStr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.into_string()?)
    }
}

impl TryFrom<String> for WString {
    type Error = crate::Error;

    fn try_from(s: String) -> crate::Result<Self> {
        Self::new(s.encode_utf16().collect::<Vec<u16>>())
    }
}

impl<'a> TryFrom<&'a str> for WString {
    type Error = crate::Error;

    fn try_from(s: &'a str) -> crate::Result<Self> {
        Self::new(s.encode_utf16().collect::<Vec<u16>>())
    }
}

impl Clone for Box<WStr> {
    #[inline]
    fn clone(&self) -> Self {
        (**self).into()
    }
}

impl WStr {
    /// Convert a wide character pointer to a wide string.
    pub unsafe fn from_ptr<'a>(ptr: *const WCHAR) -> &'a WStr {
        let len = wide_strlen(ptr);
        WStr::from_bytes_unchecked(slice::from_raw_parts(ptr, len))
    }

    /// Convert a wide character slice to a wide string.
    #[inline]
    pub const unsafe fn from_bytes_unchecked(bytes: &[WCHAR]) -> &WStr {
        &*(bytes as *const [WCHAR] as *const WStr)
    }

    /// Convert a wide character slice to a wide string, checking if the last byte is null.
    pub fn from_bytes(bytes: &[WCHAR]) -> crate::Result<&WStr> {
        if bytes.iter().rev().any(|x| *x == 0) {
            Err(crate::Error::WideStringNul)
        } else {
            Ok(unsafe { WStr::from_bytes_unchecked(bytes) })
        }
    }

    /// Convert to a pointer.
    pub const fn as_ptr(&self) -> *const WCHAR {
        self.0.as_ptr()
    }

    /// Get the bytes belonging to this string.
    pub fn to_bytes(&self) -> &[WCHAR] {
        unsafe { &*(&self.0 as *const [WCHAR]) }
    }

    /// Get the bytes belonging to this string, excluding the null byte.
    pub fn to_bytes_no_nul(&self) -> &[WCHAR] {
        let bytes = self.to_bytes();
        &bytes[..bytes.len() - 1]
    }

    /// Convert to a Rust string.
    pub fn into_string_lossy(&self) -> String {
        String::from_utf16_lossy(self.to_bytes_no_nul())
    }

    /// Convert to a Rust string, throwing an error if it cannot be encoded.
    pub fn into_string(&self) -> crate::Result<String> {
        Ok(String::from_utf16(self.to_bytes_no_nul())?)
    }
}

impl ToOwned for WStr {
    type Owned = WString;

    fn to_owned(&self) -> WString {
        WString {
            inner: self.to_bytes().into(),
        }
    }
}
