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

use std::fs::File;
use std::io::Write;

use plotters_iced::sample::lttb::LttbSource;
use serde_json::Value;
use serialport::SerialPort;
use tokio::sync::broadcast;
use tokio::sync::watch;

use crate::com_core::message::get_hex_from_chunk;
use crate::com_core::message::get_length_of_message_data;
use crate::com_core::message::{crc_check, get_structured_message_data, Utf8Lossy};

const VARIABLE_LENGTH_BYTES: usize = 4;

pub async fn serial_port_task(
    mut rx: watch::Receiver<bool>,
    mut serial_port: Box<dyn SerialPort>,
    tx_data_bf_broadcast_channel: broadcast::Sender<Value>,
) {
    //let mut test_file = File::create("./../test-file.txt").unwrap();

    let mut data_from_serial_port: Vec<u8> = Vec::new();
    data_from_serial_port.resize(VARIABLE_LENGTH_BYTES * 256 * 100, 0);

    // will need to move to another channel which will be used to send notifications BF
    //    tx_data_bf_broadcast_channel
    //        .send("Started serial...".into())
    //        .expect("Error serde");
    let mut is_start_byte: bool = false;
    let mut is_stop_byte: bool = false;
    let mut message_hex: Vec<u8> = Vec::new();
    let mut message_hex_temp: Vec<u8> = Vec::new();
    let mut data_read: bool = false;

    let crc_table: Vec<u8> = Vec::from([
        0x00, 0x07, 0x0e, 0x09, 0x1c, 0x1b, 0x12, 0x15, 0x38, 0x3f, 0x36, 0x31, 0x24, 0x23, 0x2a,
        0x2d, 0x70, 0x77, 0x7e, 0x79, 0x6c, 0x6b, 0x62, 0x65, 0x48, 0x4f, 0x46, 0x41, 0x54, 0x53,
        0x5a, 0x5d, 0xe0, 0xe7, 0xee, 0xe9, 0xfc, 0xfb, 0xf2, 0xf5, 0xd8, 0xdf, 0xd6, 0xd1, 0xc4,
        0xc3, 0xca, 0xcd, 0x90, 0x97, 0x9e, 0x99, 0x8c, 0x8b, 0x82, 0x85, 0xa8, 0xaf, 0xa6, 0xa1,
        0xb4, 0xb3, 0xba, 0xbd, 0xc7, 0xc0, 0xc9, 0xce, 0xdb, 0xdc, 0xd5, 0xd2, 0xff, 0xf8, 0xf1,
        0xf6, 0xe3, 0xe4, 0xed, 0xea, 0xb7, 0xb0, 0xb9, 0xbe, 0xab, 0xac, 0xa5, 0xa2, 0x8f, 0x88,
        0x81, 0x86, 0x93, 0x94, 0x9d, 0x9a, 0x27, 0x20, 0x29, 0x2e, 0x3b, 0x3c, 0x35, 0x32, 0x1f,
        0x18, 0x11, 0x16, 0x03, 0x04, 0x0d, 0x0a, 0x57, 0x50, 0x59, 0x5e, 0x4b, 0x4c, 0x45, 0x42,
        0x6f, 0x68, 0x61, 0x66, 0x73, 0x74, 0x7d, 0x7a, 0x89, 0x8e, 0x87, 0x80, 0x95, 0x92, 0x9b,
        0x9c, 0xb1, 0xb6, 0xbf, 0xb8, 0xad, 0xaa, 0xa3, 0xa4, 0xf9, 0xfe, 0xf7, 0xf0, 0xe5, 0xe2,
        0xeb, 0xec, 0xc1, 0xc6, 0xcf, 0xc8, 0xdd, 0xda, 0xd3, 0xd4, 0x69, 0x6e, 0x67, 0x60, 0x75,
        0x72, 0x7b, 0x7c, 0x51, 0x56, 0x5f, 0x58, 0x4d, 0x4a, 0x43, 0x44, 0x19, 0x1e, 0x17, 0x10,
        0x05, 0x02, 0x0b, 0x0c, 0x21, 0x26, 0x2f, 0x28, 0x3d, 0x3a, 0x33, 0x34, 0x4e, 0x49, 0x40,
        0x47, 0x52, 0x55, 0x5c, 0x5b, 0x76, 0x71, 0x78, 0x7f, 0x6a, 0x6d, 0x64, 0x63, 0x3e, 0x39,
        0x30, 0x37, 0x22, 0x25, 0x2c, 0x2b, 0x06, 0x01, 0x08, 0x0f, 0x1a, 0x1d, 0x14, 0x13, 0xae,
        0xa9, 0xa0, 0xa7, 0xb2, 0xb5, 0xbc, 0xbb, 0x96, 0x91, 0x98, 0x9f, 0x8a, 0x8d, 0x84, 0x83,
        0xde, 0xd9, 0xd0, 0xd7, 0xc2, 0xc5, 0xcc, 0xcb, 0xe6, 0xe1, 0xe8, 0xef, 0xfa, 0xfd, 0xf4,
        0xf3,
    ]);

    let mut message_hex_temp_window: Vec<u8> = Vec::new();
    // Waiting for the appropriate value to be received to start executing the serial port
    // receiving mechansm
    while rx.wait_for(|val| *val == true).await.is_ok() {
        //serialport.read_to_end(&mut data_from_serial_port);

        // Clearing the main buffer to which the data from serial port system buffer is read
        data_from_serial_port.clear();
        data_from_serial_port.resize(256 * 8 * VARIABLE_LENGTH_BYTES, 0);

        // Reading exact number of bytes to fill the buffer data_from_serial_port
        match serial_port.read_exact(data_from_serial_port.as_mut_slice()) {
            Ok(_t) => {
                //                println!("values read: {:?}", t);
                data_read = true;
                //data_from_serial_port.resize(data_from_serial_port.len() + 1, 0);
            }
            Err(e) => {
                println!("Error reading from a serial port {:#?}", e);
            }
        }

        // If the data was successfully read from the serial port buffer
        if data_read == true {
            // Clearing and resizing the vec to which the internal part data - after start and
            // before stop part is passed
            message_hex_temp.clear();
            message_hex_temp.resize(0, 0);
            // Clearing and resizing the vec which acts like a moving window and check for start
            // and stop byte
            message_hex_temp_window.clear();
            message_hex_temp_window.resize(0, 0);

            // Loop for taking the data from the vec which comprises of all read data from the
            // serial port
            for byte in data_from_serial_port.clone() {
                // If the moving window is long as a one variable length in bytes - so there should
                // start part obtained
                // this efectivelly creates a moving window of variable length which scans data
                if message_hex_temp_window.len() == VARIABLE_LENGTH_BYTES {
                    message_hex_temp_window.clear();
                    message_hex_temp_window.resize(0, 0);
                }
                //println!("hex temp: {:X?}", message_hex_temp);

                // Pushing data to a window for analyzing the start and stop sequnce
                message_hex_temp_window.push(byte);

                if !is_start_byte {
                    if message_hex_temp_window.len() == VARIABLE_LENGTH_BYTES {
                        let received_hex_from_chunk: u32 =
                            get_hex_from_chunk(&message_hex_temp_window)
                                .try_into()
                                .unwrap();
                        //                        if message_hex_temp_window
                        //                            .iter()
                        //                            .filter(|&&x| x == 0x2F)
                        //                            .count()
                        //                            == 4
                        if received_hex_from_chunk == 0x2F2F2F2F as u32 {
                            //                    println!("start byte from comparing");
                            //  println!("hex temp: {:X?}", message_hex_temp);
                            //                        println!("Contains start data.");
                            is_start_byte = true;
                            is_stop_byte = false;
                        } else {
                            // Clear the window and shrink to fit to size 0 to be able to receive
                            // and analyze the stop sequence
                            message_hex_temp_window.clear();
                            message_hex_temp_window.shrink_to_fit();

                            is_start_byte = false;
                            is_stop_byte = false;
                        }
                    }
                }

                // When the start Byte is read, and the message contains a stop byte, stop receiving
                // the data and enable another start byte to be read
                if is_start_byte == true {
                    message_hex_temp.push(byte);
                    // Filtres the stop byte an the vec

                    if message_hex_temp_window.len() == VARIABLE_LENGTH_BYTES {
                        let received_hex_from_chunk: u32 =
                            get_hex_from_chunk(&message_hex_temp_window)
                                .try_into()
                                .unwrap();
                        if received_hex_from_chunk == 0x5C5C5C5C as u32 {
                            //                            println!("stop byte from comparing");
                        } else {
                            //                            println!("NOT stop byte from comparing");
                        }
                        if received_hex_from_chunk == 0x5C5C5C5C as u32 {
                            //                                 println!("hex temp: {:X?}", message_hex_temp);
                            // removing preceeding part of the start part
                            message_hex_temp.remove(0);
                            // removing end part
                            if message_hex_temp.len() > 2 * VARIABLE_LENGTH_BYTES {
                                is_stop_byte = true;
                                is_start_byte = false;
                                message_hex_temp.remove(message_hex_temp.len() - 1);
                                message_hex_temp.remove(message_hex_temp.len() - 1);
                                message_hex_temp.remove(message_hex_temp.len() - 1);
                                message_hex_temp.remove(message_hex_temp.len() - 1);
                            }
                            //                        println!("Contains stop data.");
                        } else {
                            message_hex_temp_window.clear();
                            message_hex_temp_window.shrink_to_fit();
                        }
                    }

                    if is_stop_byte {
                        is_stop_byte = false;
                        // Copying the message for further usage
                        message_hex = message_hex_temp.clone();
                        //println!("FINAL MESSAGE: hex : {:X?}", message_hex);

                        if message_hex.len()
                            == ((get_length_of_message_data(message_hex.as_ref()) + 3)
                                * VARIABLE_LENGTH_BYTES as u32)
                                as usize
                            && message_hex.len() <= 256
                        {
                            let crc_status: bool = crc_check(&message_hex, &crc_table);
                            //                        println!("{:X?}", crc_status);

                            let string = format!("{}", Utf8Lossy(&message_hex));

                            //                        println!("converted: {:#?}", string);
                            //data_from_serial_port.clear();
                            //data_from_serial_port.resize(465, 0);

                            // Will need to implement some TX to get data elsewhere - to print it in
                            // another thread
                            if crc_status == true {
                                let message_structured: Value =
                                    get_structured_message_data(&message_hex);

                                //                               println!("Message Structued is : {:#?}", message_structured);
                                //                               let mut text =
                                //                                   format!("Message Structued is : {:#?}", message_structured);
                                // test_file.write_all(text.as_ref()).unwrap();
                                //println!("The serialized data: {:#?}", message_structured.to_string());

                                tokio::time::sleep(tokio::time::Duration::from_nanos(1)).await;
                                tx_data_bf_broadcast_channel
                                    .send(message_structured.into())
                                    .unwrap();
                            }
                        }

                        message_hex_temp_window.clear();
                        message_hex_temp_window.resize(0, 0);
                        // Clearing and shringking the temp vec of the message to save memory
                        message_hex_temp.clear();
                        //println!("hex temp clear: {:X?}", message_hex_temp);
                        message_hex_temp.shrink_to_fit();
                        //println!("Shrinked length {}", message_hex_temp.len());
                    }
                }
            }

            //           println!("data_read = false");
            data_read = false;
            //is_start_byte = false;
            //is_stop_byte = false;
        } else {
            data_from_serial_port.clear();
            data_from_serial_port.resize(140 * 8 * VARIABLE_LENGTH_BYTES, 0);
        }
    }
}
