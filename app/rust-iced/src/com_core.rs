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
use fixed::{types::extra::U15, FixedI32};
use serde_json::Value;
use serialport::SerialPort;
use serialport::SerialPortType;
use serialport::UsbPortInfo;
use std::cell::RefCell;
use std::ops::Add;

use tokio::sync::broadcast;
use tokio::sync::watch;

pub mod control;
use control::control_task;

pub mod message;
use message::get_structured_message_data;

pub mod serial;
use serial::serial_port_task;

pub fn initialize_serial_port(
    serial_port_name: String,
    baud_rate: u32,
) -> Option<Box<dyn SerialPort>> {
    println!("the serialport used is {}", serial_port_name);

    if serial_port_name != "not selected" && baud_rate != 0 {
        println!("seting serial port");
        let mut serial_port: Box<dyn SerialPort> = serialport::new(serial_port_name, baud_rate)
            .timeout(std::time::Duration::from_millis(100))
            .open()
            .expect("Error");
       serial_port.set_flow_control(serialport::FlowControl::Software).unwrap();

        Some(serial_port)
    } else {
        None
    }
}
// Struct used for holding various info about connected Serial Ports
#[derive(Debug, Clone)]
pub struct SerialPortConnectInfo {
    pub port_name: String,
    pub manufacturer: String,
    pub serial_number: String,
    pub product: String,
}

// Function for obtaining the info about connected devices to the USB ports for serial connection
// interface
pub fn get_serial_port_list() -> Vec<SerialPortConnectInfo> {
    let mut serial_com_vec_ports: Vec<SerialPortConnectInfo> = Vec::new();
    if let Ok(available_ports) = serialport::available_ports() {
        //        println!("Available serial ports: {:#?}", available_ports);
        for port in available_ports {
            let mut one_serial = SerialPortConnectInfo {
                serial_number: "".to_string(),
                manufacturer: "".to_string(),
                product: "".to_string(),
                port_name: "".to_string(),
            };
            match port.port_type.clone() {
                SerialPortType::UsbPort(info) => {
                    one_serial.serial_number =
                        info.serial_number.clone().unwrap_or(Default::default());
                    one_serial.manufacturer =
                        info.manufacturer.clone().unwrap_or(Default::default());
                    one_serial.product = info.product.clone().unwrap_or(Default::default());
                    //                    println!("VID: {:04x}", info.vid);
                    //                    println!("PID: {:04x}", info.pid);
                    //                    println!("Serial Number: {:?}", info.serial_number);
                    //                    println!("Manufacturer: {:?}", info.manufacturer);
                    //                    println!("Product: {:?}", info.product);
                }
                _ => {
                    //println!("Non-USB port");
                }
            }
            match port.port_name {
                port_name => {
                    one_serial.port_name = port_name.clone();
                    serial_com_vec_ports.push(one_serial.clone());
                    //                    println!("Device Name: {:?}", port.port_type);
                    //                    println!("Port Name: {}", port_name);
                }
            }
            //println!("This is obtained to the generated custom struct:\n{:#?}", one_serial);
        }
    }
    //println!("This is a vector of obtained data about serial ports: {:#?}", serial_com_vec_ports);
    return serial_com_vec_ports;
}

pub fn write_led_on_to_uart(mut serial_port: Box<dyn SerialPort>) {
    let buf: [u8; 4] = [0x6C, 0x6C, 0x6C, 0x6C];
    let _ = serial_port.clear(serialport::ClearBuffer::Output); // Need for software control flow
                                                                // to work as expected when sending
                                                                // data via UART to FPGA
    match serial_port.write_all(&buf) {
        Ok(v) => {
            println!("{:#?} bytes written", v)
        }
        Err(e) => {
            println!("Error writing bytes to serial port. {:#?}", e)
        }
    }
}

pub fn check_bytes_to_write_uart(mut serial_port: Box<dyn SerialPort>) {
    match serial_port.bytes_to_write() {
        Ok(v) => {
            println!("{:#?} bytes to write", v)
        }
        Err(e) => {
            println!("Error checking bytes to write serial port. {:#?}", e)
        }
    }

}

pub fn write_led_off_to_uart(mut serial_port: Box<dyn SerialPort>) {
    let buf: [u8; 4] = [0x23, 0x23, 0x23, 0x23];
    let _ = serial_port.clear(serialport::ClearBuffer::Output); // Need for software control flow
                                                                // to work as expected when sending
                                                                // data via UART to FPGA
    match serial_port.write_all(&buf) {
        Ok(v) => {
            println!("{:#?} bytes written", v)
        }
        Err(e) => {
            println!("Error writing bytes to serial port. {:#?}", e)
        }
    }

    //    match  serial_port.bytes_to_write(){
    //        Ok(v)=> {println!("{:#?} bytes to write", v)},
    //        Err(e) => {println!("Error checking bytes to write serial port. {:#?}", e)}
    //
    //    }
}

// Struct for the backend and frontend Broadcast Channels
#[derive(Debug)]
pub struct BroadcastBFValues {
    pub tx: Option<broadcast::Sender<serde_json::Value>>,
    pub rx: RefCell<Option<broadcast::Receiver<serde_json::Value>>>,
}

