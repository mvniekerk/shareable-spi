use embedded_hal::spi::Mode;
use embedded_hal::blocking::spi;
use crate::spi_lock::SpiLock;
use crate::reconfigurable_mode::ReconfigurableSpiMode;
use embedded_hal::blocking::spi::Transfer;

pub struct SharedSpiWithConf<SPI, DEV> {
    spi: SPI,
    mode: Mode,
    _marker: core::marker::PhantomData<DEV>
}

impl<SPI, DEV> SharedSpiWithConf<SPI, DEV> {
    pub fn new(spi: SPI, mode: Mode) -> Self
    {
        SharedSpiWithConf {
            spi,
            mode,
            _marker: Default::default()
        }
    }

    pub fn mode(&self) -> Mode {
        self.mode
    }
}

unsafe impl<DEV, SPI> Sync for SharedSpiWithConf<SPI, DEV> {}

impl<SPI, DEV> spi::Transfer<u8> for SharedSpiWithConf<SPI, DEV>
    where
        SPI: SpiLock<DEV>,
        DEV: spi::Transfer<u8> + ReconfigurableSpiMode
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

impl<SPI, DEV> spi::Write<u8> for SharedSpiWithConf<SPI, DEV>
    where
        SPI: SpiLock<DEV>,
        DEV: spi::Write<u8> + ReconfigurableSpiMode
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
