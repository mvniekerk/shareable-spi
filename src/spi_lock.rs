/// A naive lock trait.
/// It comes with a busy() indicator
pub trait SpiLock<SHARED> {
    fn lock<R, F: FnOnce(&mut SHARED) -> R>(&self, f: F) -> R;
    fn busy(&self) -> bool;
}
