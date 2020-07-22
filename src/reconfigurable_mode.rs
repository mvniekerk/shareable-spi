use embedded_hal::spi::Mode;

/// A trait for an SPI device that can change its CPOL and CPHA mode
/// This is used with the ShareableSpiWithConf struct that, before transferring, changes
/// the mode the SPI device is in.
pub trait ReconfigurableSpiMode {
    /// Change the mode of the SPI device
    fn change_spi_mode(&mut self, mode: Mode);
}
