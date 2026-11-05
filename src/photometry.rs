//! Photometric quantities for rendering. World-space distances are measured in meters.
//!
//! As in glTF, RGB light colors are linear multipliers of the intensity of a white source.

/// Luminous intensity in candelas (lumens per steradian).
///
/// ```compile_fail,E0308
/// use gilgamesh::lights::DirectionalLight;
/// use gilgamesh::photometry::Candela;
/// DirectionalLight::default().set_illuminance(Candela::new(100.0));
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd)]
pub struct Candela(f32);

impl Candela {
    /// # Panics
    /// Panics if the intensity is negative or non-finite.
    #[must_use]
    pub const fn new(candelas: f32) -> Self {
        assert!(
            candelas.is_finite() && candelas >= 0.0,
            "Candelas must be finite and nonnegative"
        );
        Self(candelas)
    }

    /// Converts the total luminous flux of an isotropic point source to candelas.
    ///
    /// # Panics
    /// Panics if the flux is negative or non-finite.
    #[must_use]
    pub fn from_lumens(lumens: f32) -> Self {
        assert!(
            lumens.is_finite() && lumens >= 0.0,
            "Lumens must be finite and nonnegative"
        );
        Self::new(lumens / (4.0 * std::f32::consts::PI))
    }

    #[must_use]
    pub const fn value(self) -> f32 {
        self.0
    }
}

/// Illuminance in lux (lumens per square meter).
#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd)]
pub struct Lux(f32);

impl Lux {
    /// # Panics
    /// Panics if the illuminance is negative or non-finite.
    #[must_use]
    pub const fn new(lux: f32) -> Self {
        assert!(
            lux.is_finite() && lux >= 0.0,
            "Lux must be finite and nonnegative"
        );
        Self(lux)
    }

    #[must_use]
    pub const fn value(self) -> f32 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn isotropic_flux_is_spread_over_four_pi_steradians() {
        let intensity = Candela::from_lumens(800.0);
        assert!((intensity.value() - 63.661_976).abs() < 1e-5);
        assert_eq!(Candela::from_lumens(0.0), Candela::default());
    }

    #[test]
    fn quantities_reject_negative_and_nonfinite_inputs() {
        for value in [-1.0, f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            assert!(std::panic::catch_unwind(|| Candela::new(value)).is_err());
            assert!(std::panic::catch_unwind(|| Candela::from_lumens(value)).is_err());
            assert!(std::panic::catch_unwind(|| Lux::new(value)).is_err());
        }
        assert_eq!(Candela::new(0.0).value(), 0.0);
        assert_eq!(Lux::new(0.0).value(), 0.0);
    }
}
