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

use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::fs::File;
use std::io::prelude::*;
use std::{u128, usize};

use chrono::Local;
use com_core::{BroadcastBFValues, BroadcastValues, SerialPortConnectInfo, WatchChannelValues};
use iced::widget::{button, Button, Column, Container, PickList, Row, Text, TextInput};
use iced::{executor, theme, Alignment, Application, Command, Element, Length, Settings, Theme};
use native_dialog::MessageDialog;
use serialport::SerialPort;
use sudo;
use tokio::sync::broadcast;

use plotters::prelude::*;
use plotters_iced::{Chart, ChartBuilder, ChartWidget, DrawingBackend};
pub mod com_core;
pub mod gui;
pub mod utils;
use std::time::{SystemTime, UNIX_EPOCH};
use gui::{*};

// Width of the desktop selection column for interface
// default/debugging/debugging free etc.
const DESKTOP_SELECTION_COLUMN_WIDTH: f32 = 120.0;


fn main() -> iced::Result {
    sudo::escalate_if_needed().expect("Could not run as a sudo.");
    //let mut file_test = File::create("/home/petr/test.txt").unwrap();
    //file_test.write_all(b"Hello there").expect("Error");
    let (sender, receiver) = broadcast::channel::<serde_json::Value>(5500);
    // Starting the tokio runtime this way
    // or use a macro #[tokio::main] and async main function

    let app_flags = AppFlags {
        broadcast_sender: sender,
        broadcast_receiver: receiver,
    };
    Rtm::run(Settings::with_flags(app_flags))
}
pub struct AppFlags {
    broadcast_receiver: broadcast::Receiver<serde_json::Value>,
    broadcast_sender: broadcast::Sender<serde_json::Value>,
}
pub struct Rtm {
    broadcast_data_bf_channel: BroadcastBFValues,                       // Backend to Frontend Channels
    task_handle: Option<tokio::task::JoinHandle<()>>,                   // Test Handle for spawning Tokio threads
    serial_port_handle: Option<Box<dyn SerialPort>>,                    // Handle for serial port connection object
    serial_control_on_off_channel: Option<WatchChannelValues>,          // Tokio watch channels for enabling and disabling the loop for fetching data in the backend
    serial_data_fetch_handle: Option<tokio::task::JoinHandle<()>>,      // Tokio thread handle for fetching serial data in the backend
    data: serde_json::Value,                                            // Data received from backend in a subscription which is passed to the new_data_points
    new_data_points: (VecDeque<u128>, HashMap<u128, VecDeque<f64>>),    // Data received from backend in a tuple (x-samples, HashMap with data from variables)
    display_mode: Option<String>,                                       // Running buffer or default mode selected value, will have to make a struct for it
    display_mode_select_values: Vec<String>,                            // List of possible modes running | default | add more later
    serial_ports_available: SerialPortData,                             // Struct with data about serial ports (baud rate, names, info)
    running_buffer: RunningBuffer,                                      // Running buffer settings - size in string and parsed size
    navigation_layout: NavigationLayout,                                // Desktops - default | debug | debug free | add more
    output_file: OutputFile,                                            // Struct for handling opened file for saving general data which are printed to a chart
    theme: Theme,                                                       // Handling Theme changing
}

#[derive(Debug, Clone)]
pub enum Message {
    HelloFromButton,                    // Debug action
    DropHelloFromButton,                // Debug action
    StartTask,                          // Debug action - writing on data to fpga
    StopTask,                           // Debug action - writing off data to fpga
    InitializeSerialPort,               // Single task - initialize serial port object
    DropSerialPort,                     // Dropping the initialized serial port object
    InitializeWatchChannel,             // Initializing control channel for enabling/disabling fetching data from backend
    DropWatchChannel,                   // Dropping control channel for enabling/disabling fetching data from backend
    InitializeSerialDataFetch,          // Initializing thread - tokio spawn for backend with all checks
    StartSerialDataFetch,               // Sending control signal via tokio watch channel to START the loop of fetching data from backend
    StopSerialDataFetch,                // Sending control signal via tokio watch channel to STOP the loop for fetching the data via serial port
    DropSerialDataFetch,                // Dropping the initialized thread from tokio spawn for backend fetching data with all checks
    ExternalDataReceived(serde_json::Value), // Message for handling received data from the backend via subscription to a broadcast channel, moving the dat to the frontend component
    ClearFigure,                        // Clearing fetched data from the vectors in a HashMap in a new_data
    SetDisplayMode(String),             // For seting the display mode of chart and data, initially default | running buffer
    GetAvailableSerialPortData,         // Fetching the available serial ports in the gui to later
                                        // pass the information to a PickList to be able to select available ports
    SetSerialPort(String),              // Set the selected serial port in the PickList
    SetBaudRate(String),                // Set the selected baudrate from an text input widget
    SetRunningBuffer(String),           // Set the the value of a running buffer constraint
    ChangeDesktop(String),              // Set selected desktop based on the selection default | debug | debug free
    CreateOutputFile,                   // Create output file for data based on a current timestamp
    StartOutputFile,                    // Start saving the obtained data to the recently created file
    StopOutputFile,                     // Stop saving the obtained data to the recentlyu created file
    InitializeRtm,                      // Chain of actions to initialize the control watch channels, serial port and spawn a backend thread for fetching the data
    ThemeChanged(Theme),                // Change the theme to selected theme
}

impl Application for Rtm {
    type Message = Message;
    type Theme = Theme;
    type Executor = executor::Default;
    type Flags = AppFlags;

