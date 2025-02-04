//! Architectural timer primitives.

use crate::warn;
use aarch64_cpu::{asm::barrier, registers::*};
use core::{
    num::{NonZeroU128, NonZeroU32, NonZeroU64},
    ops::{Add, Div},
    time::Duration,
};
use tock_registers::interfaces::Readable;

const NANOSEC_PER_SEC: NonZeroU64 = NonZeroU64::new(1_000_000_000).unwrap();

#[derive(Copy, Clone, PartialOrd, PartialEq)]
struct GenericTimerCounterValue(u64);

/// Boot assembly code overwrites this value with the value of CNTFRQ_EL0
#[no_mangle] // so compiler doesnt change anything
static ARCH_TIMER_COUNTER_FREQUENCY: NonZeroU32 = NonZeroU32::MIN;

// return the frequency of the counter
fn arch_timer_counter_freq() -> NonZeroU32 {
    unsafe { core::ptr::read_volatile(&ARCH_TIMER_COUNTER_FREQUENCY) }
}

impl GenericTimerCounterValue {
    pub const MAX: Self = GenericTimerCounterValue(u64::MAX);
}

// addition operator overloading
impl Add for GenericTimerCounterValue {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        GenericTimerCounterValue(self.0.wrapping_add(other.0))
    }
}

// convert timer counter value to a Duration
impl From<GenericTimerCounterValue> for Duration {
    fn from(counter_value: GenericTimerCounterValue) -> Self {
        if counter_value.0 == 0 {
            return Duration::ZERO;
        }

        // secs = cnt / freq
        let freq: NonZeroU64 = arch_timer_counter_freq().into();
        let sec = counter_value.0.div(freq); // get whole number of secs

        let rem_sec = counter_value.0 % freq;
        let nanos = unsafe { rem_sec.unchecked_mul(u64::from(NANOSEC_PER_SEC)) }.div(freq) as u32;

        Duration::new(sec, nanos)
    }
}

fn max_duration() -> Duration {
    Duration::from(GenericTimerCounterValue::MAX)
}

// convert duration to a timer counter value
// NOTE: TryFrom since this can result in an error
impl TryFrom<Duration> for GenericTimerCounterValue {
    type Error = &'static str;

    // returns a GenericTimer with the error(None if no error)
    fn try_from(dur: Duration) -> Result<Self, Self::Error> {
        if dur < resolution() {
            return Ok(GenericTimerCounterValue(0));
        }

        if dur > max_duration() {
            return Err("Conversion error. Duration too big");
        }

        let freq: u128 = u32::from(arch_timer_counter_freq()) as u128;
        let duration: u128 = dur.as_nanos();

        let counter_value =
            unsafe { duration.unchecked_mul(freq) }.div(NonZeroU128::from(NANOSEC_PER_SEC));

        Ok(GenericTimerCounterValue(counter_value as u64))
    }
}

#[inline(always)]
fn read_cntpct() -> GenericTimerCounterValue {
    barrier::isb(barrier::SY); // block OOO execution
    let cnt = CNTPCT_EL0.get();

    GenericTimerCounterValue(cnt)
}

/// The timer's resolution.
pub fn resolution() -> Duration {
    Duration::from(GenericTimerCounterValue(1))
}

/// The uptime since power-on of the device.
pub fn uptime() -> Duration {
    read_cntpct().into()
}

/// Spin for a given duration.
pub fn spin_for(duration: Duration) {
    let curr_counter_value = read_cntpct();

    let counter_value_delta: GenericTimerCounterValue = match duration.try_into() {
        Err(msg) => {
            warn!("spin_for: {}. Skipping", msg);
            return;
        }
        Ok(val) => val,
    };
    let counter_value_target = curr_counter_value + counter_value_delta;

    // spin
    while GenericTimerCounterValue(CNTPCT_EL0.get()) < counter_value_target {}
}
