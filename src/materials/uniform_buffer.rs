pub struct UniformBuffer<T: Default> {
    label: String,
    init_value: T,
}