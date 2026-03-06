use crate::constants::{FIVE_PM_UTC, MARGIN, NINE_AM_UTC, SECONDS_IN_A_DAY};

pub fn transfer_allowed(current_time: i64) -> bool {
    let seconds_since_midnight = current_time % SECONDS_IN_A_DAY;
    seconds_since_midnight >= NINE_AM_UTC && seconds_since_midnight <= FIVE_PM_UTC
}

pub fn is_within_timerange(current_time: i64) -> bool {
    let seconds_since_midnight = current_time % SECONDS_IN_A_DAY;
    (seconds_since_midnight >= NINE_AM_UTC && seconds_since_midnight < NINE_AM_UTC + MARGIN)
        || (seconds_since_midnight >= FIVE_PM_UTC && seconds_since_midnight < FIVE_PM_UTC + MARGIN)
}
