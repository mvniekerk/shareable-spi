use atsamd_hal::common::time::Hertz;
use atsamd_hal::samd51::clock::GenericClockController;
use atsamd_hal as hal;

pub use hal::target_device as pac;
pub use hal::common::*;
pub use hal::samd51::*;
use embedded_hal::spi::Mode;
use gpio::{Floating, Input, Port};
use shareable_spi::{SharedSpiWithConf, SharedSpi, ReconfigurableSpiMode};
use atsamd_hal::sercom::{PadPin, SPIMaster5};
use core::ops::Deref;

pub type SpiMaster = SPIMaster5<
    hal::sercom::Sercom5Pad3<gpio::Pa25<gpio::PfD>>,
    hal::sercom::Sercom5Pad0<gpio::Pa23<gpio::PfD>>,
    hal::sercom::Sercom5Pad1<gpio::Pa22<gpio::PfD>>,
>;

pub type Spi5 = SharedSpi<SpiMaster>;
pub struct Spi5Wrapper<'a>(&'a Spi5);
pub type SharedSpi5<'a> = SharedSpiWithConf<Spi5Wrapper<'a>, SpiMaster>;

pub fn spi_master<F: Into<Hertz>>(
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
    SharedSpi::new(spi)
}

pub fn shared_spi(
    spi5: &Spi5,
    lora: Mode, adxl313: Mode, tc72: Mode, eeprom: Mode
) -> (SharedSpi5, SharedSpi5, SharedSpi5, SharedSpi5) {
    let lora = SharedSpiWithConf::new(Spi5Wrapper(spi5), lora);
    let adxl313 = SharedSpiWithConf::new(Spi5Wrapper(spi5), adxl313);
    let tc72 = SharedSpiWithConf::new(Spi5Wrapper(spi5), tc72);
    let eeprom = SharedSpiWithConf::new(Spi5Wrapper(spi5), eeprom);
    (lora, adxl313, tc72, eeprom)
}

impl<'a> ReconfigurableSpiMode for Spi5Wrapper<'a> {
    fn change_spi_mode(&mut self, mode: Mode) {
        self.0.set_mode(mode);
    }
}

impl<'a> Deref for Spi5Wrapper<'a> {
    type Target = &'a Spi5;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// impl<'a> Deref for Spi5Wrapper<'a> {
//     type Target = &'a dyn embedded_hal::blocking::spi::Write<u8, Error = atsamd_hal::samd51::sercom::Error>;
//
//     fn deref(&self) -> &Self::Target {
//         &self.0
//     }
// }
//
// impl<'a> Deref for Spi5Wrapper<'a> {
//     type Target = &'a dyn ReconfigurableSpiMode;
//
//     fn deref(&self) -> &Self::Target {
//         &self.0
//     }
// }