    fn new(flags: Self::Flags) -> (Self, iced::Command<Message>) {
        // Vector for samples in X axis which is used as an input to the HashMap
        let x_axis_values_internal: VecDeque<u128> = VecDeque::new();

        // Vector available serial ports
        let mut serial_com_vec_port_names_internal: Vec<String> = Vec::new();
        // Not selected value as a default value
        serial_com_vec_port_names_internal.push("not selected".to_string());
        // Get the serial port data
        let serial_port_list_internal = com_core::get_serial_port_list();
        // Push the names to the list to be selected in a PickList by user
        for port in serial_port_list_internal.clone() {
            serial_com_vec_port_names_internal.push(port.port_name);
        }

        // Initiate structure for serial port settings with serial port names, selected value and
        // baudrates
        let serial_ports_available_internal = SerialPortData {
            names: serial_com_vec_port_names_internal.clone(),
            selected: "not selected".to_string(),
            structure: serial_port_list_internal,
            baud_rate: Default::default(),
            baud_rate_string: Default::default(),
        };

        // Struct for desktops/navigation layout selection
        let navigation_layout_internal = NavigationLayout {
            // Here insert new desktops/layouts
            available_desktops: vec![
                "default".to_string(),
                "debug".to_string(),
                "debug free".to_string(),
            ],
            // Currently active desktop
            selected_desktop: Some("default".to_string()),
        };

        let app = Rtm {
            // Initialization of struct with broadcast channels, connecting backend and frontend
            // compontens
            broadcast_data_bf_channel: BroadcastBFValues {
                tx: Some(flags.broadcast_sender),
                rx: RefCell::new(Some(flags.broadcast_receiver)),
            },

            task_handle: None,
            serial_port_handle: None,
            serial_control_on_off_channel: None,
            serial_data_fetch_handle: None,
            data: serde_json::Value::default(),
            new_data_points: (x_axis_values_internal, HashMap::new()),
            display_mode: Some("default".to_string()),
            display_mode_select_values: vec!["default".to_string(), "running".to_string()],
            serial_ports_available: serial_ports_available_internal,
            running_buffer: Default::default(),
            navigation_layout: navigation_layout_internal,
            output_file: Default::default(),
            theme: iced::Theme::TokyoNightStorm.into(),
        };
        (app, Command::none())
    }

    fn title(&self) -> String {
        String::from("RTM v0.0.1")
    }

