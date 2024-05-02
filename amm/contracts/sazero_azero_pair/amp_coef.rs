use amm_helpers::ensure;
use traits::{MathError, StablePoolError};

pub type Timestamp = u64;

/// Minimum ramp duration, in milisec.
pub const MIN_RAMP_DURATION: Timestamp = 86400000;
/// Min amplification coefficient.
pub const MIN_AMP: u128 = 1;
/// Max amplification coefficient.
pub const MAX_AMP: u128 = 1_000_000;
/// Max amplification change.
pub const MAX_AMP_CHANGE: u128 = 10;

#[derive(Default, Debug, scale::Encode, scale::Decode, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(
    feature = "std",
    derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
)]
pub struct AmplificationCoefficient {
    /// Initial amplification coefficient.
    init_amp_coef: u128,
    /// Target for ramping up amplification coefficient.
    target_amp_coef: u128,
    /// Initial amplification time.
    init_amp_time: Timestamp,
    /// Stop ramp up amplification time.
    stop_amp_time: Timestamp,
}

impl AmplificationCoefficient {
    pub fn new(init_amp_coef: u128) -> Self {
        Self {
            init_amp_coef,
            target_amp_coef: init_amp_coef,
            init_amp_time: 0,
            stop_amp_time: 0,
        }
    }
    /// from https://github.com/ref-finance/ref-contracts/blob/752f42d7ec67b66fadda7756ed7eb3d312fb6473/ref-exchange/src/stable_swap/math.rs#L100-L101
    pub fn compute_amp_coef(&self, current_time: u64) -> Result<u128, MathError> {
        if current_time < self.stop_amp_time {
            let time_range = self
                .stop_amp_time
                .checked_sub(self.init_amp_time)
                .ok_or(MathError::SubUnderflow(1))?;
            let time_delta = current_time
                .checked_sub(self.init_amp_time)
                .ok_or(MathError::SubUnderflow(2))?;

            // Compute amp factor based on ramp time
            if self.target_amp_coef >= self.init_amp_coef {
                // Ramp up
                let amp_range = self
                    .target_amp_coef
                    .checked_sub(self.init_amp_coef)
                    .ok_or(MathError::SubUnderflow(3))?;
                let amp_delta = amp_range
                    .checked_mul(time_delta as u128)
                    .ok_or(MathError::MulOverflow(1))?
                    .checked_div(time_range as u128)
                    .ok_or(MathError::DivByZero(1))?;
                self.init_amp_coef
                    .checked_add(amp_delta)
                    .ok_or(MathError::AddOverflow(1))
            } else {
                // Ramp down
                let amp_range = self
                    .init_amp_coef
                    .checked_sub(self.target_amp_coef)
                    .ok_or(MathError::SubUnderflow(4))?;
                let amp_delta = amp_range
                    .checked_mul(time_delta as u128)
                    .ok_or(MathError::MulOverflow(2))?
                    .checked_div(time_range as u128)
                    .ok_or(MathError::DivByZero(2))?;
                self.init_amp_coef
                    .checked_sub(amp_delta)
                    .ok_or(MathError::SubUnderflow(5))
            }
        } else {
            // when stop_ramp_ts == 0 or current_ts >= stop_ramp_ts
            Ok(self.target_amp_coef)
        }
    }

    pub fn ramp_amp_coef(
        &mut self,
        target_amp_coef: u128,
        ramp_duration: u64,
        current_time: u64,
    ) -> Result<(), StablePoolError> {
        ensure!(target_amp_coef >= MIN_AMP, StablePoolError::AmpCoefTooLow);
        ensure!(target_amp_coef <= MAX_AMP, StablePoolError::AmpCoefTooHigh);
        ensure!(
            ramp_duration >= MIN_RAMP_DURATION,
            StablePoolError::AmpCoefRampDurationTooShort
        );
        let current_amp_coef = self.compute_amp_coef(current_time)?;
        ensure!(
            (target_amp_coef >= current_amp_coef
                && target_amp_coef <= current_amp_coef * MAX_AMP_CHANGE)
                || (target_amp_coef < current_amp_coef
                    && target_amp_coef * MAX_AMP_CHANGE >= current_amp_coef),
            StablePoolError::AmpCoefChangeTooLarge
        );
        self.init_amp_coef = current_amp_coef;
        self.init_amp_time = current_time;
        self.target_amp_coef = target_amp_coef;
        self.stop_amp_time = current_time + ramp_duration;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use traits::StablePoolError;

    use crate::sazero_azero_pair::AmplificationCoefficient;

    #[test]
    fn amp_coef_up() {
        let amp_coef = AmplificationCoefficient {
            init_amp_coef: 100,
            target_amp_coef: 1000,
            init_amp_time: 100,
            stop_amp_time: 1600,
        };
        assert_eq!(amp_coef.compute_amp_coef(100), Ok(100));
        assert_eq!(amp_coef.compute_amp_coef(850), Ok(550));
        assert_eq!(amp_coef.compute_amp_coef(1600), Ok(1000));
    }

    #[test]
    fn amp_coef_down() {
        let amp_coef = AmplificationCoefficient {
            init_amp_coef: 1000,
            target_amp_coef: 100,
            init_amp_time: 100,
            stop_amp_time: 1600,
        };
        assert_eq!(amp_coef.compute_amp_coef(100), Ok(1000));
        assert_eq!(amp_coef.compute_amp_coef(850), Ok(550));
        assert_eq!(amp_coef.compute_amp_coef(1600), Ok(100));
    }

    #[test]
    fn amp_coef_change_duration() {
        let mut amp_coef = AmplificationCoefficient {
            init_amp_coef: 1000,
            target_amp_coef: 100,
            init_amp_time: 100,
            stop_amp_time: 1600,
        };
        let one_day: u64 = 86400000;
        assert_eq!(amp_coef.ramp_amp_coef(1000, one_day - 1, 100), Err(StablePoolError::AmpCoefRampDurationTooShort));
        assert_eq!(amp_coef.ramp_amp_coef(1000, one_day, 100), Ok(()));
    }
    
    #[test]
    fn amp_coef_change_too_large() {
        let mut amp_coef = AmplificationCoefficient {
            init_amp_coef: 100,
            target_amp_coef: 100,
            init_amp_time: 100,
            stop_amp_time: 1600,
        };
        let one_day: u64 = 86400000;
        assert_eq!(amp_coef.ramp_amp_coef(1001, one_day, 100), Err(StablePoolError::AmpCoefChangeTooLarge));
        assert_eq!(amp_coef.ramp_amp_coef(1000, one_day, 100), Ok(()));
        assert_eq!(amp_coef.ramp_amp_coef(99, one_day, one_day + 100), Err(StablePoolError::AmpCoefChangeTooLarge));
        assert_eq!(amp_coef.ramp_amp_coef(100, one_day, one_day + 100), Ok(()));
    }
}
