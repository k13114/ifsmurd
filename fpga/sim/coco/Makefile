include $(shell cocotb-config --makefiles)/Makefile.sim

# delete -s flag to allow the sdout and stderr output to html files


pytestTx:
	SIM=verilator HDL_TOPLEVEL_LANG=verilog pytest --html=testReportTx.html testTx.py -s

pytestRx:
	SIM=verilator HDL_TOPLEVEL_LANG=verilog pytest --html=testReportRx.html testRx.py -s

pytestCom:
	SIM=verilator HDL_TOPLEVEL_LANG=verilog pytest --html=testReportSerialCom.html testCom.py -s

pytestMonit:
	SIM=verilator HDL_TOPLEVEL_LANG=verilog pytest --html=testReportMonit.html testMonit.py -s

