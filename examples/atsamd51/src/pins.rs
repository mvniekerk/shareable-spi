//! PyPortal pins
use atsamd_hal::{hal, target_device, define_pins};

use atsamd_hal::common::gpio::{self, *};

define_pins!(
    /// Maps the pins to their arduino names and
    /// the numbers printed on the board.
    struct Pins,
    target_device: target_device,

    /// SPI
    pin spi_sck = a22,
    pin spi_mosi = a23,
    pin spi_miso = a25,

    /// LoRa
    pin lora_on = a18,
    pin lora_cs = a7,
    pin lora_rst = a6,
    pin lora_dio0 = b6,
    pin lora_dio1 = b7,
    pin lora_dio2 = b8,
    pin lora_dio3 = b9,
    pin lora_dio4 = a4,
    pin lora_dio5 = a5,

    /// ADXL313
    pin acc_int1 = b15,
    pin acc_int2 = a12,
    pin acc_cs = a13,
    pin acc_on = b14,

    // TC72
    pin tc72_on = a14,
    pin tc72_cs = a15,

    // EEP
    pin eep_on = a27,
    pin eep_cs = a24,

);

impl Pins {
    /// Split the device pins into subsets
    pub fn split(self) -> Sets {
        let leds = Leds {
            led0: self.led0,
            led1: self.led1,
            led2: self.led2,
            led3: self.led3
        };
        let lora = LoRa {
            dio0: self.lora_dio0,
            dio1: self.lora_dio1,
            dio2: self.lora_dio2,
            dio3: self.lora_dio3,
            dio4: self.lora_dio4,
            dio5: self.lora_dio5,
            reset: self.lora_rst,
            cs: self.lora_cs,
            on: self.lora_on
        };
        let spi = Spi {
            sck: self.spi_sck,
            mosi: self.spi_mosi,
            miso: self.spi_miso
        };
        let accl = Accelerometer {
            cs: self.acc_cs,
            on: self.acc_on,
            int1: self.acc_int1,
            int2: self.acc_int2
        };

        Sets {
            port: self.port,
            leds,
            lora,
            spi,
            accl
        }
    }
}

pub struct Sets {
    pub port: Port,
    pub leds: Leds,
    pub lora: LoRa,
    pub spi: Spi,
    pub accl: Accelerometer
}

pub struct Leds {
    pub led0: Pa19<Input<Floating>>,
    pub led1: Pb16<Input<Floating>>,
    pub led2: Pb17<Input<Floating>>,
    pub led3: Pa20<Input<Floating>>
}

pub struct LoRa {
    pub dio0: Pb6<Input<Floating>>,
    pub dio1: Pb7<Input<Floating>>,
    pub dio2: Pb8<Input<Floating>>,
    pub dio3: Pb9<Input<Floating>>,
    pub dio4: Pa4<Input<Floating>>,
    pub dio5: Pa5<Input<Floating>>,
    pub reset: Pa6<Input<Floating>>,
    pub cs: Pa7<Input<Floating>>,
    pub on: Pa18<Input<Floating>>,
}

pub struct Spi {
    pub sck: Pa22<Input<Floating>>,
    pub mosi: Pa23<Input<Floating>>,
    pub miso: Pa25<Input<Floating>>
}

pub struct Accelerometer {
    pub cs: Pa13<Input<Floating>>,
    pub on: Pb14<Input<Floating>>,
    pub int1: Pb15<Input<Floating>>,
    pub int2: Pa12<Input<Floating>>
}