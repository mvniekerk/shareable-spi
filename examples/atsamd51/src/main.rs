#![no_std]
#![no_main]
#![allow(deprecated)]
extern crate atsamd_hal;
extern crate cortex_m;
extern crate spi_memory;
extern crate sx127x_lora;
extern crate ws2812_timer_delay as ws2812;

use rtic::app;

#[cfg(not(feature = "use_semihosting"))]
extern crate panic_halt;
#[cfg(feature = "use_semihosting")]
extern crate panic_semihosting;

use core::fmt::Write;
use cortex_m_semihosting::hio;

use embedded_hal::digital::v1_compat::OldOutputPin;

use atsamd_hal::common::gpio::{Floating, Input, OpenDrain, Output, PushPull};
use atsamd_hal::hal::spi::{Phase, Polarity};
use core::borrow::Borrow;
use core::result::Result::Ok;
use hal::prelude::*;
use hal::time::MegaHertz;
use hal::timer::SpinTimer;
use hal::{clock::GenericClockController, delay::Delay};
use hal::{entry, shared_spi, spi_master, CsAdxl, SharedSpi5, Spi5, SpiMaster};

use adxl313::{Accelerometer, SleepModeFrequencyReadings};
use core::cell::UnsafeCell;
use embedded_hal::blocking::delay::DelayMs;
use embedded_hal::digital::v2::OutputPin;
use embedded_hal::spi::{MODE_0, MODE_1, MODE_2, MODE_3};
use hal::cs::{DummyOp, SharedOutputPin};
use microchip_tc72r_rs::Tc72;
use pfza_2020_01::gpio::*;

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

const FREQUENCY: i64 = 869;
// const FREQUENCY: i64 = 433;

