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

module comUnit #(parameter VARIABLE_LENGTH = 4*8, parameter NUMBER_OF_VARIABLES = 5, parameter CLKS_PER_BIT = 234) (clk, startComm,
     doneTx, senderConnector,
    outputTx, inputRx, outputRx, doneRx,
    singleVariablePartIndex, variableIndex, numberOfVariables);


input clk;                          // Clock signal
input startComm;                    // Control bit from a parent module to start the communication
reg [7:0] inputTxByte;              // Byte passed to the Tx unit

reg startTxInternal;                // Initiate the Tx


output outputTx;                    // Output from a Tx Unit - Serial Comm
output doneTx;                      // Tx byte sent
input inputRx;                      // Input from a Serial Comm to the Rx unit
output [7:0] outputRx;              // Formatted byte from 8 bits
output doneRx;                      // Successfully received the byte

// This will be moved to a single module
// Main state for message sending state machine
// maybe rebrand the name of the states later, to be more explaining
reg [3:0] state;
parameter [3:0] S0 = 0, S1 = 1, S2 = 2, S3 = 3, S4 = 4, S5 = 5, S6 = 6, S7 = 7, S8 = 8, S9 = 9, S10 = 10, S11 = 11, S12 = 12;

// This parameters will be passed to a single module
// Length of a single variable send in a message
//parameter [7:0] VARIABLE_LENGTH = 4*8;
// How many variables will be send in a message
//parameter [7:0] NUMBER_OF_VARIABLES = 5;

// Connector/Interface (rebrand later) which is a 2D array which has connected
// variables to it, which will be send via uart in a message
//  -----------
// | variable1 |
// | variable2 |
// | variable3 |
// | ......... |
// | variable_n|
//  -----------
//input [VARIABLE_LENGTH-1:0] senderConnector [NUMBER_OF_VARIABLES-1:0];
input [7:0] senderConnector;

// This will be moved to a single module
// Index for variables iterating through variables to be send via UART
output reg [7:0] variableIndex = 0;

// Lower index of variable which is moved by 8 bits every time when the
// variable is wider than 8 bits till all of the bytes were sent
output reg [7:0] singleVariablePartIndex;

// For space of one clock cycle after word is sent successfully
// Needed as space because UART doneTx is HIGH for two cycles
reg spaceHandler = 0;
// Space between single variable parts, used when the length of the variable
// is larger than 8 bits
reg indexSpaceHandler = 0;

// Registers for storing end of message and start of message parts
// Maybe later implement some CRC or parity or checksum
reg [VARIABLE_LENGTH-1:0] stopSequence;
reg [VARIABLE_LENGTH-1:0] startSequence;
input [VARIABLE_LENGTH-1:0] numberOfVariables;


wire [VARIABLE_LENGTH-1:0] crcOut;
wire [7:0] crcIn;
reg [7:0] crcIndex;
always@(posedge startTxInternal)
begin
    if(state == S5)
    begin
    crcIndex <= crcIn ^ inputTxByte;
    end
    if(state == S1)
    begin
    crcIndex <= 0;
    end
end
assign crcIn = crcOut;
reg [VARIABLE_LENGTH-1:0] crcId;
crcTable crcTableUnit(.index(crcIndex), .resultReturn(crcOut));
wire [VARIABLE_LENGTH-1:0] crcOutDataLength;

initial
begin
    startSequence = 32'h2F2F2F2F;
    stopSequence = 32'h5C5C5C5C;
    //numberOfVariables=32'h0000000A;
end

