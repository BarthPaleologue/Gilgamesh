#[must_use]
pub fn move_towards(current: f64, target: f64, speed: f64) -> f64 {
    if current < target {
        (current + speed).min(target)
    } else {
        (current - speed).max(target)
    }
}
