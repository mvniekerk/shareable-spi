use crate::reconfigurable_mode::ReconfigurableSpiMode;
use crate::spi_lock::SpiLock;
use embedded_hal::blocking::spi;
use embedded_hal::spi::Mode;

/// A simple wrapper that wraps something that implements the SpiLock interface - whose
/// SpiLock interface's lock() returns a device that implements the necessary Transfer/Write
/// traits but also the ReconfigurableMode trait
pub struct ShareableSpiWithConf<SPI, DEV> {
    spi: SPI,
    mode: Mode,
    _marker: core::marker::PhantomData<DEV>,
}

impl<SPI, DEV> ShareableSpiWithConf<SPI, DEV> {
    pub fn new(spi: SPI, mode: Mode) -> Self {
        ShareableSpiWithConf {
            spi,
            mode,
            _marker: Default::default(),
        }
    }

    pub fn mode(&self) -> Mode {
        self.mode
    }
}

unsafe impl<DEV, SPI> Sync for ShareableSpiWithConf<SPI, DEV> {}

/// SPI transfer
impl<SPI, DEV> spi::Transfer<u8> for ShareableSpiWithConf<SPI, DEV>
where
    SPI: SpiLock<DEV>,
    DEV: spi::Transfer<u8> + ReconfigurableSpiMode,
{
    type Error = DEV::Error;

    fn transfer<'w>(&mut self, words: &'w mut [u8]) -> Result<&'w [u8], Self::Error> {
        let mode = self.mode();
        self.spi.lock(move |dev| {
            dev.change_spi_mode(mode);
            dev.transfer(words)
        })
    }
}

/// SPI Write
impl<SPI, DEV> spi::Write<u8> for ShareableSpiWithConf<SPI, DEV>
where
    SPI: SpiLock<DEV>,
    DEV: spi::Write<u8> + ReconfigurableSpiMode,
{
    type Error = DEV::Error;

    fn write(&mut self, words: &[u8]) -> Result<(), Self::Error> {
        let mode = self.mode();
        self.spi.lock(move |dev| {
            dev.change_spi_mode(mode);
            dev.write(words)
        })
    }
}