    fn update(&mut self, message: Self::Message) -> Command<Message> {
        match message {
            Message::ThemeChanged(theme) => {
                self.theme = theme;

                Command::none()
            }
            Message::InitializeRtm => {
                if self.serial_port_handle.is_none() {
                    let serial_port_handle_internal = com_core::initialize_serial_port_command(
                        self.serial_port_handle.as_ref(),
                        self.serial_ports_available.selected.clone(),
                        self.serial_ports_available.baud_rate.clone(),
                    );

                    if serial_port_handle_internal.is_some() {
                        self.serial_port_handle = serial_port_handle_internal;

                        let serial_control_on_off_channel = com_core::initialize_watch_channel(
                            self.serial_control_on_off_channel.as_ref(),
                        );
                        if serial_control_on_off_channel.is_some() {
                            self.serial_control_on_off_channel = serial_control_on_off_channel;

                            let serial_data_fetch_handle_internal =
                                com_core::initialize_serial_data_fetch(
                                    self.serial_port_handle.as_ref(),
                                    self.broadcast_data_bf_channel.tx.clone(),
                                    self.serial_control_on_off_channel.as_ref(),
                                );

                            if serial_data_fetch_handle_internal.is_some() {
                                self.serial_data_fetch_handle = serial_data_fetch_handle_internal;
                            } else {
                                let _ = MessageDialog::new()
                            .set_title("Error spawning Serial Port Backend Thread!")
                            .set_text(
                                "Error spawning the Thread for fetching the data from selected Serial Port. ",
                            )
                            .set_type(native_dialog::MessageType::Error)
                            .show_alert();
                            }
                        } else {
                            let _ = MessageDialog::new()
                            .set_title("Error initializing the Control Channel!")
                            .set_text(
                                "Error initializing the control channel for Serial Port thread.",
                            )
                            .set_type(native_dialog::MessageType::Error)
                            .show_alert();
                        }
                    } else {
                        let _ = MessageDialog::new()
                        .set_title("Error initializing the Serial Port!")
                        .set_text("There was a problem regarding valid Serial Port, Baud Rate or initialization of the backend thread.")
                        .set_type(native_dialog::MessageType::Error)
                        .show_alert();
                    }
                }
                else {
                    let _ = MessageDialog::new()
                        .set_title("Error initializing the Serial Port!")
                        .set_text("Port already initialized.")
                        .set_type(native_dialog::MessageType::Error)
                        .show_alert();

                }

                Command::none()
            }
            Message::GetAvailableSerialPortData => {
                self.serial_ports_available.names.clear();
                self.serial_ports_available.names.shrink_to_fit();

                let serial_port_list_internal = com_core::get_serial_port_list();
                for port in serial_port_list_internal.clone() {
                    self.serial_ports_available.names.push(port.port_name);
                }
                println!(
                    "Serial port data: {:#?}",
                    self.serial_ports_available.structure
                );
                Command::none()
            }
            Message::HelloFromButton => {
                let _ = MessageDialog::new()
                    .set_title("This is a dialog!")
                    .set_text("This is another dialog internal text.")
                    .set_type(native_dialog::MessageType::Error)
                    .show_alert();

                Command::none()
            }
            Message::DropHelloFromButton => {
                if let Some(task_handle) = self.task_handle.take() {
                    println!("Aborting task_handle.");
                    task_handle.abort();
                }

                Command::none()
            }
            Message::StartTask => {
                println!("Starting task.");
                // Getting the serial port from the self object via Some - because it is an option
                // and taking it as a mutable - not take, to be able to reuse it in another
                // commands
                if let Some(serial_port) = self.serial_port_handle.as_ref() {
                    // Send data to uart with reusable serial_port handle obtained from the future
                    // - try_clone() should do the job here
                    com_core::write_led_on_to_uart(serial_port.try_clone().unwrap()); //Check if all bytes have been written to uart
                    com_core::check_bytes_to_write_uart(serial_port.try_clone().unwrap());
                } else {
                    println!("Serial port not initialized.");
                }
                Command::none()
            }
            Message::StopTask => {
                println!("Stopping task.");
                // Getting the serial port from the self object via Some - because it is an option
                // and taking it as a mutable - not take, to be able to reuse it in another
                // commands
                if let Some(serial_port) = self.serial_port_handle.as_mut() {
                    // Send data to uart with reusable serial_port handle obtained from the future
                    // - try_clone() should do the job here
                    com_core::write_led_off_to_uart(serial_port.try_clone().unwrap());
                    // Check if all bytes have been written to uart
                    com_core::check_bytes_to_write_uart(serial_port.try_clone().unwrap());
                } else {
                    println!("Serial port not initialized.");
                }
                Command::none()
            }
            Message::InitializeSerialPort => {
                let serial_port_handle_internal = com_core::initialize_serial_port_command(
                    self.serial_port_handle.as_ref(),
                    self.serial_ports_available.selected.clone(),
                    self.serial_ports_available.baud_rate.clone(),
                );
                if serial_port_handle_internal.is_some() {
                    self.serial_port_handle = serial_port_handle_internal;
                } else {
                    let _ = MessageDialog::new()
                        .set_title("Error initializing the Serial Port!")
                        .set_text("No valid Serial Port or Baud Rate is selected.")
                        .set_type(native_dialog::MessageType::Error)
                        .show_alert();
                }
                Command::none()
            }
            Message::DropSerialPort => {
                println!("Dropping the Serial Port");
                // Must use self.serial_port_handle.take() otherwise if used as_mut(), the value
                // wont be dropped, because only the reference would be passed to Some as an optio
                // and no object would be dropped
                if let Some(serial_port) = self.serial_port_handle.take() {
                    drop(serial_port)
                } else {
                    println!("Serial port not initialized.");
                }
                Command::none()
            }

            Message::InitializeWatchChannel => {
                let serial_control_on_off_channel =
                    com_core::initialize_watch_channel(self.serial_control_on_off_channel.as_ref());
                if serial_control_on_off_channel.is_some() {
                    self.serial_control_on_off_channel = serial_control_on_off_channel;
                } else {
                    let _ = MessageDialog::new()
                        .set_title("Error initializing the Control Channel!")
                        .set_text("Error initializing the control channel for Serial Port thread.")
                        .set_type(native_dialog::MessageType::Error)
                        .show_alert();
                }
                Command::none()
            }
            Message::DropWatchChannel => {
                com_core::drop_watch_channel(self.serial_control_on_off_channel.take());
                Command::none()
            }
            Message::InitializeSerialDataFetch => {
                let serial_data_fetch_handle_internal = com_core::initialize_serial_data_fetch(
                    self.serial_port_handle.as_ref(),
                    self.broadcast_data_bf_channel.tx.clone(),
                    self.serial_control_on_off_channel.as_ref(),
                );

                if serial_data_fetch_handle_internal.is_some() {
                    self.serial_data_fetch_handle = serial_data_fetch_handle_internal;
                } else {
                    let _ = MessageDialog::new()
                            .set_title("Error spawning Serial Port Backend Thread!")
                            .set_text(
                                "Error spawning the Thread for fetching the data from selected Serial Port. ",
                            )
                            .set_type(native_dialog::MessageType::Error)
                            .show_alert();
                }
                Command::none()
            }
            Message::DropSerialDataFetch => {
                if let Some(serial_data_fetch_handle) = self.serial_data_fetch_handle.take() {
                    println!("Aborting the serial_data_fetch_handle");
                    serial_data_fetch_handle.abort();
                } else {
                    println!("No serial_data_fetch_handle is live. Nothing to abort.");
                }
                Command::none()
            }
            Message::StopSerialDataFetch => {
                if let Some(watch_channel) = self.serial_control_on_off_channel.as_mut() {
                    let _ = watch_channel.tx.send(false);
                }
                Command::none()
            }
            Message::StartSerialDataFetch => {
                if let Some(watch_channel) = self.serial_control_on_off_channel.as_mut() {
                    let _ = watch_channel.tx.send(true);
                }
                Command::none()
            }
            Message::ExternalDataReceived(message) => {
                self.data = message.clone();

                let mut index = 0;

                // If the data to be processed are valid
                if let Some(obj) = self.data.as_object() {
                    // If there were no data, starting to write the data to the vectors
                    // start with 0
                    if self.new_data_points.0.len() == 0 {
                        self.new_data_points.0.push_back(0);
                    } else {
                        // Otherwise increment the x axis value
                        self.new_data_points
                            .0
                            .push_back(self.new_data_points.0.back().unwrap() + 1);
                    }

                    // Writing sample at X axis to the ouput file
                    if self.output_file.enable_output == true {
                        if let Some(file_handler) = &mut self.output_file.handler {
                            let index_string = self
                                .new_data_points
                                .0
                                .get(self.new_data_points.0.len() - 1)
                                .unwrap();
                            let formatted_string = format!("{}", index_string);
                            utils::write_output_to_file(file_handler, formatted_string);
                        }
                    }

                    // Loop by variables in the received data
                    for (key, value) in obj {
                        index = index + 1; // the HashMap is indexed from 1 here

                        //                        println!("obj length, {}", obj.len());
                        //                        println!("working on index: {}", index);
                        //                        println!("0 length: {}", self.new_data_points.0.len());

                        // Saving data to the HaashMap part to VecDeque objects for variables
                        self.new_data_points
                            .1
                            .entry(index)
                            .or_insert_with(VecDeque::new)
                            .push_back(value.as_f64().unwrap()); // Pushing the new data to the
                                                                 // VecDeque object in a HashMap

                        // If the output file data saving is enabled
                        if self.output_file.enable_output == true {
                            if let Some(v) = self.output_file.handler.as_mut() {
                                println!("0 direct len {}", self.new_data_points.0.len());
                                println!(
                                    "data points index: {}, data at self.. :{:#?}",
                                    index,
                                    self.new_data_points.1.get(&index).unwrap().len()
                                );
                                // Preparing string of a current variable value
                                let data_string = self
                                    .new_data_points
                                    .1
                                    .get(&index)
                                    .unwrap()
                                    .get(self.new_data_points.0.len() - 1) // is zero based, so
                                                                           // when using length,
                                                                           // need to subtract 1
                                    .unwrap();
                                let formatted_string = format!(",{}", data_string); // format the
                                                                                    // obtained
                                                                                    // data
                                // Write the formatted data to the output file
                                // writing current variable value
                                match v.write_all(formatted_string.as_ref()) {
                                    Ok(v) => {
                                        println!("Ok writing a file for data {:#?}, {:#?}", key, v);
                                    }
                                    Err(e) => {
                                        println!("Error writing a file {:#?}", e);
                                    }
                                }
                            }
                        }

                        // Firstly popping Y values, because checking the length of X axis value
                        // and based on that length, the Y value vectors should have same length
                        // and after popping Y values, the X value can be popped, otherwise if
                        // changed the position of X popping, other indexes at Y would be popped
                        
                        // Checks for running buffer mode, to pop the data after the buffer length
                        // is reached
                        if self.new_data_points.0.len()
                            == self.running_buffer.size as usize + 1 as usize // the + 1 is used
                                                                              // because when
                                                                              // length is 200 the
                                                                              // valu still should
                                                                              // not be popped,
                                                                              // because there
                                                                              // would be only 199
                                                                              // values then, so
                                                                              // when the length
                                                                              // reaches 201 and
                                                                              // comparing with 200
                                                                              // + 1, the 1 value
                                                                              // is popped, so the
                                                                              // displyed length is
                                                                              // 200
                            && self.display_mode == Some("running".to_string())
                        {
                            // Popping Y axis values and shrinking the VecDeque
                            self.new_data_points.1.entry(index).or_default().pop_front();
                            self.new_data_points
                                .1
                                .entry(index)
                                .or_default()
                                .shrink_to_fit();
                        }
                    } // Here ends looping through variable objects in received data JSON

                    // And popping of data is solved

                    // Checking the X sample values to same length as for Y values
                    // and popping the data
                    if self.new_data_points.0.len()
                        == self.running_buffer.size as usize + 1 as usize
                        && self.display_mode == Some("running".to_string())
                    {
                        //                        println!("LENGTH: {:#?}", self.new_data_points.0.len());
                        self.new_data_points.0.pop_front();
                        self.new_data_points.0.shrink_to_fit();
                        //                        println!("LENGTH: {:#?}", self.new_data_points.0.len());
                    }

                    // Writing new line after X axis value and corresponding Y values were written
                    // to output file
                    if let Some(v) = self.output_file.handler.as_mut() {
                        match v.write_all(b"\n") {
                            Ok(v) => {
                                println!("Ok writing a file ending part, {:#?}", v);
                            }
                            Err(e) => {
                                println!("Error writing a file {:#?}", e);
                            }
                        }
                    }
                }

                //                println!("Length of 0 {}", self.new_data_points.0.len());
                //                if let Some(test_data) = self.new_data_points.1.get(&1) {
                //                    println!("Length of 1 {:#?}", test_data.len());
                //                }
                //                if let Some(test_data) = self.new_data_points.1.get(&2) {
                //                    println!("Length of 2 {:#?}", test_data.len());
                //                }
                //                if let Some(test_data) = self.new_data_points.1.get(&3) {
                //                    println!("Length of 3 {:#?}", test_data.len());
                //                }
                //                if let Some(test_data) = self.new_data_points.1.get(&4) {
                //                    println!("Length of 4 {:#?}", test_data.len());
                //                }

                Command::none()
            }
            Message::ClearFigure => {
                self.new_data_points.0.clear();
                self.new_data_points.0.shrink_to_fit();
                self.new_data_points.1.clear();
                Command::none()
            }
            Message::SetDisplayMode(mode) => {
                println!("mode is: {}", mode);
                self.display_mode = Some(mode);

                Command::none()
            }
            Message::SetSerialPort(serialport) => {
                println!("serialport selected is: {}", serialport);
                self.serial_ports_available.selected = serialport;

                Command::none()
            }
            Message::SetBaudRate(baudrate) => {
                println!("baudrate selected is: {}", baudrate);
                self.serial_ports_available.baud_rate_string = baudrate.clone();
                let result: Result<u32, _> = baudrate.parse();
                let baudrate_parsed = match result {
                    Ok(number) => number,
                    Err(e) => {
                        println!("Error parsing the baudrate value from the GUI. {:#?}", e);
                        0
                    }
                };
                self.serial_ports_available.baud_rate = baudrate_parsed;
                println!("baudrate selected is: {}", baudrate_parsed);

                Command::none()
            }
            Message::SetRunningBuffer(buffer_size) => {
                println!("buffer selected is: {}", buffer_size);
                if buffer_size.clone() != 0.to_string() {
                    self.running_buffer.size_string = buffer_size.clone();
                }
                let result: Result<u64, _> = buffer_size.parse();
                let buffer_size_parsed = match result {
                    Ok(number) => number,
                    Err(e) => {
                        println!("Error parsing the running buffer size. {:#?}", e);
                        0
                    }
                };
                self.running_buffer.size = buffer_size_parsed;
                println!("buffer selected is: {}", buffer_size_parsed);

                Command::none()
            }
            Message::ChangeDesktop(desktop) => {
                println!("desktop selected is: {}", desktop);
                self.navigation_layout.selected_desktop = Some(desktop);

                Command::none()
            }
            Message::CreateOutputFile => {
                let local_time = Local::now().format("%Y-%m-%d-%H:%M:%S").to_string();
                let file_name = "./../".to_owned() + &local_time + "-rtm-capture.csv";
                let file_test = File::create(file_name.to_string()).unwrap();

                self.output_file.handler = Some(file_test);
                Command::none()
            }
            Message::StartOutputFile => {
                self.output_file.enable_output = true;
                Command::none()
            }
            Message::StopOutputFile => {
                self.output_file.enable_output = false;
                Command::none()
            }
        }
    }

