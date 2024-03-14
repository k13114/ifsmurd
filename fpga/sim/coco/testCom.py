# Cocotb runner API url: https://docs.cocotb.org/en/latest/library_reference.html#api-runner
# Verilator flags url: https://veripool.org/guide/latest/exe_verilator.html


# For getting the env variables from system
import os

# For determining the current path to a folder / project
from pathlib import Path


# cocotb definitions
# Main import
import cocotb
# For timer and setting the time in the simulation/test
from cocotb.triggers import Timer
# For generating the clock signals easily
from cocotb.clock import Clock
# For running the script with the pytest
from cocotb.runner import get_runner


@cocotb.test()
async def first_value_test(dut):
    # Just what to print in the summary
    """UART"""

    # Setting time units for whole cocotb simulation
    unitsOfTime = "ns"
    cocotb.start_soon(Clock(dut.clk, 10, units=unitsOfTime).start())

    await Timer(90, units=unitsOfTime)
    #dut.inputTxByte.value = 0b10001001  # modulationIndex = 1
    await Timer(90, units=unitsOfTime)
    dut.startTx.value = 0b1
    await Timer(90, units=unitsOfTime)
    dut.startTx.value = 0b0

    simTime = 2*200000
    await Timer(simTime, units=unitsOfTime)

# Function for running the script with pytest


def test_simple_dff_runner():

    hdlTopLevelInput = "serialCom"
    testModuleInput = "testCom"

    # Get the environmental variable from system of HDL_TOP_LEVEL_LANG, where it is not set, set the verilog option as default
    hdl_toplevel_lang = os.getenv("HDL_TOPLEVEL_LANG", "verilog")

    # Get the environmental variable from system of SIM, where the simulator is set, when no value is provided, use icarus
    sim = os.getenv("SIM", "icarus")

    # Saving the path for folder where this file is saved, where the folder resides in the system
    proj_path = Path(__file__).resolve().parent

    # Preparing blank arrays to be later used
    # For HDl sources
    verilog_sources = []
    vhdl_sources = []

    # Checking if the HDL_TOPLEVEL_LANG is set to verilog or VHDL
    # And setting the path for needed files with the project_path/ prepended
    if hdl_toplevel_lang == "verilog":
        verilog_sources = [proj_path / "../verilog/uartTx.v",
                           proj_path / "../verilog/uartRx.v",
                           proj_path / "../verilog/serialCom.v",
                           ]
    else:
        vhdl_sources = [proj_path / ""]

    # Initializing the runner with the set simulator in the sim variable
    runner = get_runner(sim)

    # Building the simulation with specified parameters
    runner.build(
        verilog_sources=verilog_sources,
        vhdl_sources=vhdl_sources,
        # Which module is the top level in this simulation
        hdl_toplevel=hdlTopLevelInput,
        # Always run the build step
        always=True,
        # Arguments for the simulator - for Verilator this time
        # Ignoring some warnings, enabling multiple jobs to build the simulation, enabling tracing to dump.vcd file, enabling timing support
        build_args=[ "-Wno-TIMESCALEMOD", "-Wno-WIDTHTRUNC" , "--verilate-jobs", "-j 10", "--trace-fst", "--trace-structs", "--timing"],
    )

    # Run the test when running with pytest
    runner.test(
                # Specifing the hdl_top level again
                hdl_toplevel=hdlTopLevelInput,
                # How the pythons script, where the tests are located is named - this file for now
                test_module=testModuleInput,
                # Should be generating waves output, but is not doing it right now
                waves=True,
                # Print more verbose output
                verbose=True
                )