// Struct for the general Broadcast Channels
#[derive(Debug)]
pub struct BroadcastValues {
    pub tx: tokio::sync::broadcast::Sender<Value>,
    pub rx: tokio::sync::broadcast::Receiver<Value>,
}

#[derive(Debug)]
pub struct WatchChannelValues {
    pub tx: tokio::sync::watch::Sender<bool>,
    pub rx: tokio::sync::watch::Receiver<bool>,
}

pub fn drop_broadcast_channel_listening(
    mut broadcast_channel_listening_handle_from_self: Option<tokio::task::JoinHandle<()>>,
) {
    if let Some(broadcast_channel_listening_handle) =
        broadcast_channel_listening_handle_from_self.take()
    {
        println!("Dropping channel listening for broadcasts.");
        broadcast_channel_listening_handle.abort();
    } else {
        println!(
            "No channel listening for broadcast is live.Dropping channel listening for broadcasts."
        );
    }
}

pub fn drop_broadcast_channel(
    mut broadcast_channel_from_self: Option<BroadcastValues>,
    mut broadcast_channel_listening_handle_from_self: Option<tokio::task::JoinHandle<()>>,
) {
    if let Some(broadcast_channel) = broadcast_channel_from_self.take() {
        println!("Dropping initialized broadcast channel.");
        crate::com_core::drop_broadcast_channel_listening(
            broadcast_channel_listening_handle_from_self.take(),
        );

        drop(broadcast_channel);
    } else {
        println!("Broadcast channel is not initialized.");
    }
}

pub fn drop_watch_channel(mut watch_channel_from_self: Option<WatchChannelValues>) {
    // Implement dropping listening to the watch channel
    if let Some(watch_channel) = watch_channel_from_self.take() {
        println!("Dropping initialized watch channel.");
        drop(watch_channel);
    } else {
        println!("Watch channel is not initialized.");
    }
}

// Function for initializing serial port from the backend and passing the returned obect to
// frontend GUI for later usage
pub fn initialize_serial_port_command(
    mut serial_port_handle: Option<&Box<dyn SerialPort>>,
    serial_port_selected: String,
    serial_port_baud_rate: u32,
) -> Option<Box<dyn SerialPort>> {
    println!("Initializing the Serial Port task.");
    // Checking if any serial_port is already initialized, if yes, print message, otherwise initialize the port object to a self
    if let Some(serial_port) = serial_port_handle.as_mut() {
        println!("Serial port already initialized.");
        None
    } else {
        println!("New serial port: {}", serial_port_selected);
        let serial_port_handle_ret =
            initialize_serial_port(serial_port_selected.clone(), serial_port_baud_rate.clone());
        serial_port_handle_ret
    }
}

// Function for intializing watch CONTROL channel for controlling the start and stop of data
// fetching from a serial port
pub fn initialize_watch_channel(
    serial_control_on_off_channel: Option<&WatchChannelValues>,
) -> Option<WatchChannelValues> {
    println!("Initializing the Watch Channel");

    if serial_control_on_off_channel.is_none() {
        let (tx_watch, rx_watch): (
            tokio::sync::watch::Sender<bool>,
            tokio::sync::watch::Receiver<bool>,
        ) = tokio::sync::watch::channel(true);

        let serial_control_on_off_channel_ret = Some(WatchChannelValues {
            tx: tx_watch,
            rx: rx_watch,
        });

        serial_control_on_off_channel_ret
    } else {
        println!("Watch channel already initialized.");
        None
    }
}

// Function for initilizing data fetch tokio thread
pub fn initialize_serial_data_fetch(
    serial_port_handle: Option<&Box<dyn SerialPort>>,
    broadcast_data_bf_channel_tx: Option<broadcast::Sender<serde_json::Value>>,
    serial_control_on_off_channel: Option<&WatchChannelValues>,
) -> Option<tokio::task::JoinHandle<()>> {
    if let Some(serial_port_handle) = serial_port_handle {
        if let Some(tx_data_bf_broadcast_channel) = broadcast_data_bf_channel_tx.clone() {
            if let Some(serial_control_on_off) = serial_control_on_off_channel {
                println!("Initializing the serial data fetch.");
                let rx_serial_control_on_off = serial_control_on_off.rx.clone();
                let tx_serial_control_on_off = serial_control_on_off.tx.send(false);
                let serial_data_fetch_handle_ret = tokio::spawn(serial::serial_port_task(
                    rx_serial_control_on_off,
                    serial_port_handle.try_clone().unwrap(),
                    tx_data_bf_broadcast_channel,
                ));
                Some(serial_data_fetch_handle_ret)
            } else {
                println!("When trying to enable fetching the serial data no watch channel is enabled. No further actions were taken.");
                None
            }
        } else {
            println!("When trying to enable fetching the serial data no broadcast channel is enabled. No further actions were taken.");
            None
        }
    } else {
        println!("When trying to enable fetching the serial data no serial port is opened. No further actions were taken.");
        None
    }
}
