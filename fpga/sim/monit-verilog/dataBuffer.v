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

module dataBuffer #(parameter BUFFER_LENGTH = 32, // Number of positions in the buffer
    parameter PTR_LENGTH = 5, // Should correspond to the BUFFER_LENGTH, used in a definition, so when PTR_LENGTH = 2, the ptr is defined with [2-1:0] wr_ptr, rd_ptr; so maximum number is 2^2-1 here
    parameter VARIABLE_LENGTH_BITS = 32 // Length of stored variable in bits
)
(rst, clk, wr_enable, rd_enable, buf_in, buf_out);

input clk, rst;                                 // clock and reset signals
input [VARIABLE_LENGTH_BITS-1:0] buf_in;        // buffer input signal
reg [VARIABLE_LENGTH_BITS-1:0] buf_out_reg;     // buffer output register connected to the buffer output signal
output [VARIABLE_LENGTH_BITS-1:0] buf_out;      // buffer output signal
reg buf_full, buf_empty;                        // indicators if the buf_full - buffer is full, buf_empty - buffer is empty
input wr_enable, rd_enable;                     // signals from connected module for wr_enable - enabling writing operation to the buffer, rd_enable - enabling reading from the buffer at the current position
reg [PTR_LENGTH-1:0] wr_ptr, rd_ptr;            // pointers in the buffer - wr_ptr - write pointer, rd_ptr - read pointer
reg [VARIABLE_LENGTH_BITS-1:0] buf_mem [BUFFER_LENGTH-1:0]; // buffer memory, where the data is stored
reg [VARIABLE_LENGTH_BITS-1:0] buf_cnt;         // buffer counter for counting used positions in the buffer structure and based on that disabling or enabling writing/reading from the buffer

// Output wire from the buffer
assign buf_out = buf_out_reg;

// Seting the flags if the buffer is full or not
always@(buf_cnt)
begin
    buf_full <= (buf_cnt == BUFFER_LENGTH);
    buf_empty <= (buf_cnt == 0);
end

// Buffer counter to know, if the buffer is full or not
always@(posedge clk or posedge rst)
begin
    if(rst)
    begin
        buf_cnt <= 0;
    end
    else
    begin
        if((!buf_full && wr_enable) && (!buf_empty && rd_enable))
        begin
            buf_cnt <= buf_cnt;
        end
        else if(!buf_full && wr_enable)
        begin
            buf_cnt <= buf_cnt + 1;
        end
        else if(!buf_empty && rd_enable)
        begin
            buf_cnt <= buf_cnt - 1;
        end
        else
        begin
            buf_cnt <= buf_cnt;
        end
    end
end

// Moving the write and read pointers
always@(posedge clk or posedge rst)
begin
    if(rst)
    begin
        wr_ptr <= 0;
        rd_ptr <= 0;
    end
    else
    begin
        if(!buf_full && wr_enable)
        begin
            wr_ptr <= wr_ptr + 1;
        end
        else
        begin
            wr_ptr <= wr_ptr;
        end

        if(!buf_empty && rd_enable)
        begin
            rd_ptr <= rd_ptr + 1;
        end
        else
        begin
            rd_ptr <= rd_ptr;
        end
    end
end

// Reading from buffer
always@(posedge clk or posedge rst)
begin
    if(rst)
    begin
        buf_out_reg <= 0;
    end
    else
    begin
        if(!buf_empty && rd_enable)
        begin
            buf_out_reg <= buf_mem[rd_ptr];
        end
        else
        begin
            buf_out_reg <= buf_out;
        end
    end
end


// Writing to buffer
always@(posedge clk or posedge rst)
begin
    if(rst)
    begin
        buf_mem[wr_ptr] <= 0; // maybe delete later
    end
    else
    begin
        if(!buf_full && wr_enable)
        begin
            buf_mem[wr_ptr] <= buf_in;
        end
        else
        begin
            buf_mem[wr_ptr] <= buf_mem[wr_ptr];
        end
    end
end

endmodule