    fn subscription(&self) -> iced::Subscription<Self::Message> {
        // Subscription of frontend to the backend data
        // the channels are initialized before running the GUI in the main function
        let broadcast_listener_subscription = iced::subscription::unfold(
            "broadcast listener main",
            self.broadcast_data_bf_channel.rx.take(),
            move |mut receiver| async move {
                let message = receiver.as_mut().unwrap().recv().await.unwrap();
                (Message::ExternalDataReceived(message), receiver)
            },
        );

        // Batch is here used to be able to add more subscriptions later
        iced::Subscription::batch(vec![broadcast_listener_subscription])
    }

    fn view(&self) -> iced::Element<'_, Self::Message> {

        // Changing themes
        let theme_widget: PickList<'_, Theme, &[Theme], Theme, Message> =
            iced::widget::pick_list::PickList::new(
                iced::Theme::ALL,
                Some(self.theme.clone()),
                Message::ThemeChanged,
            );

        // Chain of action used to initialize all needed to fetch data
        let initialize_rtm_button: Button<Message> = Button::new("Initialize RTM")
            .style(theme::Button::Destructive)
            .on_press(Message::InitializeRtm);

        // Changing dekstops
        let desktop_selection_title = Text::<'_, Theme>::new("Desktop");
        let desktop_selection_widget: PickList<'_, String, &[String], String, Message> =
            iced::widget::pick_list::PickList::new(
                self.navigation_layout.available_desktops.as_ref(),
                self.navigation_layout.selected_desktop.clone(),
                Message::ChangeDesktop,
            );
        let desktop_selection_widget_debug_free: PickList<'_, String, &[String], String, Message> =
            iced::widget::pick_list::PickList::new(
                self.navigation_layout.available_desktops.as_ref(),
                self.navigation_layout.selected_desktop.clone(),
                Message::ChangeDesktop,
            );

