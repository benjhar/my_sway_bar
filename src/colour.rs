use serde::Serialize;

#[derive(Clone, Copy)]
pub struct Rgb([u8; 3]);

impl Rgb {
    pub const fn new(c: [u8; 3]) -> Self {
        Rgb(c)
    }
}

impl Serialize for Rgb {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_str(&format_args!(
            "#{:02X}{:02X}{:02X}",
            self.0[0], self.0[1], self.0[2]
        ))
    }
}
