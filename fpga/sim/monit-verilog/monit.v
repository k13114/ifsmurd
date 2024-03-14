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

module monit(clk, startTx, inputRx, outputTx, led1,
rstBuffer);
reg [31:0] testik;
initial
begin
    testik = 32'h80000000;
end
// clk - clock signal from a board or a PLL
// startTx - signal from a board to start the state machine for sending
// messages - for tersting purposes, normally the data sending procedure is
// invoked with some period - will have to implement
input clk, startTx;

// Input from a serial port = input to Rx
input inputRx; // when in another module - update the module inputs
wire  inputRxInternal; // for testing purposes
assign inputRxInternal = outputTx; // for testing purposes used as a loopback

// Output of Tx procedure = serial port
output outputTx;

// The Tx procedure is done
wire doneTx;

// The received Byte from a Rx procedure
wire [7:0] outputRx;

// Info that the Rx procedure is completed
wire doneRx;


wire startTxNeg;
// assign startTxNeg = ~ startTx; // when using GOWIN Tang nano 9 k
assign startTxNeg = startTx; // when using cocotb or any board without pull up rezistors, or where the normal state is of and not on as in Tang nano 9k/1k

// when using this state machine, the GOWIN tang nano is used
// the rst must be set as an input to the module with corresponding signal
// all things must be uncommented down there
// the startTxNeg with ~ startTx must be used
//parameter [2:0] S0 = 0, S1 = 1, S2 = 2;
//reg [2:0] state;
//
//always@(posedge clk)
//begin
//    case(state)
//        S0:
//         begin
//            startTxNeg <= 0;
//            if(~ startTx)
//            begin
//                state <= S1;  
//            end
//          end
//        S1:
//        begin
//            startTxNeg <= 1;
//            if(~ rst)
//            begin
//                state <= S0;
//            end
//         end
//        default:
//        begin
//            state <= S0;
//        end
//    endcase
//end

// This parameters will be passed to a single module
// Length of a single variable send in a message
parameter [7:0] VARIABLE_LENGTH = 4*8;
// How many variables will be send in a message
parameter [7:0] NUMBER_OF_VARIABLES = 5*2;

// Frequency of used FPGA clock to drive the Tx and Rx units divided by
// baudrate of UART
// clkFreq/baudrate
// for baudrate 115200
// for 100 MHz = 868
// for 27 MHz (Tang Nano 9k/1k GOWIN FPGA) = 234
// for 27 MHz and baudrate 1500000 = 18
parameter CLKS_PER_BIT = 868;

// Section of code which is a synthetizable solution for working with
// "memories" aka registers which have some depth as displayed bellow for the
// monitConnector reg which consists of regs VARIABLE_LENGHT wide and the
// array is deep NUMBER_OF_VARIABLES
// this workaroud is working, because the comUnit sends out a signal, which
// variableIndex and which singleVariablePartIndex is needed in the FSM of
// a connection module, based on that, the corresponding section of
// monitConnector is selected and assigned to a wire of
// senderConnectorInternal which is connected to the pin of comUnit sender
// connector
wire [7:0] senderConnectorInternal;                 // Connector from an array of variables to be send to the communication block to the communication block
wire [7:0] singleVariablePartIndex;                 // Index moving through parts of the variable which is being sent when the length is wider than 8 bits
wire [7:0] variableIndex;                           // Index which is used to point to the variable which should be send from the array to the communication unit to the uart Tx
reg [VARIABLE_LENGTH-1:0] numberOfVariables;        // Number of variables to be send in the UART protocol, must correspond to NUMBER_OF_VARIABLES const but in a format for UART
                                                    // if the NUMBER_OF_VARIABLES is used in the UART instead, it will be (without changing the algorithm of the sending machine) sending the same bytes over and over as defined in the LSB

initial begin
    numberOfVariables=32'h0000000A;
end

assign senderConnectorInternal = monitConnector[variableIndex][singleVariablePartIndex+:8];