        // Running buffer settings
        let running_buffer_size_text_info_widget = Text::<'_, Theme>::new(format!(
            "Running Buffer Size: {}",
            self.running_buffer.size.clone()
        ));

        // Baud rate settings
        let baud_rate_text_info_widget = Text::<'_, Theme>::new(format!(
            "BaudRate: {}",
            self.serial_ports_available.baud_rate_string
        ));

        // Printing out selected serial port
        let serial_port_text_info_widget = Text::<'_, Theme>::new(format!(
            "SerialPort: {}",
            self.serial_ports_available.selected
        ));


        // Running buffer
        let text_progress_bar = Text::<'_, Theme>::new("Running buffer");

        let text_progress_bar_value = Text::<'_, Theme>::new(format!(
            "Progress value: {:#?}%",
            100.0 * self.new_data_points.0.len() as f32 / self.running_buffer.size.clone() as f32
        ));

        // Displaying X samples from x axis
        let x_axis_data_internal = match self.new_data_points.0.back() {
            Some(&x) => x,
            None => 0,
        };

        // Selection mode default | debug
        let selection_mode_title = Text::<'_, Theme>::new("Selection mode");

        let mode_selection_widget: PickList<'_, String, &[String], String, Message, _, _> =
            iced::widget::pick_list::PickList::new(
                self.display_mode_select_values.as_ref(),
                self.display_mode.clone(),
                Message::SetDisplayMode,
            );

        // Selecting serial ports from a dropdown
        let serialport_selection_widget: PickList<'_, String, &[String], String, Message> =
            iced::widget::pick_list::PickList::new(
                self.serial_ports_available.names.as_ref(),
                Some(self.serial_ports_available.selected.clone()),
                Message::SetSerialPort,
            );

        // Input of baudrate value
        let baudrate_input_widget: TextInput<'_, Message> =
            TextInput::new("Default", &self.serial_ports_available.baud_rate_string)
                .on_input(Message::SetBaudRate);

        // Input of running buffer size
        let running_buffer_input_widget: TextInput<'_, Message> =
            TextInput::new("Default", &self.running_buffer.size_string)
                .on_input(Message::SetRunningBuffer)
                .width(125);

        // Printing out X axis values
        let x_axis_data_widget =
            Text::<'_, Theme>::new(format!("Samples: {}", x_axis_data_internal));

        // Just info text
        let data_serialized_text_widget = Text::<'_, Theme>::new("Obtained data:");

        // Pretty obtained RAW data from backend
        let serialized_data = serde_json::to_string_pretty(&self.data).unwrap();

        let data_serialized_widget = Text::<'_, Theme>::new(serialized_data);

        // Output file handling actions
        let create_output_file_button: Button<Message> =
            Button::new("Create output file").on_press(Message::CreateOutputFile);

        let start_output_file_button: Button<Message> =
            Button::new("Enable data output file").on_press(Message::StartOutputFile);

        let stop_output_file_button: Button<Message> =
            Button::new("Disable data output file").on_press(Message::StopOutputFile);

        let get_available_serial_ports_button: Button<Message> =
            Button::new("Refresh SP").on_press(Message::GetAvailableSerialPortData);

        // Test buttons
        let hello_button: Button<Message> =
            Button::new("Hello from button. And spawning async fn.")
                .on_press(Message::HelloFromButton);
        let hello_button_drop: Button<Message> =
            Button::new("Droping async fn with task_handle").on_press(Message::DropHelloFromButton);

        // Send test data to an FPGA
        let start_task_button: Button<Message> =
            Button::new("Send LED on data to FPGA").on_press(Message::StartTask);
        let stop_task_button: Button<Message> =
            Button::new("Send LED off data to FPGA").on_press(Message::StopTask);

        // Buttons regarding serial port
        let initialize_the_serialport_button: Button<Message> =
            Button::new("Initialize the serial port")
                .on_press(Message::InitializeSerialPort)
                .style(theme::Button::Positive);

        let drop_the_serialport_button: Button<Message> =
            Button::new("Drop the serial port").on_press(Message::DropSerialPort);

        // Testing buttons with custom design
        let custom_button_from_out: Button<Message> = button("Hey from custom button from out")
            .style(theme::Button::Custom(Box::new(EdtTheme)))
            .on_press(Message::StopTask);

        // Initializing Control channel for starting and disabling serial fetch data in a loop
        let initialize_watch_channel_button: Button<Message> =
            Button::new("Initialize serial control channel channel")
                .on_press(Message::InitializeWatchChannel);

        // Dropping initialized Control channel for starting and disabling serial fetch data in a loop
        let drop_watch_channel_button: Button<Message> =
            Button::new("Drop serial control channel channel").on_press(Message::DropWatchChannel);

        // Initializing the thread for obtaining the data from the serial port in a loop
        // the thread loop is controlled using a Control channel
        let initialize_serial_data_fetch_button: Button<Message> =
            Button::new("Initialize serial data fetch.")
                .on_press(Message::InitializeSerialDataFetch);

        // Sending signal "true" via the Control channel for controlling the loop of the serial
        // fetch data
        let start_serial_data_fetch_button: Button<Message> =
            Button::new("Start data fetch").on_press(Message::StartSerialDataFetch);

        // Sending signal "false" via the Control channel for controlling the loop of the serial
        // fetch data
        let stop_serial_data_fetch_button: Button<Message> =
            Button::new("Stop data fetch").on_press(Message::StopSerialDataFetch);

        // Dropping the handle and thus thread which is used for fetching the data from the serial
        // port in a loop controlled by the Control channel
        let drop_serial_data_fetch_button: Button<Message> =
            Button::new("Drop serial data fetch").on_press(Message::DropSerialDataFetch);

        // Clearing the data from corresponding vectors and hashmap, thus successfully cleaning the
        // plotted chart
        let clear_chart_button: Button<Message> =
            Button::new("Clear chart").on_press(Message::ClearFigure);

        // Widget only active in a running mode, which displays percentage on how the running
        // buffer is full before it is full of data, thus the charts starts moving
        let running_mode_progress_bar_widget = iced::widget::ProgressBar::new(
            0.0..=1.0,
            self.new_data_points.0.len() as f32 / self.running_buffer.size as f32,
        );

        let tooltip_test = iced::widget::tooltip(
            Button::new("Clear figure with tooltip").on_press(Message::ClearFigure),
            text_progress_bar_value.clone(),
            iced::widget::tooltip::Position::Top,
        );
        // Column for a main chart
        let mut chart_column;
        let mut selection_mode_row;

        // Setting selection agnostic rows and columns

        // Row for selecting the capture modes
        // Default or Running
        selection_mode_row = Row::new()
            .push(
                Column::new()
                    .push(selection_mode_title)
                    .push(mode_selection_widget)
                    .align_items(iced::Alignment::Start),
            )
            .width(Length::Fill)
            .spacing(10)
            .align_items(iced::Alignment::Center);

        // Settings row with selection of serial ports, baudrate and basic info
        let connection_settings_row = Row::new()
            .push(get_available_serial_ports_button)
            .push(
                Column::new()
                    .push(serialport_selection_widget)
                    .push(baudrate_input_widget)
                    .spacing(5),
            )
            .width(Length::Fill)
            .spacing(10)
            .align_items(iced::Alignment::Center);

        // Displaying info about connection
        let connection_info_row: Row<Message> = Row::new()
            .push(
                Column::new()
                    .push(
                        Row::new()
                            .push(baud_rate_text_info_widget)
                            .push(serial_port_text_info_widget)
                            .spacing(15),
                    )
                    .width(Length::Fill),
            )
            .push(
                Column::new()
                    .push(desktop_selection_title.clone())
                    .push(desktop_selection_widget)
                    .spacing(10)
                    .width(DESKTOP_SELECTION_COLUMN_WIDTH),
            )
            .spacing(10)
            .align_items(iced::Alignment::Center);

        // Generating rows and columns based on the capture mode selected
        match &self.display_mode {
            Some(display_mode) if display_mode == "running" => {
                // Buffer with progress bar Row
                let buffer_row = Row::new()
                    .push(text_progress_bar)
                    .push(running_buffer_input_widget)
                    .push(running_buffer_size_text_info_widget)
                    .push(text_progress_bar_value)
                    .height(iced::Length::Shrink)
                    .spacing(15)
                    .align_items(iced::Alignment::Center);

                // Chart column when the runnign mode is used
                chart_column = Column::new()
                    //.push(text_data_time)
                    .push(buffer_row)
                    .push(running_mode_progress_bar_widget)
                    .push(gui::DefaultChart::view(DefaultChart::new(
                        self.new_data_points.clone(),
                        self.data.clone(),
                        self.theme.clone(),
                    )))
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .align_items(iced::Alignment::Start)
                    .spacing(10);
            }
            Some(display_mode) if display_mode == "default" => {
                // Only chart column without progress bar is used when the default mode is
                // selected
                chart_column = Column::new()
                    //       .push(text_data_time)
                    .push(gui::DefaultChart::view(DefaultChart::new(
                        self.new_data_points.clone(),
                        self.data.clone(),
                        self.theme.clone(),
                    )))
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .align_items(iced::Alignment::Start)
                    .spacing(10);
            }
            _ => {
                // Handle other cases or provide a default behavior
                chart_column = Column::new();
            }
        }

        let mut left_column = Column::new();

        let mut content;

        // Selecting Layout based on a desktop type
        match &self.navigation_layout.selected_desktop {
            Some(display_mode) if display_mode == "default" => {
                // Left control column in a default screen
                left_column = Column::new()
                    .spacing(10)
                    .push(connection_settings_row)
                    .push(initialize_rtm_button)
                    .push(start_serial_data_fetch_button)
                    .push(stop_serial_data_fetch_button)
                    .push(clear_chart_button)
                    .push(create_output_file_button)
                    .push(start_output_file_button)
                    .push(stop_output_file_button)
                    .push(iced::widget::Space::new(0, 10))
                    .push(selection_mode_row)
                    .push(data_serialized_text_widget)
                    .push(x_axis_data_widget.clone())
                    .push(data_serialized_widget)
                    .width(350);

                // Creating a content from all defined parts to be used later in a display mode screen
                content = Column::new()
                    .push(connection_info_row)
                    .push(
                        Row::new()
                            .spacing(5)
                            .push(left_column)
                            .push(chart_column)
                            .width(Length::Fill)
                            .height(Length::Fill),
                    )
                    .width(Length::Fill)
                    .height(Length::Fill);
            }
            Some(display_mode) if display_mode == "debug" => {
                // Left control column in a debug screen
                left_column = Column::new()
                    .spacing(10)
                    .push(connection_settings_row)
                    .push(initialize_rtm_button)
                    .push(hello_button)
                    .push(hello_button_drop)
                    .push(start_task_button)
                    .push(stop_task_button)
                    .push(initialize_the_serialport_button)
                    .push(drop_the_serialport_button)
                    .push(initialize_watch_channel_button)
                    .push(drop_watch_channel_button)
                    .push(initialize_serial_data_fetch_button)
                    .push(start_serial_data_fetch_button)
                    .push(stop_serial_data_fetch_button)
                    .push(drop_serial_data_fetch_button)
                    .push(clear_chart_button)
                    .push(theme_widget)
                    .push(tooltip_test)
                    .push(create_output_file_button)
                    .push(start_output_file_button)
                    .push(stop_output_file_button)
                    .push(selection_mode_row)
                    .push(custom_button_from_out)
                    .push(data_serialized_text_widget)
                    .push(x_axis_data_widget.clone())
                    .push(data_serialized_widget)
                    .width(350);

                // Creating a content from all defined parts to be used later in a display mode screen
                content = Column::new()
                    .push(connection_info_row)
                    .push(
                        Row::new()
                            .spacing(5)
                            .push(left_column)
                            .push(chart_column)
                            .width(Length::Fill),
                    )
                    .width(Length::Fill);
            }
            Some(display_mode) if display_mode == "debug free" => {
                // Content which displays only desktop selection widget without any content, good
                // for using for showing the debug info via F12
                content = Column::new().push(
                    Row::new()
                        .push(Column::new().width(Length::FillPortion(5)))
                        .push(Column::new().width(Length::FillPortion(2)))
                        .push(
                            Column::new()
                                .push(
                                    Column::new()
                                        .push(desktop_selection_title)
                                        .push(desktop_selection_widget_debug_free)
                                        .spacing(10)
                                        .width(DESKTOP_SELECTION_COLUMN_WIDTH),
                                )
                                .align_items(Alignment::End),
                        )
                        .push(Row::new().height(Length::Fill)),
                );
            }

            _ => {
                content =
                    Column::new().push(Row::new().spacing(15).push(left_column).push(chart_column));
            }
        }

        // Final container
        Container::new(content)
            .padding(10)
            .center_x()
            .center_y()
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn theme(&self) -> iced::Theme {
        //iced::Theme::TokyoNightStorm.into()
        self.theme.clone()
    }
}

