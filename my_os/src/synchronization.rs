// ! The mutex

use core::cell::UnsafeCell;

// synchronization interface (for the hardware)
pub mod interface {
    pub trait Mutex {
        type Data; // type of data for the mutex
        fn lock<'a, R>(&'a self, f: impl FnOnce(&'a mut Self::Data) -> R) -> R; // lock the mutex
    }
}

// only works when kernel is executing single-threaded, aka only running on a single core with
// interrupts disabled
pub struct SpinLock<T>
where
    T: ?Sized,
{
    data: UnsafeCell<T>,
}

unsafe impl<T> Send for SpinLock<T> where T: ?Sized + Send {}
unsafe impl<T> Sync for SpinLock<T> where T: ?Sized + Send {}

impl<T> SpinLock<T> {
    pub const fn new(data: T) -> Self {
        Self {
            data: UnsafeCell::new(data),
        }
    }
}

impl<T> interface::Mutex for SpinLock<T> {
    type Data = T;

    fn lock<'a, R>(&'a self, f: impl FnOnce(&'a mut Self::Data) -> R) -> R {
        // mutable reference will ever only be given out once at a time.
        let data = unsafe { &mut *self.data.get() }; // get pointer to item

        return f(data); // return the function's result from the data
    }
}