// Connector/Interface (rebrand later) which is a 2D array which has connected
// variables to it, which will be send via uart in a message
// This array is connected to the senderConnectorInternal wire, which is then
// routed to the communication unit to be send via Tx by uart
// This array is not directly connected to the module pin of connection unit,
// it works in a simulation, however is not synthetizable and the workaround
// using external whire which is routed to the module must be used
// found that somewhere on the internet and using textbooks
// some reference why:
// https://electronics.stackexchange.com/questions/127164/what-is-wrong-with-following-verilog-code-where-i-am-trying-to-pass-a-one-dimens
//  -----------
// | variable1 |
// | variable2 |
// | variable3 |
// | ......... |
// | variable_n|
//  -----------
wire [VARIABLE_LENGTH-1:0] monitConnector [NUMBER_OF_VARIABLES-1:0];
// Maybe later change the reg to the wire and connect different variables to
// be send to corresponding wires/parts
// however the workaround using the wire connected to the connection unit pin
// must be preserved


// The communication unit which is used to send the starting part of the
// message, the data and the ending part of the message
comUnit #(.VARIABLE_LENGTH(VARIABLE_LENGTH), .NUMBER_OF_VARIABLES(NUMBER_OF_VARIABLES), .CLKS_PER_BIT(CLKS_PER_BIT)) comUnitModule (.clk(clk), .variableIndex(variableIndex), .singleVariablePartIndex(singleVariablePartIndex) , .senderConnector(senderConnectorInternal), .startComm(startTxNeg), .doneTx(doneTx), .doneRx(doneRx), .outputRx(outputRx), .outputTx(outputTx), .inputRx(inputRxInternal), .numberOfVariables(numberOfVariables));

reg [VARIABLE_LENGTH-1:0] dataRxBuffer = 0;
output reg led1;
reg [7:0] inputVariableIndex = VARIABLE_LENGTH-1;

