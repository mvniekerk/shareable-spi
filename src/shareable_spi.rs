use crate::spi_lock::SpiLock;
use core::cell::UnsafeCell;
use core::sync::atomic::{AtomicBool, Ordering};
use embedded_hal::blocking::spi;

/// Simple wrapper to share a SPI bus between multiple drivers. Will panic if two drivers attempt to
/// access it at the same time.
/// Can be used as-is when there are no mode differences between the SPI devices
pub struct SharedSpi<DEV> {
    spi: UnsafeCell<DEV>,
    busy: AtomicBool,
}

impl<SPI> SharedSpi<SPI> {
    pub fn new(spi: SPI) -> SharedSpi<SPI> {
        return SharedSpi {
            spi: UnsafeCell::new(spi),
            busy: AtomicBool::new(false)
        }
    }
}

/// Locks the SPI device or crashes. So check busy() first before you engage this.
/// After locking it, runs an FnOnce with the SPI device as parameter
impl<SPI> SpiLock<SPI> for &SharedSpi<SPI> {
    fn lock<R, F: FnOnce(&mut SPI) -> R>(&self, f: F) -> R {
        self.busy
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .expect("SPI bus conflict");

        let r = f(unsafe { &mut *self.spi.get() });

        self.busy.store(false, Ordering::SeqCst);
        r
    }

    /// Get from the atomic bool if the device is busy
    fn busy(&self) -> bool {
        self.busy.load(Ordering::Relaxed)
    }
}

unsafe impl<SPI> Sync for SharedSpi<SPI> {}

/// SPI Transfer
impl<SPI> spi::Transfer<u8> for &SharedSpi<SPI>
where
    SPI: spi::Transfer<u8>,
{
    type Error = SPI::Error;

    fn transfer<'w>(&mut self, words: &'w mut [u8]) -> Result<&'w [u8], Self::Error> {
        self.lock(move |spi| spi.transfer(words))
    }
}

/// SPI Write
impl<SPI> spi::Write<u8> for &SharedSpi<SPI>
where
    SPI: spi::Write<u8>,
{
    type Error = SPI::Error;

    fn write(&mut self, words: &[u8]) -> Result<(), Self::Error> {
        self.lock(move |spi| spi.write(words))
    }
}
