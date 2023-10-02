pub fn move_towards(current: f32, target: f32, speed: f32) -> f32 {
    if current < target {
        (current + speed).min(target)
    } else {
        (current - speed).max(target)
    }
}