#[app(device = crate::hal::pac, peripherals = true)]
const APP: () = {
    struct Resources {
        led0: Pa19<Output<OpenDrain>>,
        led1: Pb16<Output<OpenDrain>>,
        led2: Pb17<Output<OpenDrain>>,
        led3: Pa20<Output<OpenDrain>>,

        lora: sx127x_lora::LoRa<SharedSpi5<'static>, Pa7<Output<PushPull>>, Pa6<Output<OpenDrain>>>,
        lora_on: Pa18<Output<PushPull>>,

        adxl313: adxl313::Adxl313<SharedSpi5<'static>, Pa13<Output<PushPull>>>,
        adxl313_on: Pb14<Output<OpenDrain>>,

        tc72: microchip_tc72r_rs::Tc72<SharedSpi5<'static>, Pa15<Output<OpenDrain>>>,
        tc72_on: Pa14<Output<PushPull>>,

        delay: atsamd_hal::common::delay::Delay,
        count: u8,
        temp: f32,
        x: f32,
        y: f32,
        z: f32,

        spi: &'static Spi5,
        e25x: spi_memory::series25::Flash<SharedSpi5<'static>, Pa24<Output<OpenDrain>>>,

        lora_buf: [u8; 255],
    }

    #[init]
    fn init(c: init::Context) -> init::LateResources {
        static mut SPI5: Option<Spi5> = None;
        let mut device: hal::pac::Peripherals = c.device;
        let mut core: hal::pac::CorePeripherals = hal::pac::CorePeripherals::take().unwrap();

        let mut clocks = GenericClockController::with_external_32kosc(
            device.GCLK,
            &mut device.MCLK,
            &mut device.OSC32KCTRL,
            &mut device.OSCCTRL,
            &mut device.NVMCTRL,
        );

        let mut delay = Delay::new(core.SYST, &mut clocks);
        let gclk = clocks.gclk0();

        let mut pins = hal::Pins::new(device.PORT);
        let mut led0 = pins.led0.into_open_drain_output(&mut pins.port);
        let mut led1 = pins.led1.into_open_drain_output(&mut pins.port);
        let mut led2 = pins.led2.into_open_drain_output(&mut pins.port);
        let mut led3 = pins.led3.into_open_drain_output(&mut pins.port);

        let mut lora_cs = pins.lora_cs.into_push_pull_output(&mut pins.port);
        let mut lora_on = pins.lora_on.into_push_pull_output(&mut pins.port);
        let mut lora_reset = pins.lora_rst.into_open_drain_output(&mut pins.port);

        let mut adxl313_on = pins.acc_on.into_open_drain_output(&mut pins.port);
        let mut adxl313_cs = pins.acc_cs.into_push_pull_output(&mut pins.port);

        let mut tc72_on = pins.tc72_on.into_push_pull_output(&mut pins.port);
        let mut tc72_cs = pins.tc72_cs.into_open_drain_output(&mut pins.port);

        let mut eep_on = pins.eep_on.into_push_pull_output(&mut pins.port);
        let mut eep_cs = pins.eep_cs.into_open_drain_output(&mut pins.port);

        lora_cs.set_high().unwrap();
        adxl313_cs.set_high().unwrap();

        led0.set_high().unwrap();
        led1.set_high().unwrap();
        led2.set_high().unwrap();
        led3.set_high().unwrap();

        adxl313_on.set_high().unwrap();
        delay.delay_ms(500u32);
        adxl313_on.set_low().unwrap();
        adxl313_cs.set_high().unwrap();
        delay.delay_ms(500u32);

        *SPI5 = Some(spi_master(
            // &mut clocks.sercom5_core(&gclk).unwrap(),
            &mut clocks,
            // MegaHertz(8),
            MegaHertz(2),
            device.SERCOM5,
            &mut device.MCLK,
            // &mut device.MCLK,
            pins.spi_sck,
            pins.spi_mosi,
            pins.spi_miso,
            &mut pins.port,
        ));

        let (lora_spi, adxl313_spi, temp_spi, eep_spi) =
            shared_spi(SPI5.as_ref().unwrap(), MODE_0, MODE_1, MODE_3, MODE_3);

        lora_cs.set_low().unwrap();
        (0..10).for_each(|_| delay.delay_ms(100u32));
        lora_on.set_low().unwrap();
        lora_cs.set_high().unwrap();
        (0..10).for_each(|_| delay.delay_ms(100u32));

        let lora = sx127x_lora::LoRa::new(lora_spi, lora_cs, lora_reset, 869, &mut delay);

        if let Err(_e) = lora {
            led0.set_low().unwrap();
            led1.set_low().unwrap();
            led2.set_high().unwrap();
            led3.set_low().unwrap();
            loop {}
        }

        let lora = lora.unwrap();

        eep_on.set_low().unwrap();
        eep_cs.set_low().unwrap();
        (0..10).for_each(|_| delay.delay_ms(100u32));
        eep_on.set_low().unwrap();
        eep_cs.set_high().unwrap();
        (0..10).for_each(|_| delay.delay_ms(100u32));
        let e25x = spi_memory::series25::Flash::init_with_page_size(eep_spi, eep_cs, 256);

        if let Err(_e) = e25x {
            led0.set_high().unwrap();
            led1.set_high().unwrap();
            led2.set_low().unwrap();
            led3.set_low().unwrap();
            loop {}
        }
        let mut e25x = e25x.unwrap();
        let e25x_manufacturer = e25x.wake_up_and_get_manufacturer_id();

        if let Err(_e) = e25x_manufacturer {
            led0.set_high().unwrap();
            led1.set_high().unwrap();
            led2.set_low().unwrap();
            led3.set_low().unwrap();
            loop {}
        }

        let e25x_manufacturer = e25x_manufacturer.unwrap();

        if e25x_manufacturer != 0x29 {
            led0.set_high().unwrap();
            led1.set_high().unwrap();
            led2.set_low().unwrap();
            led3.set_low().unwrap();
            loop {}
        }

        let adxl313 = adxl313::Adxl313::new(adxl313_spi, adxl313_cs);

        if let Err(_e) = adxl313 {
            led0.set_low().unwrap();
            led1.set_high().unwrap();
            led2.set_low().unwrap();
            led3.set_low().unwrap();
            loop {}
        }

        tc72_on.set_low().unwrap();
        (0..10).for_each(|_| delay.delay_ms(100u32));

        let tc72 = microchip_tc72r_rs::Tc72::new(temp_spi, tc72_cs);

        if let Err(_e) = tc72 {
            led0.set_high().unwrap();
            led1.set_low().unwrap();
            led2.set_low().unwrap();
            led3.set_low().unwrap();
            loop {}
        }

        let adxl313 = adxl313.unwrap();
        let tc72 = tc72.unwrap();

        init::LateResources {
            led0,
            led1,
            led2,
            led3,
            lora,
            lora_on,
            delay,
            adxl313,
            adxl313_on,
            tc72,
            tc72_on,
            count: 0,
            temp: 0.0,
            x: 0.0,
            y: 0.0,
            z: 0.0,
            lora_buf: [0; 255],
            spi: SPI5.as_ref().unwrap(),
            e25x,
        }
    }

    #[task(resources = [adxl313, x, y, z])]
    fn read_adxl(c: read_adxl::Context) {
        let adxl: &mut adxl313::Adxl313<_, _> = c.resources.adxl313;
        adxl.power_control(
            true,
            false,
            false,
            true,
            false,
            SleepModeFrequencyReadings::_1_HZ,
        )
            .unwrap();
        let reading = adxl.accel_norm().unwrap();
        dbgprint!("X: {}, Y: {}, Z: {}", reading.x, reading.y, reading.z);
        *c.resources.x = reading.x;
        *c.resources.y = reading.y;
        *c.resources.z = reading.z;
    }

    #[idle(spawn = [read_adxl])]
    fn idle(cx: idle::Context) -> ! {
        loop {
            cx.spawn.read_adxl().unwrap();
        }
    }

    // Interrupt handlers used to dispatch software tasks
    extern "C" {
        fn I2S();
    }
};
