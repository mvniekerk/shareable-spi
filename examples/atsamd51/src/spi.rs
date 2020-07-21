use atsamd_hal::common::time::Hertz;
use atsamd_hal::samd51::clock::GenericClockController;
use atsamd_hal as hal;

pub use hal::target_device as pac;
pub use hal::common::*;
pub use hal::samd51::*;
use embedded_hal::spi::Mode;
use gpio::{Floating, Input, Port};
use crate::{Spi5, SharedSpi5};
use shareable_spi::{SharedSpiWithConf, SharedSpi};
use atsamd_hal::sercom::{PadPin, SPIMaster5};


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
    let lora = SharedSpiWithConf::new(spi5, lora);
    let adxl313 = SharedSpiWithConf::new(spi5, adxl313);
    let tc72 = SharedSpiWithConf::new(spi5, tc72);
    let eeprom = SharedSpiWithConf::new(spi5, eeprom);
    (lora, adxl313, tc72, eeprom)
}