always@(posedge clk)
begin
    // If one byte has been successfully received
    if(doneRx)
    begin
        // Push the data to the buffer
        // The bytes are send but the message must fill the buffer from LSB
        // (here least significant byte not bit) or
        // MSB, in this module, the buffer is filled from the MSB (here most
        // significant byte, not bit) (top) to
        // bottom, otherwise the shifting of the received LSB to the left
        // would have to be done every time new data byte is received
        dataRxBuffer[inputVariableIndex-:8] <= outputRx;
        inputVariableIndex <= inputVariableIndex - 8;
    end

    // When the overflow of the index happend when shifting through the buffer
    // based on doneRx singal
    // The buffer index is againt set to the max to be later subtracted
    if(inputVariableIndex == 8'hFF)
    begin
        inputVariableIndex <= VARIABLE_LENGTH-1;
    end

    // Example logic for testing the received data and making the LED shine
    // or not
    // Buffer for 4x l = 23'h6C6C6C6C
    // Buffer for example data = 32'h2F2F0D0A
if(dataRxBuffer == 32'h6C6C6C6C)
    begin
        // When tang nano 9k/1k with GOWIN FPGA is used
        led1 <= 0;
    end

    // Buffer for 4 x # = 32'h23232323
    // Buffer for example data = 32'h5C5C0D0A
    if(dataRxBuffer == 32'h23232323)
    begin
        // When tang nano 9k/1k with GOWIN FPGA is used
        led1 <= 1;
    end

end

// Timers for test output values generation
reg [31:0] testingCounter, testingCounter2;

initial
begin
    testingCounter = 0;
    testingCounter2 = 0;
end

always@(posedge clk)
begin
    testingCounter <= testingCounter + 1'b1;
    testingCounter2 <= testingCounter2 - 1'b1;
end


// Section dedidacted for a buffer/memory
// which periodically saves the values from defined input variables
// to the monitConnectorMemory which is used to send the data to the comUnit


reg [31:0] timingCounter; // Timer for sampling the analysed variables
parameter RESET_TIMING_COUNTER = 255; // 6920, 7488 // (1 + 1 + NUMBER_OF_VARIABLES + 2 + 1) * VARIABLE_LENGTH * CLKS_PER_BIT

initial
begin
    timingCounter = 0;
end


// resetTimingCounter = period of variables sampling / FPGA clock period
// example:
// want to sample every 2550 ns: period of variables sampling = 2550
// FPGA clock cycle period = 10 ns
// resetTimingCounter = 2550/10 = 255
always@(posedge clk)
begin
    timingCounter <= timingCounter + 1'b1;
    // If not RESET_TIMING_COUNTER + 1 is used, the value which is set to
    // memory is value which was present in the monitConnector at
    // RESET_TIMING_COUNTER - 1, because the monitConnector (when reg) is also
    // updated with the clock, so the values are passed to the monitConnector
    // from the values (eg. test time) at the end of the each clock cycle, so
    // when there is passed value which correspons to RESET_TIMING_COUNTER,
    // the value is not passed to monitConnectionMemory because at the same
    // clock cycle, the current value of monitConnector is passed to the
    // memory, so the RESET_TIMING_COUNTER - 1 is passed
    // there is a tradeof:
    // 1. the values will be passed to memory at a defined
    // RESET_TIMING_COUNTER regardless if the value passing is the value
    // present at the monitConnector at the selected time
    // 2. Or RESET_TIMING_COUNTER + 1 is used and the values are to the memory
    // passed one cycle later than requested and the values corresponding to
    // RESET_TIMING_CONSTANT are passed from monitConnector to the
    // monitConnectorMemory
    //
    // If the monitConnector is wire, the values wire would be driven at all
    // times regardless the clock and the values from monitConnector to the
    // monitConnectorMemory are passed with values which are at the time of
    // RESET_TIMING_COUNTER

    if(timingCounter == RESET_TIMING_COUNTER)
    begin
        timingCounter <= 0;
    end

    // Handling when to write values to the buffers
    if(timingCounter == RESET_TIMING_COUNTER-1)
    begin
        wr_enable <= 1;
    end
    else
    begin
        wr_enable <= 0;
    end


    // Handling when to get the values at rd_ptr from buffers
    // now it is set that when it it done sending the message
    if(doneTx)
    begin
        rd_enable <= 1;
    end
    // Need to stop reading another value after new value was read
    if(doneTx && rd_enable)
    begin
        rd_enable <= 0;
    end
end

       // by variables for bytes and variables by senderConnector
       assign monitConnector[0] = 32'h40303030; // @varId = @0x000000
       assign monitConnector[1] = buf_out_1;
       assign monitConnector[2] = 32'h40303031; // @varId = @0x000001
       assign monitConnector[3] = buf_out_3;
       assign monitConnector[4] = 32'h40303032; // @varId = @0x000002
       assign monitConnector[5] = buf_out_5;
       assign monitConnector[6] = 32'h40303033; // @varId = @0x000003
       assign monitConnector[7] = buf_out_7;
       assign monitConnector[8] = 32'h40303034; // @varId = @0x000004
       assign monitConnector[9] = buf_out_9;

       // Connecting the analysed variables to the buffer inputs
       assign buf_in_1 = testingCounter;   // value
       assign buf_in_3 = testingCounter2;   // value
       assign buf_in_5 = 2*testingCounter+32'h00FF34FF;   // value
       assign buf_in_7 = 10*testingCounter2+32'hFFFF00FF;   // value
       assign buf_in_9 = 7*testingCounter2+32'hFFAB00FF;   // value

// Buffer section

input rstBuffer; // Reset the buffers
reg wr_enable, rd_enable; // Enable writing and reading to buffers, the writing is at a sampled time as specified by user and reading is when the uart message is sent

wire [31:0] buf_in_1, buf_in_3, buf_in_5, buf_in_7, buf_in_9; // Buffer input - analysed values
wire [31:0] buf_out_1, buf_out_3, buf_out_5, buf_out_7, buf_out_9; // Buffer output - values retrieved when reading from the buffer

dataBuffer #() dataBuffer1 (.clk(clk), .rst(rstBuffer), .wr_enable(wr_enable), .rd_enable(rd_enable), .buf_in(buf_in_1), .buf_out(buf_out_1));
dataBuffer #() dataBuffer3 (.clk(clk), .rst(rstBuffer), .wr_enable(wr_enable), .rd_enable(rd_enable), .buf_in(buf_in_3), .buf_out(buf_out_3));
dataBuffer #() dataBuffer5 (.clk(clk), .rst(rstBuffer), .wr_enable(wr_enable), .rd_enable(rd_enable), .buf_in(buf_in_5), .buf_out(buf_out_5));
dataBuffer #() dataBuffer7 (.clk(clk), .rst(rstBuffer), .wr_enable(wr_enable), .rd_enable(rd_enable), .buf_in(buf_in_7), .buf_out(buf_out_7));
dataBuffer #() dataBuffer9 (.clk(clk), .rst(rstBuffer), .wr_enable(wr_enable), .rd_enable(rd_enable), .buf_in(buf_in_9), .buf_out(buf_out_9));

endmodule
