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

use fixed::{traits::Fixed, types::extra::U15, FixedI32};
use serde_json::{Map, Value};
use core::panic;
use std::fmt;

// Custom format for displaying ASCII or Hexadecimal representation
pub struct Utf8Lossy<'a>(pub &'a [u8]);

impl<'a> fmt::Display for Utf8Lossy<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for &byte in self.0 {
            if byte.is_ascii() {
                // Display ASCII characters as they are
                write!(f, "{}", byte as char)?;
            } else {
                // Display non-ASCII characters as hexadecimal
                write!(f, "0x{:02X}", byte)?;
            }
        }
        Ok(())
    }
}

// Scaling factor for using the fixed point representation
const FIXED_POINT_SCALING_FACTOR_Q_32_15: f32 = 32768.0; // 2^15
const FIXED_POINT_INTEGER_PART_MASK_Q_32_15: u32 = 0xFFFF_8000;
const FIXED_POINT_FRACTIONAL_PART_MASK_Q_32_15: u32 = 0x0000_7FFF;
const VARIABLE_LENGTH: usize = 4;
const NUMBER_OF_VARIABLES: usize = 8;
const ENDING_PART_LENGTH_BYTES: usize = 2;

pub fn get_length_of_message_data(data: &Vec<u8>) -> u32 {
    let data_message_length: Vec<u8> = data[0..VARIABLE_LENGTH].iter().copied().rev().collect();


//    println!("message obtained before obtaining length: {:#?}", data);
//    println!(
//        "message data length from message : {:#?}",
//        data_message_length
//    );

    let string = format!("{}", Utf8Lossy(&data_message_length));
//    println!("FORMATED MSG LENGTH {:#?}", string);

    let received_hex_from_chunk: u32 = get_hex_from_chunk(&data_message_length).try_into().unwrap();

//    println!("length in u32 {:#?}", received_hex_from_chunk);
//    if received_hex_from_chunk > 8
//    {
//        panic!("is higher");
//    }
    if(received_hex_from_chunk < 256)
    {

    return received_hex_from_chunk
    }
    else {
        return 256
    }
}
// Function for obtaining the inner part of the message
// the message structure
// | start part | message data part | CRC ID | CRC value | end part |
// | variable length | variable length * number of variables | variable length edited for CRC | variable length CRC (CRC = 8 bits and the rest are nulls) | varible length |
// length of the variable, which means in bits length * 8, so when the variable is 32 bits long,
// the length of the variable is 4 bytes
pub fn get_message_inner_part(data: &Vec<u8>) -> Vec<u8> {
    let start_part_length = 4; // How many bytes are there in the start message part, use to strip
                               // the start part from data based on the index which starts from 0,
                               // the start part length is dependent on the data structure passed
                               // in the function

    // It should be checked by preceeding message manipulation which should be correct because the
    // CRC is checked and CRC does not match when the variable is missing
    if data.len() >= VARIABLE_LENGTH * ENDING_PART_LENGTH_BYTES {
        let ending_part_length = data.len() - 4 * 2; // 4 bytes of a stop word is stripped when receiving the data
                                                     // because of the structure of used vecs and moving frames, 4 bytes of CRC, 4 bytes of @CRC
                                                     // indicator
        let message_data: Vec<u8> = Vec::from(&data[start_part_length..ending_part_length]);
        return message_data;
    } else {
        let message_data: Vec<u8> = vec![];
        return message_data;
    }
}

