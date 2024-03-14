/*

MIT License

Copyright (c) 2023 Petr Zakopal, Deparment of Electric Drives and Traction, CTU FEE

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

// CLKS_PER_BIT = clock frequency/uart baud rate
// example: clock frequency 100 MHz = 100000000
// baud rate 115200 (bits per second)
// CLKS_PER_BIT = 100000000 / 115200 = 868

module uartTx #(parameter CLKS_PER_BIT = 870)(
    clk, inputTxByte, isTxActive, outputTx, doneTx, startTx
);

// Inpputs and outputs definition
input clk, startTx;
input [7:0] inputTxByte;
output isTxActive, doneTx;
output reg outputTx;

// Parameters for state machine

parameter idleState = 3'b000;
parameter startBitState = 3'b001;
parameter transmitBitState = 3'b010;
parameter stopBitState = 3'b011;
parameter cleanUpState = 3'b100;

reg [2:0] state = 0;
reg [31:0] clockCount = 0;
reg [2:0] bitIndex = 0;
reg [7:0] inputTxByteInternal= 0;
reg doneTxInternal = 0;
reg isTxActiveInternal = 0;

assign isTxActive = isTxActiveInternal;
assign doneTx = doneTxInternal;


always @(posedge clk)
begin
    case (state)
        idleState:
        begin
            outputTx <= 1'b1;
            doneTxInternal <= 0;
            isTxActiveInternal <= 0;
            bitIndex <= 0;

            if(startTx)
            begin
                isTxActiveInternal <= 1;
                outputTx <= 0;
                inputTxByteInternal <= inputTxByte;
                state <= startBitState;
            end
            else
            begin
                state <= idleState;
            end
        end

        startBitState:
        begin
            outputTx <= 1'b0;
            if(clockCount < (CLKS_PER_BIT-1))
            begin
                clockCount <= clockCount + 1;
                state <= startBitState;
            end
            else
            begin
                clockCount <= 0;
                state <= transmitBitState;
            end
        end


        transmitBitState:
        begin
            outputTx <= inputTxByteInternal[bitIndex];

            if(clockCount < (CLKS_PER_BIT-1))
                begin
                    clockCount <= clockCount + 1'b1;
                    state <= transmitBitState;
                end
                else
                begin
                    clockCount <= 0;
                    if(bitIndex < 7)
                        begin
                            bitIndex <= bitIndex + 1;
                            state <= transmitBitState;
                        end
                    else
                        begin
                            bitIndex <= 0;
                            state <= stopBitState;
                        end
                    end
                end



        stopBitState:
        begin
            outputTx <= 1'b1;

            if(clockCount < (CLKS_PER_BIT-1))
            begin
                clockCount <= clockCount + 1;
                state <= stopBitState;
            end
            else
            begin
                clockCount <= 0;
                state <= cleanUpState;
                doneTxInternal <= 1'b1;
                isTxActiveInternal <= 1'b0;
            end
        end

        cleanUpState:
        begin
            doneTxInternal <= 1'b1;
            state <= idleState;
        end
        
        default:
        begin
            state <= idleState;
        end
    endcase
end

endmodule
