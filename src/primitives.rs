#[derive(Debug)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Color {
        Color { r, g, b, a }
    }
}

#[derive(Debug)]
pub struct Rectangle<T> {
    pub left: T,
    pub top: T,
    pub width: T,
    pub height: T,
}