// Function for obtaining the JSON strucured data of the message inner part
pub fn get_structured_message_data(data: &Vec<u8>) -> Value {
    // Length of the variable data in bytes, it is the same length as length of variable id
    // so together the length of analyzed chunk is variable_length * 2
    let variable_length = 4;
    let mut variable_ids: Vec<String> = Vec::new();
    let mut variable_data: Vec<f32> = Vec::new();
    let mut message_json: Value = Value::Object(Default::default());
    let message_data: Vec<u8> = get_message_inner_part(&data);

    //    println!(
    //        "Message data {:#?} and len {}",
    //        message_data,
    //        message_data.len()
    //    );

    // Obtaining data from message data vector which contains inner part of the message with
    // variable IDs and variable data
    // the chunks are 8 in length, because the variable length is 4, so length of variable ID
    // variable data is 4+4
    // need to implement some setting when the length is something else
    // if message_data.len() == (NUMBER_OF_VARIABLES * VARIABLE_LENGTH)
    //   if message_data.len()/(2 * VARIABLE_LENGTH) == 0   // checking if the chunks can be made
    if true
    // if it is dividable without any remainder
    {
        for chunk in message_data.chunks_exact(ENDING_PART_LENGTH_BYTES * VARIABLE_LENGTH) {
            // chunks_exact should perform
            // that it takes exactly the
            // required number of chunks
            // Checking ig the length is truly the right amount as specified in chunks
            //if chunk.len() <= 2 * variable_length { // think about it....
            if true {
                // Specify where split the chunks, because the length of the variable ID and variable
                // data is the same in this version of the application, it is splitted in the middle
                let (var_id, var_data) = chunk.split_at(variable_length);

                // The UART sends LSB first, co the buffer is filled from the LSB to MSB so after the
                // whole variable is received, the buffer for one variable sent from FPGA via uart in
                // fact looks like | LSB_fpga .... MSB_fpga | so in order to see the variable as is in
                // the register in the FPGA, the order of bits / bytes must be reversed
                // the message is left intact, only the when processing the data for obtaining the
                // structured data, the bits are reversed using following statemens
                // so in the reversed var_data_reverse and var_id_reverse the variable looks like
                // | MSB_fpga ... LSB_fpga |
                let var_data_reverse: Vec<u8> = var_data.iter().copied().rev().collect();
                let var_id_reverse: Vec<u8> = var_id.iter().copied().rev().collect();
                //            println!("Reversed data chunk: {:X?}", var_data_reverse);
                //            println!("Reversed id chunk: {:X?}", var_id_reverse);
                // Print currently analyzed var_id and var_data in the current chunk
                //            println!("var_id: {:X?}", var_id);
                //            println!("var_data: {:X?}", var_data);

                // let var_id_hex: String = hex::encode(var_id);
                //           let var_id_ascii = match std::str::from_utf8(var_id){
                //               Ok(s) => s,
                //                Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
                //           };
                //           println!("var_id_ascii {:#?}", var_id_ascii);

                // Converting var_id to ASCII representation with custom struct to be able to convert
                // even non utf8 signs, but the UTF signs are used, so it it only for convinience,
                // otherwise the match statement with std::str::from_utf8 would be used
                let var_id_ascii = format!("{}", Utf8Lossy(&var_id_reverse));

                // Pushing the var_id in ascii to the vector, which will be later used for generating
                // the json object
                variable_ids.push(var_id_ascii.clone());
                // Debug output of the ascii id
                //            println!("string_test {:#?}", var_id_ascii);

                // Debug output of the hexadecimal representation of the variabled data
                //            println!("var_data_hex_value. {:X?}", get_hex_from_chunk(&var_data));

                // Converting the variable data to a fixed point representation based on the specified
                // format, in the future it must be implemeted some kind of automat for selecting the
                // representation

                // Convert the hex value to a signed integer

                let received_hex_from_chunk: u32 =
                    get_hex_from_chunk(&var_data_reverse).try_into().unwrap();

                //            println!("Received hex from chunk {:X?}", received_hex_from_chunk);
                // Interpret the integer and fractional parts
                let integer_part = ((received_hex_from_chunk
                    & FIXED_POINT_INTEGER_PART_MASK_Q_32_15)
                    as i32
                    >> 15) as f32; // Sign extension and integer part
                let fractional_part =
                    ((received_hex_from_chunk & FIXED_POINT_FRACTIONAL_PART_MASK_Q_32_15) as f32)
                        / FIXED_POINT_SCALING_FACTOR_Q_32_15; // Fractional part

                // Combine integer and fractional parts
                let var_data_from_fixed = integer_part + fractional_part;

                // Library not used
                //            println!("Fixed-point value: {}", fixed_point_value);
                //
                //            println!("Received hex from chunk {:X?}", received_hex_from_chunk);
                //            let mut var_data_from_fixed =
                //                FixedI32::<U15>::wrapping_from_num(received_hex_from_chunk);

                //            println!(
                //                "from fixed to float from vardata {:.5}",
                //                var_data_from_fixed
                //            );
                // variable_data.push(get_hex_from_chunk(&var_data));
                variable_data.push(var_data_from_fixed);
            } else {
                // Handle chunks with less than 8 elements (if needed)
                //            println!("Chunk has less than 8 elements");
            }

            // Obtained vectors of variable IDs and variable data so printing it out as for debugging
            //        println!("variable ids: {:#?}", variable_ids);
            //        println!("variable data: {:#?}", variable_data);

            // Creating Map object to construct a structured JSON data object for transmitting the data
            let mut message_map = Map::new();

            // Iterating through variable_ids and variable_data together, where id is a key and data is
            // value
            // both vectors have the same length
            for (key, value) in variable_ids.iter().zip(variable_data.iter()) {
                // Converting the fixed point value to a string to be parsed as a float
                let value_in_string = value.to_string();
                // Parsing the String formt to the float format for values
                let value_in_float = match value_in_string.parse::<f64>() {
                    Ok(v) => v,
                    Err(e) => {
                        println!("Error parsing the String to a float value. e: {:#?}", e);
                        0.0
                    }
                };
                // Constructing the Map object which is in fact a JSON object to be used when
                // transmitting the data
                message_map.insert(
                    key.to_string(),
                    Value::Number(serde_json::Number::from_f64(value_in_float).unwrap()),
                );
            }

            // Final serde_json object with key value pairs of
            // var_id:var_data
            // where var_id is String
            // where var_data is Number
            // but for the Number to be a float number and not a decimal, when using the fixed point
            // representation, the Fixed point representation from a crate fixed must be converted to
            // String representation and then parsed to the f64
            message_json = Value::Object(message_map);

            //println!("message json obtained: {:#?}", message_json);
        }
    }

    return message_json;
}