//// Default chart struct used for inputs
//pub struct DefaultChart {
//    pub data: (VecDeque<u128>, HashMap<u128, VecDeque<f64>>),
//    pub json_data: serde_json::Value,
//    pub theme: Theme,
//}
//
//
//// Implementing the chart with builder for used struct - defining displayed chart
//impl Chart<Message> for DefaultChart {
//    type State = ();
//    fn build_chart<DB: DrawingBackend>(&self, _state: &Self::State, mut builder: ChartBuilder<DB>) {
//        let mut newest_time: u128;
//        let mut oldest_time: u128 = 0;
//
//        if let Some(newest_time_data) = self.data.0.back().map(|&x| x) {
//            newest_time = newest_time_data;
//        } else {
//            //newest_time = self.new_data_points.0[self.new_data_points.0.len()];
//            newest_time = 0;
//        }
//
//        if let Some(oldest_time_iter) = self
//            .data
//            .0
//            .iter()
//            .map(|&x| x)
//            .min_by(|a, b| a.partial_cmp(b).unwrap())
//        {
//            oldest_time = oldest_time_iter;
//        }
//
//        if let Some(newest_time_iter) = self
//            .data
//            .0
//            .iter()
//            .map(|&x| x)
//            .max_by(|a, b| a.partial_cmp(b).unwrap())
//        {
//            newest_time = newest_time_iter;
//        }
//
//        // Used for dynamic axis - would need to change the variable names to correspond with the
//        // newest version
//        // otherwise when using Q32.15
//        // the min max would be
//        // -65536.0 131072.0
//        // Finding max y value from tuples
//
//        //        let max_y = match self
//        //            .new_data_points.0
//        //            .iter()
//        //            .map(|& y| y)
//        //            .max_by(|a, b| a.partial_cmp(b).unwrap())
//        //        {
//        //            Some(v) => v,
//        //            None => 0.0,
//        //        };
//        //
//        //        let min_y = match self
//        //            .new_data_points.0
//        //            .iter()
//        //            .map(|& y| y)
//        //            .min_by(|a, b| a.partial_cmp(b).unwrap())
//        //        {
//        //            Some(v) => v,
//        //            None => 0.0,
//        //        };
//        //
//
//        let mut bold_line_style_color: RGBAColor;
//        let mut light_line_style_color: RGBAColor;
//        let mut axis_style_color: RGBAColor;
//        let mut label_style_color: RGBAColor;
//
//        if self.theme == theme::Theme::Light {
//            bold_line_style_color = plotters::style::colors::BLACK.mix(0.8);
//            light_line_style_color = plotters::style::colors::BLACK.mix(0.4);
//            axis_style_color = plotters::style::colors::BLACK.mix(0.8);
//            label_style_color = plotters::style::colors::BLACK.mix(1.0);
//        } else {
//            bold_line_style_color = plotters::style::colors::WHITE.mix(0.1);
//            light_line_style_color = plotters::style::colors::WHITE.mix(0.05);
//            axis_style_color = plotters::style::colors::WHITE.mix(0.45);
//            label_style_color = plotters::style::colors::WHITE.mix(1.0);
//        }
//
//        let mut chart = builder
//            .x_label_area_size(55)
//            .y_label_area_size(40)
//            .margin(25)
//            .build_cartesian_2d(oldest_time..newest_time, -100000.0..100000.0)
//            .expect("failed to build chart");
//
//        chart
//            .configure_mesh()
//            .bold_line_style(bold_line_style_color)
//            .light_line_style(light_line_style_color)
//            .axis_style(ShapeStyle::from(axis_style_color).stroke_width(1))
//            .y_labels(10)
//            .x_labels(10)
//            .y_label_style(
//                ("sans-serif", 15)
//                    .into_font()
//                    .color(&label_style_color)
//                    .transform(FontTransform::Rotate90),
//            )
//            .y_label_formatter(&|y: &f64| format!("{}", y))
//            .x_label_style(("sans-serif", 15).into_font().color(&label_style_color))
//            .x_label_formatter(&|x| format!("{}", x))
//            .x_desc("Sample")
//            //.y_desc("Here's the label for Y")
//            .draw()
//            .expect("failed to draw chart mesh");
//
//        if let Some(obj) = self.json_data.as_object() {
//            let mut index_chart = 1;
//            let mut index_chart_color = 1;
//            for (key, _value) in obj {
//                chart
//                    .draw_series(
//                        LineSeries::new(
//                            self.data.0.iter().enumerate().filter_map(|(index, &x)| {
//                                // Get the value from the hashmap at the selected index
//                                if let Some(y_values) = self.data.1.get(&index_chart) {
//                                    // Ensure that the index is within bounds of the y_values vector
//                                    if index < y_values.len() {
//                                        Some((x as u128, y_values[index]))
//                                    } else {
//                                        None
//                                    }
//                                } else {
//                                    None
//                                }
//                            }),
//                            // 0.,
//                            // &Palette99::pick(index_chart_color).mix(0.),
//                            gui::get_line_series_style(index_chart_color),
//                        ), // .border_style(ShapeStyle::from(Palette99::pick(index_chart_color)).stroke_width(2)),
//                           //.border_style(&Palette99::pick(index_chart_color)),
//                    )
//                    .expect("failed to draw chart data")
//                    .label(key.to_string())
//                    .legend(move |(x, y)| {
//                        Rectangle::new(
//                            [(x - 1, y - 1), (x + 15, y)],
//                            &Palette99::pick(index_chart_color as usize),
//                        )
//                    });
//
//                index_chart = index_chart + 1;
//                index_chart_color = index_chart_color + 1;
//            }
//        }
//
//        //        chart
//        //            .draw_series(
//        //                AreaSeries::new(
//        //                    //self.data_points.iter().map(|x| (x.0, x.1)),
//        //                    self.new_data_points
//        //                        .0
//        //                        .iter()
//        //                        .enumerate()
//        //                        .filter_map(|(index, &x)| {
//        //                            // Get the value from the hashmap at index 1
//        //                            if let Some(y_values) = self.new_data_points.1.get(&1) {
//        //                                // Ensure that the index is within bounds of the y_values vector
//        //                                if index < y_values.len() {
//        //                                    Some((x as u128, y_values[index]+200.0))
//        //                                } else {
//        //                                    None
//        //                                }
//        //                            } else {
//        //                                None
//        //                            }
//        //                        }),
//        //                    0.,
//        //                    PLOT_LINE_COLOR_2.mix(0.175),
//        //                )
//        //                .border_style(ShapeStyle::from(PLOT_LINE_COLOR_2).stroke_width(2)),
//        //            )
//        //            .expect("failed to draw chart data")
//        //            .label("Var 2")
//        //            .legend(|(x, y)| {
//        //                // Increase the x-coordinate to add space next to the rectangle
//        //                Rectangle::new([(x - 1, y + 1), (x + 15, y)], PLOT_LINE_COLOR_2)
//        //            });
//
//        chart
//            .configure_series_labels()
//            .background_style(&WHITE.mix(0.8))
//            .border_style(&BLACK)
//            .legend_area_size(25)
//            .margin(10)
//            .position(SeriesLabelPosition::UpperLeft)
//            .draw()
//            .expect("Failed to draw a series label.");
//    }
//}
//
//impl DefaultChart {
//    pub fn new(
//        data: (VecDeque<u128>, HashMap<u128, VecDeque<f64>>),
//        json_data: serde_json::Value,
//        theme: Theme,
//    ) -> Self {
//        Self {
//            data,
//            json_data,
//            theme,
//        }
//    }
//    fn view(self) -> Element<'static, Message> {
//        ChartWidget::new(self).into()
//    }
//}

