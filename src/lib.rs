#![no_std]
#![recursion_limit = "1024"]
extern crate embedded_hal;

mod reconfigurable_mode;
mod shareable_spi;
mod shareable_spi_with_conf;
mod spi_lock;

pub use reconfigurable_mode::ReconfigurableSpiMode;
pub use shareable_spi::SharedSpi;
pub use shareable_spi_with_conf::ShareableSpiWithConf;
pub use spi_lock::SpiLock;