pub fn get_hex_from_chunk(data: &[u8]) -> u32 {
    // Join bytes to one string and convert to hex in a string represebration
    let data_string: String = data.iter().map(|&b| format!("{:02X}", b)).collect();
    //println!("joined var_data in chunk loop {:X?}", data_string);

    // Converting to direct u32 representation
    match u32::from_str_radix(&data_string, 16) {
        Ok(hex_value) => {
            // println!("Succesfully converted hex value 0x{:X?}", hex_value);
            // println!("Succesfully converted hex value decimal representation {:#?}", hex_value);
            return hex_value;
        }
        Err(e) => {
            println!("Encountered error converting the value to hex. {:#?}", e);
            return 0;
        }
    }
    //println!("hex var_id {:X?}", var_id_hex);
}

// Function for checking the CRC of message
// the CRC check only the data part - inner part if the message
// but the whole message with start, data, crc, ending part is passed and the corresponding
// inner data part is innerly obtained
// data is obtained from the website https://crccalc.com/
pub fn crc_check(data: &Vec<u8>, crc_table: &Vec<u8>) -> bool {
    let variable_length = 4; // the variable is 4 bytes long = 32 bits

    //    let crc_table: Vec<u8> = Vec::from([
    //        0x00, 0x07, 0x0e, 0x09, 0x1c, 0x1b, 0x12, 0x15, 0x38, 0x3f, 0x36, 0x31, 0x24, 0x23, 0x2a,
    //        0x2d, 0x70, 0x77, 0x7e, 0x79, 0x6c, 0x6b, 0x62, 0x65, 0x48, 0x4f, 0x46, 0x41, 0x54, 0x53,
    //        0x5a, 0x5d, 0xe0, 0xe7, 0xee, 0xe9, 0xfc, 0xfb, 0xf2, 0xf5, 0xd8, 0xdf, 0xd6, 0xd1, 0xc4,
    //        0xc3, 0xca, 0xcd, 0x90, 0x97, 0x9e, 0x99, 0x8c, 0x8b, 0x82, 0x85, 0xa8, 0xaf, 0xa6, 0xa1,
    //        0xb4, 0xb3, 0xba, 0xbd, 0xc7, 0xc0, 0xc9, 0xce, 0xdb, 0xdc, 0xd5, 0xd2, 0xff, 0xf8, 0xf1,
    //        0xf6, 0xe3, 0xe4, 0xed, 0xea, 0xb7, 0xb0, 0xb9, 0xbe, 0xab, 0xac, 0xa5, 0xa2, 0x8f, 0x88,
    //        0x81, 0x86, 0x93, 0x94, 0x9d, 0x9a, 0x27, 0x20, 0x29, 0x2e, 0x3b, 0x3c, 0x35, 0x32, 0x1f,
    //        0x18, 0x11, 0x16, 0x03, 0x04, 0x0d, 0x0a, 0x57, 0x50, 0x59, 0x5e, 0x4b, 0x4c, 0x45, 0x42,
    //        0x6f, 0x68, 0x61, 0x66, 0x73, 0x74, 0x7d, 0x7a, 0x89, 0x8e, 0x87, 0x80, 0x95, 0x92, 0x9b,
    //        0x9c, 0xb1, 0xb6, 0xbf, 0xb8, 0xad, 0xaa, 0xa3, 0xa4, 0xf9, 0xfe, 0xf7, 0xf0, 0xe5, 0xe2,
    //        0xeb, 0xec, 0xc1, 0xc6, 0xcf, 0xc8, 0xdd, 0xda, 0xd3, 0xd4, 0x69, 0x6e, 0x67, 0x60, 0x75,
    //        0x72, 0x7b, 0x7c, 0x51, 0x56, 0x5f, 0x58, 0x4d, 0x4a, 0x43, 0x44, 0x19, 0x1e, 0x17, 0x10,
    //        0x05, 0x02, 0x0b, 0x0c, 0x21, 0x26, 0x2f, 0x28, 0x3d, 0x3a, 0x33, 0x34, 0x4e, 0x49, 0x40,
    //        0x47, 0x52, 0x55, 0x5c, 0x5b, 0x76, 0x71, 0x78, 0x7f, 0x6a, 0x6d, 0x64, 0x63, 0x3e, 0x39,
    //        0x30, 0x37, 0x22, 0x25, 0x2c, 0x2b, 0x06, 0x01, 0x08, 0x0f, 0x1a, 0x1d, 0x14, 0x13, 0xae,
    //        0xa9, 0xa0, 0xa7, 0xb2, 0xb5, 0xbc, 0xbb, 0x96, 0x91, 0x98, 0x9f, 0x8a, 0x8d, 0x84, 0x83,
    //        0xde, 0xd9, 0xd0, 0xd7, 0xc2, 0xc5, 0xcc, 0xcb, 0xe6, 0xe1, 0xe8, 0xef, 0xfa, 0xfd, 0xf4,
    //        0xf3,
    //    ]);

    if (data.len() > VARIABLE_LENGTH) {
        let message_data_crc_index = data.len() - 1 * variable_length; // is 8th index from the end, when using the

        // notation that the CRC is 32 bit and send as first
        // and then just zeros - example: 0xE3000000;
        // but when the data is
        // | start | data | crcIndicator
        // | CRC | stop | the CRC is at
        // the 4th index from the end

        let message_data: Vec<u8> = get_message_inner_part(&data);
        let message_data_crc: &u8 = &data[message_data_crc_index];
        //    println!("message data crc: {:X?}", message_data_crc);
        //    println!("stripped data {:X?}", message_data);
        //    println!("message data {:X?}", &data);

        // Initial CRC index with which the checking starts / and encoding on the FPGA has to start
        // with the same index
        let mut crc_index = 0x00;

        for byte in message_data {
            crc_index = crc_index ^ byte;
            crc_index = crc_table[usize::from(crc_index)];
            //        println!("CRC INDEX: {:X?}", crc_index);
        }
        //println!("CRC INDEX: {:X?}", crc_index);

        if &crc_index == message_data_crc {
//                    println!(
//                        "CRC does match, data CRC: {:X?}, calculated CRC: {:X?}",
//                        message_data_crc, crc_index
//                    );
            true
        } else {
//            println!(
//                "CRC does not match, data CRC: {:X?}, calculated CRC: {:X?}",
//                message_data_crc, crc_index
//            );
            false
        }
    } else {
        false
    }
}
