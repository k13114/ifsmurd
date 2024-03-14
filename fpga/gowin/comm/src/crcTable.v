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

module crcTable(index, resultReturn);

input [7:0] index;
reg [7:0] result;
output [7:0] resultReturn;

assign resultReturn = result;

always@(index)
begin


// Generated using https://crccalc.com/
// with CRC-8 HEX settings
case(index)
    8'd0: result = 8'h00;
    8'd1: result = 8'h07;
    8'd2: result = 8'h0e;
    8'd3: result = 8'h09;
    8'd4: result = 8'h1c;
    8'd5: result = 8'h1b;
    8'd6: result = 8'h12;
    8'd7: result = 8'h15;
    8'd8: result = 8'h38;
    8'd9: result = 8'h3f;
    8'd10: result = 8'h36;
    8'd11: result = 8'h31;
    8'd12: result = 8'h24;
    8'd13: result = 8'h23;
    8'd14: result = 8'h2a;
    8'd15: result = 8'h2d;
    8'd16: result = 8'h70;
    8'd17: result = 8'h77;
    8'd18: result = 8'h7e;
    8'd19: result = 8'h79;
    8'd20: result = 8'h6c;
    8'd21: result = 8'h6b;
    8'd22: result = 8'h62;
    8'd23: result = 8'h65;
    8'd24: result = 8'h48;
    8'd25: result = 8'h4f;
    8'd26: result = 8'h46;
    8'd27: result = 8'h41;
    8'd28: result = 8'h54;
    8'd29: result = 8'h53;
    8'd30: result = 8'h5a;
    8'd31: result = 8'h5d;
    8'd32: result = 8'hE0;
    8'd33: result = 8'hE7;
    8'd34: result = 8'hEE;
    8'd35: result = 8'hE9;
    8'd36: result = 8'hFC;
    8'd37: result = 8'hFB;
    8'd38: result = 8'hF2;
    8'd39: result = 8'hF5;
    8'd40: result = 8'hD8;
    8'd41: result = 8'hDF;
    8'd42: result = 8'hD6;
    8'd43: result = 8'hD1;
    8'd44: result = 8'hC4;
    8'd45: result = 8'hC3;
    8'd46: result = 8'hCA;
    8'd47: result = 8'hCD;
    8'd48: result = 8'h90;
    8'd49: result = 8'h97;
    8'd50: result = 8'h9E;
    8'd51: result = 8'h99;
    8'd52: result = 8'h8C;
    8'd53: result = 8'h8B;
    8'd54: result = 8'h82;
    8'd55: result = 8'h85;
    8'd56: result = 8'hA8;
    8'd57: result = 8'hAF;
    8'd58: result = 8'hA6;
    8'd59: result = 8'hA1;
    8'd60: result = 8'hB4;
    8'd61: result = 8'hB3;
    8'd62: result = 8'hBA;
    8'd63: result = 8'hBD;
    8'd64: result = 8'hC7;
    8'd65: result = 8'hC0;
    8'd66: result = 8'hC9;
    8'd67: result = 8'hCE;
    8'd68: result = 8'hDB;
    8'd69: result = 8'hDC;
    8'd70: result = 8'hD5;
    8'd71: result = 8'hD2;
    8'd72: result = 8'hFF;
    8'd73: result = 8'hF8;
    8'd74: result = 8'hF1;
    8'd75: result = 8'hF6;
    8'd76: result = 8'hE3;
    8'd77: result = 8'hE4;
    8'd78: result = 8'hED;
    8'd79: result = 8'hEA;
    8'd80: result = 8'hB7;
    8'd81: result = 8'hB0;
    8'd82: result = 8'hB9;
    8'd83: result = 8'hBE;
    8'd84: result = 8'hAB;
    8'd85: result = 8'hAC;
    8'd86: result = 8'hA5;
    8'd87: result = 8'hA2;
    8'd88: result = 8'h8F;
    8'd89: result = 8'h88;
    8'd90: result = 8'h81;
    8'd91: result = 8'h86;
    8'd92: result = 8'h93;
    8'd93: result = 8'h94;
    8'd94: result = 8'h9D;
    8'd95: result = 8'h9A;
    8'd96: result = 8'h27;
    8'd97: result = 8'h20;
    8'd98: result = 8'h29;
    8'd99: result = 8'h2E;
    8'd100: result = 8'h3B;
    8'd101: result = 8'h3C;
    8'd102: result = 8'h35;
    8'd103: result = 8'h32;
    8'd104: result = 8'h1F;
    8'd105: result = 8'h18;
    8'd106: result = 8'h11;
    8'd107: result = 8'h16;
    8'd108: result = 8'h03;
    8'd109: result = 8'h04;
    8'd110: result = 8'h0D;
    8'd111: result = 8'h0A;
    8'd112: result = 8'h57;
    8'd113: result = 8'h50;
    8'd114: result = 8'h59;
    8'd115: result = 8'h5E;
    8'd116: result = 8'h4B;
    8'd117: result = 8'h4C;
    8'd118: result = 8'h45;
    8'd119: result = 8'h42;
    8'd120: result = 8'h6F;
    8'd121: result = 8'h68;
    8'd122: result = 8'h61;
    8'd123: result = 8'h66;
    8'd124: result = 8'h73;
    8'd125: result = 8'h74;
    8'd126: result = 8'h7D;
    8'd127: result = 8'h7A;
    8'd128: result = 8'h89;
    8'd129: result = 8'h8E;
    8'd130: result = 8'h87;
    8'd131: result = 8'h80;
    8'd132: result = 8'h95;
    8'd133: result = 8'h92;
    8'd134: result = 8'h9B;
    8'd135: result = 8'h9C;
    8'd136: result = 8'hB1;
    8'd137: result = 8'hB6;
    8'd138: result = 8'hBF;
    8'd139: result = 8'hB8;
    8'd140: result = 8'hAD;
    8'd141: result = 8'hAA;
    8'd142: result = 8'hA3;
    8'd143: result = 8'hA4;
    8'd144: result = 8'hF9;
    8'd145: result = 8'hFE;
    8'd146: result = 8'hF7;
    8'd147: result = 8'hF0;
    8'd148: result = 8'hE5;
    8'd149: result = 8'hE2;
    8'd150: result = 8'hEB;
    8'd151: result = 8'hEC;
    8'd152: result = 8'hC1;
    8'd153: result = 8'hC6;
    8'd154: result = 8'hCF;
    8'd155: result = 8'hC8;
    8'd156: result = 8'hDD;
    8'd157: result = 8'hDA;
    8'd158: result = 8'hD3;
    8'd159: result = 8'hD4;
    8'd160: result = 8'h69;
    8'd161: result = 8'h6E;
    8'd162: result = 8'h67;
    8'd163: result = 8'h60;
    8'd164: result = 8'h75;
    8'd165: result = 8'h72;
    8'd166: result = 8'h7B;
    8'd167: result = 8'h7C;
    8'd168: result = 8'h51;
    8'd169: result = 8'h56;
    8'd170: result = 8'h5F;
    8'd171: result = 8'h58;
    8'd172: result = 8'h4D;
    8'd173: result = 8'h4A;
    8'd174: result = 8'h43;
    8'd175: result = 8'h44;
    8'd176: result = 8'h19;
    8'd177: result = 8'h1E;
    8'd178: result = 8'h17;
    8'd179: result = 8'h10;
    8'd180: result = 8'h05;
    8'd181: result = 8'h02;
    8'd182: result = 8'h0B;
    8'd183: result = 8'h0C;
    8'd184: result = 8'h21;
    8'd185: result = 8'h26;
    8'd186: result = 8'h2F;
    8'd187: result = 8'h28;
    8'd188: result = 8'h3D;
    8'd189: result = 8'h3A;
    8'd190: result = 8'h33;
    8'd191: result = 8'h34;
    8'd192: result = 8'h4E;
    8'd193: result = 8'h49;
    8'd194: result = 8'h40;
    8'd195: result = 8'h47;
    8'd196: result = 8'h52;
    8'd197: result = 8'h55;
    8'd198: result = 8'h5C;
    8'd199: result = 8'h5B;
    8'd200: result = 8'h76;
    8'd201: result = 8'h71;
    8'd202: result = 8'h78;
    8'd203: result = 8'h7F;
    8'd204: result = 8'h6A;
    8'd205: result = 8'h6D;
    8'd206: result = 8'h64;
    8'd207: result = 8'h63;
    8'd208: result = 8'h3E;
    8'd209: result = 8'h39;
    8'd210: result = 8'h30;
    8'd211: result = 8'h37;
    8'd212: result = 8'h22;
    8'd213: result = 8'h25;
    8'd214: result = 8'h2C;
    8'd215: result = 8'h2B;
    8'd216: result = 8'h06;
    8'd217: result = 8'h01;
    8'd218: result = 8'h08;
    8'd219: result = 8'h0F;
    8'd220: result = 8'h1A;
    8'd221: result = 8'h1D;
    8'd222: result = 8'h14;
    8'd223: result = 8'h13;
    8'd224: result = 8'hAE;
    8'd225: result = 8'hA9;
    8'd226: result = 8'hA0;
    8'd227: result = 8'hA7;
    8'd228: result = 8'hB2;
    8'd229: result = 8'hB5;
    8'd230: result = 8'hBC;
    8'd231: result = 8'hBB;
    8'd232: result = 8'h96;
    8'd233: result = 8'h91;
    8'd234: result = 8'h98;
    8'd235: result = 8'h9F;
    8'd236: result = 8'h8A;
    8'd237: result = 8'h8D;
    8'd238: result = 8'h84;
    8'd239: result = 8'h83;
    8'd240: result = 8'hDE;
    8'd241: result = 8'hD9;
    8'd242: result = 8'hD0;
    8'd243: result = 8'hD7;
    8'd244: result = 8'hC2;
    8'd245: result = 8'hC5;
    8'd246: result = 8'hCC;
    8'd247: result = 8'hCB;
    8'd248: result = 8'hE6;
    8'd249: result = 8'hE1;
    8'd250: result = 8'hE8;
    8'd251: result = 8'hEF;
    8'd252: result = 8'hFA;
    8'd253: result = 8'hFD;
    8'd254: result = 8'hF4;
    8'd255: result = 8'hF3;
    default: result = 8'h00;
endcase
end

endmodule
