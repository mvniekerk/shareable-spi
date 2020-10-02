#![no_std]
#![recursion_limit = "1024"]

use atsamd_hal as hal;

#[cfg(feature = "rt")]
pub use cortex_m_rt::entry;

use hal::*;

pub use crate::pins::Pins;

pub use hal::common::*;
pub use hal::samd51::*;
pub use hal::target_device as pac;

use crate::cs::SharedOutputPin;
use atsamd_hal::common::gpio::{Floating, Input, Output, Port, PushPull};
use core::ops::{Deref, DerefMut};
use embedded_hal::blocking::spi::{Transfer, Write};
use embedded_hal::spi::Phase::{CaptureOnFirstTransition, CaptureOnSecondTransition};
use embedded_hal::spi::Polarity::{IdleHigh, IdleLow};
use embedded_hal::spi::{Mode, Polarity};
use hal::clock::GenericClockController;
use hal::sercom::{PadPin, SPIMaster5};
use hal::time::Hertz;
use shareable_spi::{ReconfigurableSpiMode, ShareableSpiWithConf, SharedSpi, SpiLock};

pub type SpiMaster = SPIMaster5<
    hal::sercom::Sercom5Pad3<gpio::Pa25<gpio::PfD>>,
    hal::sercom::Sercom5Pad0<gpio::Pa23<gpio::PfD>>,
    hal::sercom::Sercom5Pad1<gpio::Pa22<gpio::PfD>>,
>;

pub struct Spi5Wrapper(SpiMaster);

pub type Spi5 = SharedSpi<Spi5Wrapper>;
pub type SharedSpi5<'a> = ShareableSpiWithConf<&'a Spi5, Spi5Wrapper>;

pub type CsAdxl = SharedOutputPin<gpio::Pa13<Output<PushPull>>>;

static mut SPI5: Option<Spi5> = None;

/// This powers up SERCOM5 and configures it for use as an
/// SPI Master in SPI Mode 0.
/// Unlike the `flash_spi_master` function, this
/// one does not accept a CS pin; configuring a pin for CS
/// is the responsibility of the caller, because it could be
/// any OutputPin, or even a pulled up line on the slave.
pub fn spi_master<'a, F: Into<Hertz>>(
    clocks: &mut GenericClockController,
    bus_speed: F,
    sercom5: pac::SERCOM5,
    mclk: &mut pac::MCLK,
    sck: gpio::Pa22<Input<Floating>>,
    mosi: gpio::Pa23<Input<Floating>>,
    miso: gpio::Pa25<Input<Floating>>,
    port: &mut Port,
) -> Spi5 {
    let gclk0 = clocks.gclk0();
    let spi = SPIMaster5::new(
        &clocks.sercom5_core(&gclk0).unwrap(),
        bus_speed.into(),
        hal::hal::spi::Mode {
            phase: hal::hal::spi::Phase::CaptureOnFirstTransition,
            polarity: hal::hal::spi::Polarity::IdleLow,
        },
        sercom5,
        mclk,
        (miso.into_pad(port), mosi.into_pad(port), sck.into_pad(port)),
    );

    SharedSpi::new(Spi5Wrapper(spi))
}

impl Deref for Spi5Wrapper {
    type Target = SpiMaster;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Spi5Wrapper {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl ReconfigurableSpiMode for Spi5Wrapper {
    fn change_spi_mode(&mut self, mode: Mode) {
        self.set_mode(mode);
    }
}

impl Transfer<u8> for Spi5Wrapper {
    type Error = atsamd_hal::sercom::Error;

    fn transfer<'w>(&mut self, words: &'w mut [u8]) -> Result<&'w [u8], Self::Error> {
        let r: &mut SpiMaster = self.deref_mut();
        r.transfer(words)
    }
}

impl Write<u8> for Spi5Wrapper {
    type Error = atsamd_hal::sercom::Error;

    fn write(&mut self, words: &[u8]) -> Result<(), Self::Error> {
        let r: &mut SpiMaster = self.deref_mut();
        r.write(words)
    }
}

pub fn shared_spi(
    spi5: &Spi5,
    lora: Mode,
    adxl313: Mode,
    tc72: Mode,
    eeprom: Mode,
) -> (SharedSpi5, SharedSpi5, SharedSpi5, SharedSpi5) {
    let lora = ShareableSpiWithConf::new(spi5, lora);
    let adxl313 = ShareableSpiWithConf::new(spi5, adxl313);
    let tc72 = ShareableSpiWithConf::new(spi5, tc72);
    let eeprom = ShareableSpiWithConf::new(spi5, eeprom);
    (lora, adxl313, tc72, eeprom)
}
