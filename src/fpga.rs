//! Control of the SNAP board running the gateware

use casperfpga::transport::{
    tapcp::{Platform, Tapcp},
    Transport,
};
use casperfpga_derive::fpga_from_fpg;
use chrono::{DateTime, TimeZone, Utc};
use rsntp::SynchronizationResult;
use std::net::SocketAddr;

fpga_from_fpg!(GrexFpga, "gateware/grex_gateware_2022-11-09_2251.fpg");

pub struct Device {
    fpga: GrexFpga<Tapcp>,
}

impl Device {
    #[allow(clippy::missing_panics_doc)]
    #[must_use]
    pub fn new(addr: SocketAddr) -> Self {
        let fpga = GrexFpga::new(Tapcp::connect(addr, Platform::SNAP).expect("Connection failed"))
            .expect("Failed to build FPGA object");
        assert!(
            fpga.transport.lock().unwrap().is_running().unwrap(),
            "SNAP board is not programmed/running"
        );
        Self { fpga }
    }

    /// Send a trigger pulse to start the flow of bytes, returning the true time of the start of packets
    #[allow(clippy::missing_panics_doc)]
    pub fn trigger(&mut self, time_sync: &SynchronizationResult) -> DateTime<Utc> {
        // Get the current time, and wait to send the triggers to align the time with a rising PPS edge
        let now: DateTime<Utc> = time_sync.datetime().try_into().unwrap();
        // If we wait until halfway through the second, we have the maximum likleyhood of preventing a fencepost error
        let trigger_time = Utc.timestamp_opt(now.timestamp() + 1, 500_000_000).unwrap();
        // PPS will trigger on the next starting edge after we arm
        let start_time = Utc.timestamp_opt(now.timestamp() + 2, 0).unwrap();
        std::thread::sleep((trigger_time - now).to_std().unwrap());
        // Send the trigger
        self.fpga.master_rst.write(false).unwrap();
        self.fpga.master_rst.write(true).unwrap();
        self.fpga.master_rst.write(false).unwrap();
        // Update our time
        start_time
    }

    /// Force a PPS pulse (timing will be inaccurate)
    #[allow(clippy::missing_panics_doc)]
    pub fn force_pps(&mut self) {
        self.fpga.pps_trig.write(false).unwrap();
        self.fpga.pps_trig.write(true).unwrap();
        self.fpga.pps_trig.write(false).unwrap();
    }
}
