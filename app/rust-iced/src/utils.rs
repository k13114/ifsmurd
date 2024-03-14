/*

MIT License

Copyright (c) 2024 Petr Zakopal, Deparment of Electric Drives and Traction, CTU FEE

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
*/

/*----------------------------------------------------------------------------*/

use std::{fs::File, io::Write};


// Function used for writing formatted_string to a file specified as a prop
pub fn write_output_to_file(output_file: &mut File, formatted_string: String) {
    match output_file.write_all(formatted_string.as_ref()) {
        Ok(v) => {
            println!("Ok writing a file for index, {:#?}", v);
        }
        Err(e) => {
            println!("Error writing a file {:#?}", e);
        }
    }
}
