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

use std::{collections::{HashMap, VecDeque}, fs::File};

use iced::{theme, Element, Theme};
use plotters::{chart::SeriesLabelPosition, element::Rectangle, series::LineSeries, style::{Color, FontTransform, IntoFont, Palette, Palette99, RGBAColor, ShapeStyle, BLACK, WHITE}};
use plotters_iced::{Chart, ChartBuilder, ChartWidget, DrawingBackend};

use crate::{com_core::SerialPortConnectInfo, Message};

// Gives back result of ShapeStyle which is then used in a LineSeries chart
// The color is automatically generated from a Palette based on a chart index
// Used for generating unique styles for plotted lines
pub fn get_line_series_style(index: u32) -> ShapeStyle {
    let new_style = ShapeStyle {
        color: Palette99::pick(index as usize).to_rgba(),
        filled: false,
        stroke_width: 2,
    };

    new_style
}

// Running buffer struct for iced widget input
#[derive(Default, Clone)]
pub struct RunningBuffer {
    pub size: u64,              // Parsed size of a running buffer
    pub size_string: String,    // String format of the size of a running buffer
}

// Selecting desktops struct for iced PickList
#[derive(Default)]
pub struct NavigationLayout {
    pub available_desktops: Vec<String>,    // Vector of strings of available desktops
    pub selected_desktop: Option<String>,   // Currently selected desktop from the PickList
}

// Struct for handling writing data to file output
#[derive(Default)]
pub struct OutputFile {
    pub handler: Option<File>,  // For opened file hadler to be able to access write operation
                                // between cycles of data process
    pub enable_output: bool,    // For enabling and disabling the output to file
}

// Struct for Serial Port settings
#[derive(Debug, Clone)]
pub struct SerialPortData {
    pub structure: Vec<SerialPortConnectInfo>,  // Contains data of all serial ports
    pub names: Vec<String>,                     // Contains only names of serial ports for
                                                // displaying the the PickList
    pub selected: String,                       // Selected serial port value to be passed to
                                                // initializing function for serial port
    pub baud_rate: u32,                         // Parsed baud rate in u32 format for opening
                                                // serial port connection
    pub baud_rate_string: String,               // String format of baud rate inseted by used
}




// Default chart struct used for inputs
pub struct DefaultChart {
    pub data: (VecDeque<u128>, HashMap<u128, VecDeque<f64>>),
    pub json_data: serde_json::Value,
    pub theme: Theme,
}


