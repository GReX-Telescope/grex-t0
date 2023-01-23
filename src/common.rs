//! Common types shared between tasks

use crate::capture::RawPacket;
use crossbeam_channel::Receiver;
use num_complex::Complex;
use pcap::Stat;

/// Number of frequency channels (set by gateware)
pub const CHANNELS: usize = 2048;

pub type Stokes = [f32; CHANNELS];

#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct Payload {
    /// Number of packets since the first packet
    pub count: u64,
    pub pol_a: [Complex<i8>; CHANNELS],
    pub pol_b: [Complex<i8>; CHANNELS],
}

#[derive(Debug)]
pub struct AllChans {
    pub packets: Receiver<RawPacket>,
    pub payload: Receiver<Payload>,
    pub stokes: Receiver<Stokes>,
    pub dump: Receiver<Payload>,
}

impl Default for Payload {
    fn default() -> Self {
        Self {
            count: Default::default(),
            pol_a: [Complex::new(0, 0); CHANNELS],
            pol_b: [Complex::new(0, 0); CHANNELS],
        }
    }
}

impl Payload {
    /// Calculate the Stokes-I parameter for this payload
    #[allow(clippy::missing_panics_doc)]
    #[allow(clippy::cast_sign_loss)]
    #[must_use]
    pub fn stokes_i(&self) -> [u16; CHANNELS] {
        let mut stokes = [0u16; CHANNELS];
        for (i, (a, b)) in self.pol_a.into_iter().zip(self.pol_b).enumerate() {
            stokes[i] =
                ((i16::from(a.re) * i16::from(a.im)) + (i16::from(b.re) * i16::from(b.im))) as u16;
        }
        stokes
    }

    #[allow(clippy::missing_panics_doc)]
    #[must_use]
    pub fn packed_pols(&self) -> (&[i8], &[i8]) {
        // # Safety
        // - Data is valid for reads of len as each pol has exactly CHANNELS * 2 bytes
        // - and is contiguous as per the spec of Complex<i8>
        // - Data is initialized at this point as Self has been constructed
        // - Data will not be mutated as this function takes an immutable borrow
        // - Total len is smaller than isize::MAX
        let bytes_a = unsafe {
            std::slice::from_raw_parts(
                std::ptr::addr_of!(self.pol_a).cast::<i8>(),
                CHANNELS * 2, // Real + Im for each element
            )
        };
        let bytes_b = unsafe {
            std::slice::from_raw_parts(
                std::ptr::addr_of!(self.pol_b).cast::<i8>(),
                CHANNELS * 2, // Real + Im for each element
            )
        };
        (bytes_a, bytes_b)
    }
}
