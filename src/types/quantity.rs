use std::ops;

use typed_floats::{
    tf64::{NonNaNFinite, PositiveFinite, StrictlyPositiveFinite},
    InvalidNumber,
};

use crate::Complex64;

/// Sample rate in Hz.
///
/// Valid range: \(0, ∞\)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SampleRate(pub StrictlyPositiveFinite);

impl SampleRate {
    /// Creates a new sample rate.
    ///
    /// # Examples
    ///
    /// ```
    /// use bosing::types::SampleRate;
    /// use typed_floats::InvalidNumber;
    ///
    /// assert!(matches!(SampleRate::new(44100.0), Ok(_)));
    /// assert!(matches!(SampleRate::new(0.0), Err(InvalidNumber::Zero)));
    /// assert!(matches!(SampleRate::new(f64::INFINITY), Err(InvalidNumber::Infinite)));
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the value is not a positive non-zero finite number.
    pub fn new(value: f64) -> Result<Self, InvalidNumber> {
        StrictlyPositiveFinite::new(value).map(Self)
    }

    /// Returns the value of the sample rate.
    #[must_use]
    pub fn value(self) -> f64 {
        self.0.into()
    }

    /// Returns the time step in seconds.
    #[must_use]
    pub fn dt(self) -> Duration {
        Duration(self.0.recip().into())
    }
}

/// Span of time in seconds.
///
/// Valid range: \[0, ∞\)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Duration(pub PositiveFinite);

impl Duration {
    /// Creates a new duration.
    ///
    /// # Examples
    ///
    /// ```
    /// use bosing::types::Duration;
    /// use typed_floats::InvalidNumber;
    ///
    /// assert!(matches!(Duration::new(1e-9), Ok(_)));
    /// assert!(matches!(SampleRate::new(0.0), Ok(_)));
    /// assert!(matches!(SampleRate::new(f64::INFINITY), Err(InvalidNumber::Infinite)));
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the value is not a positive finite number.
    pub fn new(value: f64) -> Result<Self, InvalidNumber> {
        PositiveFinite::new(value).map(Self)
    }

    /// Returns the value of the duration.
    #[must_use]
    pub fn value(self) -> f64 {
        self.0.into()
    }
}

/// Time instant in seconds.
///
/// Valid range: \(-∞, ∞\)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Time(pub NonNaNFinite);

impl Time {
    /// Creates a new time instant.
    ///
    /// # Examples
    ///
    /// ```
    /// use bosing::types::Time;
    /// use typed_floats::InvalidNumber;
    ///
    /// assert!(matches!(Time::new(1.0), Ok(_)));
    /// assert!(matches!(Time::new(-1.0), Ok(_)));
    /// assert!(matches!(Time::new(f64::INFINITY), Err(InvalidNumber::Infinite)));
    /// ```
    pub fn new(value: f64) -> Result<Self, InvalidNumber> {
        NonNaNFinite::new(value).map(Self)
    }

    /// Returns the value of the time instant.
    #[must_use]
    pub fn value(self) -> f64 {
        self.0.into()
    }
}

impl ops::Add<Duration> for Time {
    type Output = Self;

    fn add(self, rhs: Duration) -> Self::Output {
        let output = (self.0 + rhs.0)
            .try_into()
            .expect("Should not produce Infinities");
        Self(output)
    }
}

/// Frequency in Hz.
///
/// Valid range: \(-∞, ∞\)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Frequency(pub NonNaNFinite);

impl Frequency {
    /// Creates a new frequency.
    ///
    /// # Examples
    ///
    /// ```
    /// use bosing::types::Frequency;
    /// use typed_floats::InvalidNumber;
    ///
    /// assert!(matches!(Frequency::new(440.0), Ok(_)));
    /// assert!(matches!(Frequency::new(-440.0), Ok(_)));
    /// assert!(matches!(Frequency::new(f64::INFINITY), Err(InvalidNumber::Infinite)));
    /// ```
    pub fn new(value: f64) -> Result<Self, InvalidNumber> {
        NonNaNFinite::new(value).map(Self)
    }

    /// Returns the value of the frequency.
    #[must_use]
    pub fn value(self) -> f64 {
        self.0.into()
    }
}

/// Number of cycles.
///
/// Valid range: \(-∞, ∞\)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Phase(pub NonNaNFinite);

impl Phase {
    /// Creates a new phase.
    ///
    /// # Examples
    ///
    /// ```
    /// use bosing::types::Phase;
    /// use typed_floats::InvalidNumber;
    ///
    /// assert!(matches!(Phase::new(0.5), Ok(_)));
    /// assert!(matches!(Phase::new(-0.5), Ok(_)));
    /// assert!(matches!(Phase::new(f64::INFINITY), Err(InvalidNumber::Infinite)));
    /// ```
    pub fn new(value: f64) -> Result<Self, InvalidNumber> {
        NonNaNFinite::new(value).map(Self)
    }

    /// Returns the value of the phase.
    #[must_use]
    pub fn value(self) -> f64 {
        self.0.into()
    }

    /// Returns the phase in radians.
    #[must_use]
    pub fn radians(self) -> f64 {
        self.value() * std::f64::consts::TAU
    }

    /// Returns the phase vector as a complex number.
    #[must_use]
    pub fn phasor(self) -> Complex64 {
        Complex64::from_polar(1.0, self.radians())
    }
}

/// Amplitude.
///
/// Valid range: \(-∞, ∞\)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Amplitude(pub NonNaNFinite);

impl Amplitude {
    /// Creates a new amplitude.
    ///
    /// # Examples
    ///
    /// ```
    /// use bosing::types::Amplitude;
    /// use typed_floats::InvalidNumber;
    ///
    /// assert!(matches!(Amplitude::new(0.5), Ok(_)));
    /// assert!(matches!(Amplitude::new(-0.5), Ok(_)));
    /// assert!(matches!(Amplitude::new(f64::INFINITY), Err(InvalidNumber::Infinite)));
    /// ```
    pub fn new(value: f64) -> Result<Self, InvalidNumber> {
        NonNaNFinite::new(value).map(Self)
    }

    /// Returns the value of the amplitude.
    #[must_use]
    pub fn value(self) -> f64 {
        self.0.into()
    }
}