// The glorious state machine for sending the message to uart
// which has a structure
// | start byte | data | end byte |
always@(posedge clk)
begin
    case(state)
        S0:
        // Idle state
        begin
            if(startComm)
            begin
                // If there is a command for starting the transfer from a parent module
                state <= S1;
                // Pass the starting part of the message
                inputTxByte <= startSequence[7:0];
                singleVariablePartIndex <= 0;
                variableIndex <= 0;
                crcId <= 32'h43524340;
            end
            else
            begin
                // If no command for a communication/sending the variables
                // stay in the idle state
                state <= S0;
            end
        end

        // Sending Defined start part of the message
        S1:
        begin
                if(doneTx)
                begin
                    if(singleVariablePartIndex+7 < VARIABLE_LENGTH && indexSpaceHandler == 0)
                    begin
                        // Pass the starting part of the message
                        singleVariablePartIndex <= singleVariablePartIndex + 8;
                        indexSpaceHandler <= 1;
                        startTxInternal <= 0;
                    end
                    if(singleVariablePartIndex+7 >= VARIABLE_LENGTH && indexSpaceHandler == 1)
                    begin
                        state <= S2;
                        indexSpaceHandler <= 0;
                        startTxInternal <= 0;
                        singleVariablePartIndex <= 0;
                        inputTxByte <= numberOfVariables[7:0];
                    end
                    if(singleVariablePartIndex+7 < VARIABLE_LENGTH && indexSpaceHandler == 1)
                    begin
                        inputTxByte <= startSequence[singleVariablePartIndex+:8];
                        indexSpaceHandler <= 0;
                    end
                end
                else
                begin
                    // Start the communication
                    startTxInternal <= 1;
                    state <= S1;
                end
        end

        // Needed as space because UART doneTx is HIGH for two cycles
        S2:
        begin
            state <= S3;
            startTxInternal <= 1;
        end
    

        //  NEW - ADDING NUMBER OF VARIABLES

        // Sending Defined start part of the message
        S3:
        begin
                if(doneTx)
                begin
                    if(singleVariablePartIndex+7 < VARIABLE_LENGTH && indexSpaceHandler == 0)
                    begin
                        // Pass the starting part of the message
                        singleVariablePartIndex <= singleVariablePartIndex + 8;
                        indexSpaceHandler <= 1;
                        startTxInternal <= 0;
                    end
                    if(singleVariablePartIndex+7 >= VARIABLE_LENGTH && indexSpaceHandler == 1)
                    begin
                        state <= S4;
                        indexSpaceHandler <= 0;
                        startTxInternal <= 0;
                        singleVariablePartIndex <= 0;
                        inputTxByte <= senderConnector;
                    end
                    if(singleVariablePartIndex+7 < VARIABLE_LENGTH && indexSpaceHandler == 1)
                    begin
                        inputTxByte <= numberOfVariables[singleVariablePartIndex+:8]; // here paste number of variables
                        indexSpaceHandler <= 0;
                    end
                end
                else
                begin
                    // Start the communication
                    startTxInternal <= 1;
                    state <= S3;
                end
        end

        // Needed as space because UART doneTx is HIGH for two cycles
        S4:
        begin
            state <= S5;
            startTxInternal <= 1;
        end


        // Main data sending part of the message
        S5:
        begin
            // If there are variables which were not yet sent
            if(variableIndex < NUMBER_OF_VARIABLES)
            begin
                // Still the same state
                state <= S5;
                // If the transport of one varible is done and the
                // spaceHandler is 0 - means right after the variable was sent
                if(doneTx && spaceHandler == 0)
                begin
                    // If there are parts of the current variable which is
                    // being sent and there are some parts left
                    if(singleVariablePartIndex+7 < VARIABLE_LENGTH && indexSpaceHandler == 0)
                    begin
                        // Pass the starting part of the message
                        singleVariablePartIndex <= singleVariablePartIndex + 8;
                        // Space between indexes - between parts of the
                        // variable
                        indexSpaceHandler <= 1;
                        // Stop the transfer - it is needed otherwise the old
                        // part of the variable would be currently send - it
                        // stops the sending in when a clearing and idle
                        // state is set
                        // so when in idle state, no new transfer with old
                        // TxByteInternal is send
                        startTxInternal <= 0;
                    end
                    if(singleVariablePartIndex+7 >= VARIABLE_LENGTH && indexSpaceHandler == 1)
                    begin
                        // Stop the transfer when all the parts of a signle
                        // variable are sent
                        startTxInternal <= 0;
                        // Reset the index for the next variable to be send
                        singleVariablePartIndex <= 0;
                        // Make space for one cycle
                        spaceHandler <= 1'b1;
                        // Select next variable
                        variableIndex <= variableIndex + 1'b1;
                        // Make one clk space between the parts of the
                        // variable
                        // it should probably not be here, because this was
                        // the last part of the sending variable but because
                        // of the design, the indexSpaceHandler is set HIGH
                        // after each part of the variable, so it needs to be
                        // set LOW even when the last part of the variable was
                        // sent
                        indexSpaceHandler <= 0;
                    end
                    if(indexSpaceHandler == 1)
                    begin
                        // If it is needed to send next part of the current
                        // variable - pass to the Tx module the current
                        // variable but with shifted index by 8 bits

                        inputTxByte <= senderConnector;
                        // Needed to create space between parts of the
                        // variable where the Tx is shut down - to prevent
                        // sending old parts of the variable
                        indexSpaceHandler <= 0;
                    end
                end
                else
                begin
                    // is doneTx != 1 and spaceHandler != 0
                    // this means, that the Tx is not done or the spaceHandler
                    // between variables is set to 1

                    // In a next clock cycle, when there is a new inputTxByte
                    // set, start the Tx
                    // space between variable parts is not set, because it has
                    // been set to 0 in time when doneTx was HIGH
                    // indexSpaceHandler was set to LOW in time when
                    // doneTx was HIGH and spaceHandler is LOW so it is
                    // only a transition between parts of the variable
                    if(indexSpaceHandler == 0)
                    begin
                        startTxInternal <= 1;
                    end

                    // if there is a space between variables so new variable
                    // based on updated index (updated in time when doneTx is
                    // HIGH) must be send
                    if(spaceHandler == 1)
                    begin
                        // Send a new Byte to Tx with an updated variableIndex
                        inputTxByte <= senderConnector;
                        // Disable the space between variables
                        spaceHandler <= 1'b0;
                        // Start the transfer in a next clock cycle
                        startTxInternal <= 1;
                    end
                    // The variable was successfully sent and one cycle space
                    // has ended
                    // Send the same variable which is selected based on the
                    // right position in the message
                    // variableIndex <= variableIndex;
                end
            end
            else
            begin
                // ALL variables from the data part has been sent
                //
                //
                // Transition to a next state when there is a space of one
                // cycle beause of how Tx is designed
                state <= S6;
                spaceHandler <= 1'b0;
                // Passing the stopSequence here to be present before the
                // startTxInternal and be able to be passed to Tx in
                // TxInternal structure
                //inputTxByte <= stopSequence[7:0];
                inputTxByte <= crcId[7:0];
            end
        end

        // Needed as space because UART doneTx is HIGH for two cycles
        S6:
        begin
            state <= S7;
        end
        // CRC id
        S7:
        begin
            // Starting the transfer
            startTxInternal <= 1;
                if(doneTx)
                begin
                    if(singleVariablePartIndex+7 < VARIABLE_LENGTH && indexSpaceHandler == 0)
                    begin
                        // Pass the starting part of the message
                        singleVariablePartIndex <= singleVariablePartIndex + 8;
                        indexSpaceHandler <= 1;
                        startTxInternal <= 0;
                    end
                    if(singleVariablePartIndex+7 >= VARIABLE_LENGTH && indexSpaceHandler == 1)
                    begin
                        state <= S8;
                        indexSpaceHandler <= 0;
                        startTxInternal <= 0;
                        singleVariablePartIndex <= 0;
                        inputTxByte <= crcOut[7:0];
                    end
                    if(singleVariablePartIndex+7 < VARIABLE_LENGTH && indexSpaceHandler == 1)
                    begin
                        inputTxByte <= crcId[singleVariablePartIndex+:8];
                        indexSpaceHandler <= 0;
                    end
                end
                else
                begin
                    startTxInternal <= 1;
                    state <= S7;
                end
        end

        S8:
        begin
            state <= S9;
        end
        // CRC data
        S9:
        begin
            // Starting the transfer
            startTxInternal <= 1;
                if(doneTx)
                begin
                    if(singleVariablePartIndex+7 < VARIABLE_LENGTH && indexSpaceHandler == 0)
                    begin
                        // Pass the starting part of the message
                        singleVariablePartIndex <= singleVariablePartIndex + 8;
                        indexSpaceHandler <= 1;
                        startTxInternal <= 0;
                    end
                    if(singleVariablePartIndex+7 >= VARIABLE_LENGTH && indexSpaceHandler == 1)
                    begin
                        state <= S10;
                        indexSpaceHandler <= 0;
                        startTxInternal <= 0;
                        singleVariablePartIndex <= 0;
                        inputTxByte <= stopSequence[7:0];
                    end
                    if(singleVariablePartIndex+7 < VARIABLE_LENGTH && indexSpaceHandler == 1)
                    begin
                        inputTxByte <= crcOut[singleVariablePartIndex+:8];
                        indexSpaceHandler <= 0;
                    end
                end
                else
                begin
                    startTxInternal <= 1;
                    state <= S9;
                end
        end
        S10:
        begin
            state <= S11;
        end
        // Defined end part of the message
        S11:
        begin
            // Starting the transfer
            startTxInternal <= 1;
                if(doneTx)
                begin
                    if(singleVariablePartIndex+7 < VARIABLE_LENGTH && indexSpaceHandler == 0)
                    begin
                        // Pass the starting part of the message
                        singleVariablePartIndex <= singleVariablePartIndex + 8;
                        indexSpaceHandler <= 1;
                        startTxInternal <= 0;
                    end
                    if(singleVariablePartIndex+7 >= VARIABLE_LENGTH && indexSpaceHandler == 1)
                    begin
                        state <= S12;
                        indexSpaceHandler <= 0;
                        startTxInternal <= 0;
                        singleVariablePartIndex <= 0;
                    end
                    if(singleVariablePartIndex+7 < VARIABLE_LENGTH && indexSpaceHandler == 1)
                    begin
                        inputTxByte <= stopSequence[singleVariablePartIndex+:8];
                        indexSpaceHandler <= 0;
                    end
                end
                else
                begin
                    startTxInternal <= 1;
                    state <= S11;
                end
        end

        // Needed as space because UART doneTx is HIGH for two cycles
        // Transfering to idle state
        S12:
        begin
            state <= S0;
        end

        default:
        begin
            state <= S0;
        end
    endcase
end

// Transmit via Serial Communication Module
uartTx #(.CLKS_PER_BIT(CLKS_PER_BIT)) uartTxModule(.clk(clk), .inputTxByte(inputTxByte), .isTxActive(), .outputTx(outputTx), .doneTx(doneTx), .startTx(startTxInternal));


// Receive via Serial Communication Module
uartRx #(.CLKS_PER_BIT(CLKS_PER_BIT)) uartRxModule(.clk(clk), .inputRx(inputRx), .outputRx(outputRx), .doneRx(doneRx));
endmodule
