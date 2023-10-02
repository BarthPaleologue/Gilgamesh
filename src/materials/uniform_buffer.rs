pub struct UniformBuffer<T: Default> {
    label: String,
    init_value: T,
}

impl<T: Default> UniformBuffer<T> {
    pub fn new(label: &str) -> Self {
        UniformBuffer {
            label: label.to_string(),
            init_value: T::default(),
        }
    }
}