pub struct Entry {
    pub text: String,
    pub action: Box<dyn Fn() -> ()>,
}
