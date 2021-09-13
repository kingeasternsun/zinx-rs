#[derive(Debug, Clone, Copy)]
pub struct ConnID(u32);
impl ConnID {
    pub fn new(id: u32) -> Self {
        ConnID(id)
    }
}
