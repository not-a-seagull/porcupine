/* -----------------------------------------------------------------------------------
 * porcupine_constant_wide/src/lib.rs - Convert string literal to wide bytes.
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

use proc_macro::{TokenStream, TokenTree};

#[proc_macro]
pub fn constant_text(tin: TokenStream) -> TokenStream {
    let literal = tin
        .into_iter()
        .next()
        .expect("Unable to get first token of stream");

    let literal = match literal {
        TokenTree::Literal(l) => l,
        _ => panic!("First element is not a string token"),
    };

    let literal = literal.to_string();
    let literal = literal.split('"').nth(1).unwrap();

    // convert to bytes
    let mut parts = Vec::with_capacity(literal.len() + 2);
    parts.push("&WStr::from_bytes_unchecked(&[".to_string());

    let eutf16 = literal.encode_utf16().collect::<Vec<u16>>();
    let len = eutf16.len();
    eutf16
        .into_iter()
        .enumerate()
        .for_each(|(i, s)| parts.push(format!("{}{}", s, if i != len - 1 { "," } else { "" })));

    parts.push("])".to_string());

    // parse and return
    let res = parts.join("");
    res.parse().unwrap()
}