// Implementing the chart with builder for used struct - defining displayed chart
impl Chart<Message> for DefaultChart {
    type State = ();
    fn build_chart<DB: DrawingBackend>(&self, _state: &Self::State, mut builder: ChartBuilder<DB>) {
        let mut newest_time: u128;
        let mut oldest_time: u128 = 0;

        if let Some(newest_time_data) = self.data.0.back().map(|&x| x) {
            newest_time = newest_time_data;
        } else {
            //newest_time = self.new_data_points.0[self.new_data_points.0.len()];
            newest_time = 0;
        }

        if let Some(oldest_time_iter) = self
            .data
            .0
            .iter()
            .map(|&x| x)
            .min_by(|a, b| a.partial_cmp(b).unwrap())
        {
            oldest_time = oldest_time_iter;
        }

        if let Some(newest_time_iter) = self
            .data
            .0
            .iter()
            .map(|&x| x)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
        {
            newest_time = newest_time_iter;
        }

        // Used for dynamic axis - would need to change the variable names to correspond with the
        // newest version
        // otherwise when using Q32.15
        // the min max would be
        // -65536.0 131072.0
        // Finding max y value from tuples

        //        let max_y = match self
        //            .new_data_points.0
        //            .iter()
        //            .map(|& y| y)
        //            .max_by(|a, b| a.partial_cmp(b).unwrap())
        //        {
        //            Some(v) => v,
        //            None => 0.0,
        //        };
        //
        //        let min_y = match self
        //            .new_data_points.0
        //            .iter()
        //            .map(|& y| y)
        //            .min_by(|a, b| a.partial_cmp(b).unwrap())
        //        {
        //            Some(v) => v,
        //            None => 0.0,
        //        };
        //

        let mut bold_line_style_color: RGBAColor;
        let mut light_line_style_color: RGBAColor;
        let mut axis_style_color: RGBAColor;
        let mut label_style_color: RGBAColor;

        if self.theme == theme::Theme::Light {
            bold_line_style_color = plotters::style::colors::BLACK.mix(0.8);
            light_line_style_color = plotters::style::colors::BLACK.mix(0.4);
            axis_style_color = plotters::style::colors::BLACK.mix(0.8);
            label_style_color = plotters::style::colors::BLACK.mix(1.0);
        } else {
            bold_line_style_color = plotters::style::colors::WHITE.mix(0.1);
            light_line_style_color = plotters::style::colors::WHITE.mix(0.05);
            axis_style_color = plotters::style::colors::WHITE.mix(0.45);
            label_style_color = plotters::style::colors::WHITE.mix(1.0);
        }

        let mut chart = builder
            .x_label_area_size(55)
            .y_label_area_size(40)
            .margin(25)
            .build_cartesian_2d(oldest_time..newest_time, -100000.0..100000.0)
            .expect("failed to build chart");

        chart
            .configure_mesh()
            .bold_line_style(bold_line_style_color)
            .light_line_style(light_line_style_color)
            .axis_style(ShapeStyle::from(axis_style_color).stroke_width(1))
            .y_labels(10)
            .x_labels(10)
            .y_label_style(
                ("sans-serif", 15)
                    .into_font()
                    .color(&label_style_color)
                    .transform(FontTransform::Rotate90),
            )
            .y_label_formatter(&|y: &f64| format!("{}", y))
            .x_label_style(("sans-serif", 15).into_font().color(&label_style_color))
            .x_label_formatter(&|x| format!("{}", x))
            .x_desc("Sample")
            //.y_desc("Here's the label for Y")
            .draw()
            .expect("failed to draw chart mesh");

        if let Some(obj) = self.json_data.as_object() {
            let mut index_chart = 1;
            let mut index_chart_color = 1;
            for (key, _value) in obj {
                chart
                    .draw_series(
                        LineSeries::new(
                            self.data.0.iter().enumerate().filter_map(|(index, &x)| {
                                // Get the value from the hashmap at the selected index
                                if let Some(y_values) = self.data.1.get(&index_chart) {
                                    // Ensure that the index is within bounds of the y_values vector
                                    if index < y_values.len() {
                                        Some((x as u128, y_values[index]))
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                }
                            }),
                            // 0.,
                            // &Palette99::pick(index_chart_color).mix(0.),
                            get_line_series_style(index_chart_color),
                        ), // .border_style(ShapeStyle::from(Palette99::pick(index_chart_color)).stroke_width(2)),
                           //.border_style(&Palette99::pick(index_chart_color)),
                    )
                    .expect("failed to draw chart data")
                    .label(key.to_string())
                    .legend(move |(x, y)| {
                        Rectangle::new(
                            [(x - 1, y - 1), (x + 15, y)],
                            &Palette99::pick(index_chart_color as usize),
                        )
                    });

                index_chart = index_chart + 1;
                index_chart_color = index_chart_color + 1;
            }
        }

        //        chart
        //            .draw_series(
        //                AreaSeries::new(
        //                    //self.data_points.iter().map(|x| (x.0, x.1)),
        //                    self.new_data_points
        //                        .0
        //                        .iter()
        //                        .enumerate()
        //                        .filter_map(|(index, &x)| {
        //                            // Get the value from the hashmap at index 1
        //                            if let Some(y_values) = self.new_data_points.1.get(&1) {
        //                                // Ensure that the index is within bounds of the y_values vector
        //                                if index < y_values.len() {
        //                                    Some((x as u128, y_values[index]+200.0))
        //                                } else {
        //                                    None
        //                                }
        //                            } else {
        //                                None
        //                            }
        //                        }),
        //                    0.,
        //                    PLOT_LINE_COLOR_2.mix(0.175),
        //                )
        //                .border_style(ShapeStyle::from(PLOT_LINE_COLOR_2).stroke_width(2)),
        //            )
        //            .expect("failed to draw chart data")
        //            .label("Var 2")
        //            .legend(|(x, y)| {
        //                // Increase the x-coordinate to add space next to the rectangle
        //                Rectangle::new([(x - 1, y + 1), (x + 15, y)], PLOT_LINE_COLOR_2)
        //            });

        chart
            .configure_series_labels()
            .background_style(&WHITE.mix(0.8))
            .border_style(&BLACK)
            .legend_area_size(25)
            .margin(10)
            .position(SeriesLabelPosition::UpperLeft)
            .draw()
            .expect("Failed to draw a series label.");
    }
}

impl DefaultChart {
    pub fn new(
        data: (VecDeque<u128>, HashMap<u128, VecDeque<f64>>),
        json_data: serde_json::Value,
        theme: Theme,
    ) -> Self {
        Self {
            data,
            json_data,
            theme,
        }
    }
    pub fn view(self) -> Element<'static, Message> {
        ChartWidget::new(self).into()
    }
}
