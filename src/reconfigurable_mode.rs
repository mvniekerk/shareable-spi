use embedded_hal::spi::Mode;

/// A trait for an SPI device that can change its CPOL and CPHA mode
pub trait ReconfigurableSpiMode {
    fn change_spi_mode(&mut self, mode: Mode);
}