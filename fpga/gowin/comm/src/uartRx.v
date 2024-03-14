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

module uartRx #(parameter CLKS_PER_BIT = 870)(clk, inputRx, outputRx, doneRx);

input clk, inputRx;
output [7:0] outputRx;
output doneRx;


// Parameters for state machine

parameter idleState = 3'b000;
parameter startBitState = 3'b001;
parameter transmitBitState = 3'b010;
parameter stopBitState = 3'b011;
parameter cleanUpState = 3'b100;

reg [2:0] state = 3'b0;
reg [31:0] clockCount = 0;
reg [2:0] bitIndex = 0;
reg [7:0] outputRxInternal = 0;
reg doneRxInternal = 0;
reg inputRxInternalPrev = 1'b1;
reg inputRxInternal = 1'b1;

assign doneRx = doneRxInternal;
assign outputRx = outputRxInternal;


always@(posedge clk)
begin
    inputRxInternalPrev <= inputRx;
    inputRxInternal <= inputRxInternalPrev;
end


always@(posedge clk)
begin
    case(state)

        idleState:
        begin
            doneRxInternal <= 1'b0;
            clockCount <= 0;
            bitIndex <= 0;

            if(inputRxInternal == 1'b0)
            begin
                state <= startBitState;
            end
            else
            begin
                state <= idleState;
            end
        end


        startBitState:
        begin
            if(clockCount < (CLKS_PER_BIT -1)/2)
            begin
                clockCount <= clockCount + 1;
            end
            else
            begin
                if(inputRxInternal == 1'b0)
                begin
                    state <= transmitBitState;
                    clockCount <= 0;
                end
                else
                begin
                    state <= startBitState;
                end
            end
        end

        transmitBitState:
        begin
            if(clockCount < (CLKS_PER_BIT - 1))
            begin
                clockCount <= clockCount + 1;
                state <= transmitBitState;
            end
            else
            begin
                clockCount <= 0;
                outputRxInternal[bitIndex] <= inputRxInternal;
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
            if(clockCount < (CLKS_PER_BIT-1))
            begin
                state <= stopBitState;
                clockCount <= clockCount + 1;
            end
            else
            begin
                state <= cleanUpState;
                clockCount <= 0;
                doneRxInternal <= 1'b1;
            end
        end

        cleanUpState:
        begin
            state <= idleState;
            doneRxInternal <= 1'b0;
        end

        default:
        begin
            state <= idleState;
        end
    endcase
end



endmodule