struct EdtTheme;
impl button::StyleSheet for EdtTheme {
    type Style = iced::Theme;
    fn active(&self, style: &Self::Style) -> button::Appearance {
        // `match` on `style` if you want to change anything for different themes.
        button::Appearance {
            text_color: iced::Color::from_rgba(1.0, 1.0, 1.0, 1.0),
            background: Some(iced::Background::Color(iced::Color {
                r: 0.12,
                g: 0.65,
                b: 1.0,
                a: 1.2,
            })),
            border: iced::Border {
                color: iced::Color {
                    r: 0.12,
                    g: 0.65,
                    b: 1.0,
                    a: 1.0,
                },
                width: 1.0,
                radius: 3.0.into(),
            },
            shadow: iced::Shadow {
                color: iced::Color {
                    r: 0.0,
                    g: 0.0,
                    b: 0.0,
                    a: 1.0,
                },
                offset: iced::Vector { x: 0.0, y: 0.0 },
                blur_radius: 0.0,
            },
            ..Default::default()
        }
    }
    fn hovered(&self, style: &Self::Style) -> button::Appearance {
        button::Appearance {
            text_color: iced::Color::from_rgba(1.0, 1.0, 1.0, 1.0),
            background: Some(iced::Background::Color(iced::Color {
                r: 0.82,
                g: 0.25,
                b: 0.26,
                a: 1.2,
            })),
            border: iced::Border {
                color: iced::Color {
                    r: 0.12,
                    g: 0.65,
                    b: 1.0,
                    a: 1.0,
                },
                width: 1.0,
                radius: 12.0.into(),
            },
            shadow: iced::Shadow {
                color: iced::Color {
                    r: 0.0,
                    g: 0.0,
                    b: 0.0,
                    a: 1.0,
                },
                offset: iced::Vector { x: 0.0, y: 0.0 },
                blur_radius: 0.0,
            },
            ..Default::default()
        }
    }
}
