#![no_std]
#![no_main]
#![allow(deprecated)]
extern crate sx127x_lora;
extern crate cortex_m;
extern crate atsamd_hal;
extern crate shareable_spi;

use atsamd_hal as hal;

pub mod pins;
pub mod spi;

use hal::*;

pub use crate::pins::Pins;
pub use hal::target_device as pac;
pub use hal::common::*;
pub use hal::samd51::*;

use gpio::{Floating, Input, Port};
use hal::sercom::{PadPin, SPIMaster5};
use hal::time::Hertz;
use shareable_spi::{SharedSpi, SharedSpiWithConf};
use embedded_hal::spi::Polarity::{IdleLow, IdleHigh};
use embedded_hal::spi::Phase::{CaptureOnFirstTransition, CaptureOnSecondTransition};
use embedded_hal::spi::Mode;

// use cortex_m_semihosting::hio;
use core::fmt::Write;

use embedded_hal::digital::v1_compat::OldOutputPin;

use hal::prelude::*;
use hal::timer::SpinTimer;
use hal::{clock::GenericClockController, delay::Delay};
use hal::time::MegaHertz;
use core::borrow::Borrow;
use core::result::Result::Ok;
use atsamd_hal::hal::spi::{Phase, Polarity};

use embedded_hal::blocking::delay::DelayMs;
use adxl313::{SleepModeFrequencyReadings, Accelerometer};
use microchip_tc72r_rs::Tc72;
use embedded_hal::digital::v2::OutputPin;
use core::cell::UnsafeCell;
use embedded_hal::spi::{MODE_0, MODE_1, MODE_2, MODE_3};
use crate::spi::{shared_spi, spi_master};
use atsamd_hal::common::gpio::{PushPull, Output, OpenDrain, Pa7, Pa13, Pa15, Pa24, Pa6};

use rtic::{app, Peripherals};

use crate::spi::{Spi5, SharedSpi5};


#[cfg(not(feature = "use_semihosting"))]
extern crate panic_halt;
#[cfg(feature = "use_semihosting")]
extern crate panic_semihosting;

#[cfg(feature = "use_semihosting")]
macro_rules! dbgprint {
    ($($arg:tt)*) => {
        {
            use cortex_m_semihosting::hio;
            use core::fmt::Write;
            let mut stdout = hio::hstdout().unwrap();
            writeln!(stdout, $($arg)*).ok();
        }
    };
}

#[cfg(not(feature = "use_semihosting"))]
macro_rules! dbgprint {
    ($($arg:tt)*) => {{}};
}
const FREQUENCY: i64 = 868;

#[app(device = atsamd_hal::target_device, peripherals = true)]
const APP: () = {
    struct Resources {
        lora: sx127x_lora::LoRa<SharedSpi5<'static>, Pa7<Output<PushPull>>, Pa6<Output<OpenDrain>>>,
        adxl313: adxl313::Adxl313<SharedSpi5<'static>, Pa13<Output<PushPull>>>,
        tc72: microchip_tc72r_rs::Tc72<SharedSpi5<'static>, Pa15<Output<OpenDrain>>>,
        e25x: spi_memory::series25::Flash<SharedSpi5<'static>, Pa24<Output<OpenDrain>>>,
        // e25x: spi_memory::series25::Flash<, Pa24<Output<OpenDrain>>>,
        delay: atsamd_hal::common::delay::Delay,
        spi: &'static Spi5,
    }

    #[init]
    fn init(c: init::Context) -> init::LateResources {
        static mut SPI5: Option<Spi5> = None;
        let mut device: Peripherals = c.device;
        let mut core = atsamd_hal::target_device::Peripherals::take().unwrap();

        let mut clocks = GenericClockController::with_external_32kosc(
            device.GCLK,
            &mut device.MCLK,
            &mut device.OSC32KCTRL,
            &mut device.OSCCTRL,
            &mut device.NVMCTRL,
        );

        let mut delay = Delay::new(core.SYST, &mut clocks);
        let gclk = clocks.gclk0();

        let mut pins = pins::Pins::new(device.PORT);

        let mut lora_reset = pins.lora_rst.into_open_drain_output(&mut pins.port);
        let mut lora_cs = pins.lora_cs.into_push_pull_output(&mut pins.port);
        let mut adxl313_cs = pins.acc_cs.into_push_pull_output(&mut pins.port);
        let mut tc72_cs = pins.tc72_cs.into_open_drain_output(&mut pins.port);
        let mut eep_cs = pins.eep_cs.into_open_drain_output(&mut pins.port);

        *SPI5 = Some(spi_master(
            // &mut clocks.sercom5_core(&gclk).unwrap(),
            &mut clocks,
            MegaHertz(5),
            device.SERCOM5,
            &mut device.MCLK,
            pins.spi_sck,
            pins.spi_mosi,
            pins.spi_miso,
            &mut pins.port,
        ));

        let (lora_spi, adxl313_spi, temp_spi, eep_spi) = shared_spi(
            SPI5.as_ref().unwrap(),
            MODE_0, MODE_1, MODE_2, MODE_3,
        );

        let e25x = spi_memory::series25::Flash::init_with_page_size(&eep_spi, eep_cs, 256);

        if let Err(_e) = e25x {
            loop {}
        }

        let adxl313 = adxl313::Adxl313::new(adxl313_spi, adxl313_cs);

        if let Err(_e) = adxl313 {
            loop {}
        }

        let lora = sx127x_lora::LoRa::new(
            lora_spi, lora_cs, lora_reset, FREQUENCY,
            &mut delay);

        if let Err(_e) = lora {
            loop {}
        }

        let tc72 = microchip_tc72r_rs::Tc72::new(temp_spi, tc72_cs);

        if let Err(_e) = tc72 {
            loop {}
        }

        let lora = lora.unwrap();
        let adxl313 = adxl313.unwrap();
        let tc72 = tc72.unwrap();
        let e25x = e25x.unwrap();

        init::LateResources {
            lora,
            adxl313,
            tc72,
            e25x,
            spi: SPI5.as_ref().unwrap(),
        }
    }

    // Interrupt handlers used to dispatch software tasks
    extern "C" {
        fn I2S();
    }
};
