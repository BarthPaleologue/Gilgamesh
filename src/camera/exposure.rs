/// Photographic exposure using Filament's saturation-based calibration.
///
/// The default is daylight: f/16, 1/125 second, ISO 100, with no compensation.
/// These settings affect brightness only; they do not simulate motion blur,
/// depth of field, or sensor noise.
/// See <https://github.com/google/filament/blob/main/filament/src/Exposure.cpp>.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Exposure {
    aperture_f_number: f32,
    shutter_seconds: f32,
    iso: f32,
    compensation_stops: f32,
}

impl Default for Exposure {
    fn default() -> Self {
        Self::new(16.0, 1.0 / 125.0, 100.0)
    }
}

impl Exposure {
    /// # Panics
    /// Panics if any setting is nonpositive or non-finite, or the resulting gain
    /// cannot be represented as a positive finite shader float.
    #[must_use]
    pub fn new(aperture_f_number: f32, shutter_seconds: f32, iso: f32) -> Self {
        Self {
            aperture_f_number,
            shutter_seconds,
            iso,
            compensation_stops: 0.0,
        }
        .validated()
    }

    /// Positive compensation brightens the image; +1 stop doubles linear RGB.
    ///
    /// # Panics
    /// Panics if the compensation is non-finite or the resulting gain cannot be
    /// represented as a positive finite shader float.
    #[must_use]
    pub fn with_compensation_stops(mut self, stops: f32) -> Self {
        self.compensation_stops = stops;
        self.validated()
    }

    #[must_use]
    pub const fn aperture_f_number(self) -> f32 {
        self.aperture_f_number
    }

    #[must_use]
    pub const fn shutter_seconds(self) -> f32 {
        self.shutter_seconds
    }

    #[must_use]
    pub const fn iso(self) -> f32 {
        self.iso
    }

    #[must_use]
    pub const fn compensation_stops(self) -> f32 {
        self.compensation_stops
    }

    /// Exposure value at ISO 100, before compensation.
    #[must_use]
    #[expect(
        clippy::cast_possible_truncation,
        reason = "Exposure values use shader f32 precision"
    )]
    pub fn ev100(self) -> f32 {
        self.ev100_scale().log2() as f32
    }

    /// Gain used by the fullscreen exposure pass to convert scene luminance to exposed linear RGB.
    #[must_use]
    #[expect(
        clippy::cast_possible_truncation,
        reason = "The constructor validates the gain after narrowing to shader f32"
    )]
    pub fn gain(self) -> f32 {
        (f64::from(self.compensation_stops).exp2() / (1.2 * self.ev100_scale())) as f32
    }

    fn ev100_scale(self) -> f64 {
        f64::from(self.aperture_f_number).powi(2) / f64::from(self.shutter_seconds)
            * (100.0 / f64::from(self.iso))
    }

    fn validated(self) -> Self {
        for value in [self.aperture_f_number, self.shutter_seconds, self.iso] {
            assert!(
                value.is_finite() && value > 0.0,
                "Exposure settings must be finite and positive"
            );
        }
        assert!(
            self.compensation_stops.is_finite(),
            "Exposure compensation must be finite"
        );
        let gain = self.gain();
        assert!(
            gain.is_finite() && gain > 0.0,
            "Exposure gain must fit a positive finite shader float"
        );
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn daylight_defaults_match_filament_calibration() {
        let exposure = Exposure::default();
        assert!((exposure.ev100() - 14.965_784).abs() < 1e-5);
        assert!((exposure.gain() - 1.0 / 38_400.0).abs() < 1e-10);
    }

    #[test]
    fn photographic_stops_scale_linear_light() {
        let base = Exposure::new(4.0, 1.0 / 60.0, 100.0);
        for exposure in [
            Exposure::new(4.0, 1.0 / 60.0, 200.0),
            Exposure::new(4.0, 1.0 / 30.0, 100.0),
            base.with_compensation_stops(1.0),
        ] {
            assert!((exposure.gain() / base.gain() - 2.0).abs() < 1e-6);
        }
        let smaller_aperture = Exposure::new(8.0, 1.0 / 60.0, 100.0);
        assert!((smaller_aperture.gain() / base.gain() - 0.25).abs() < 1e-6);
        assert!((base.with_compensation_stops(-1.0).gain() / base.gain() - 0.5).abs() < 1e-6);
    }

    #[test]
    fn invalid_settings_and_unrepresentable_gains_are_rejected() {
        for value in [0.0, -1.0, f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            assert!(std::panic::catch_unwind(|| Exposure::new(value, 1.0, 100.0)).is_err());
            assert!(std::panic::catch_unwind(|| Exposure::new(4.0, value, 100.0)).is_err());
            assert!(std::panic::catch_unwind(|| Exposure::new(4.0, 1.0, value)).is_err());
        }
        for stops in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY, 1e6, -1e6] {
            assert!(
                std::panic::catch_unwind(|| Exposure::default().with_compensation_stops(stops))
                    .is_err()
            );
        }
    }
}
