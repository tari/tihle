use sdl2::render::{Canvas, RenderTarget};

pub struct Display<T>
where
    T: RenderTarget,
{
    canvas: Canvas<T>,
}

impl<T: RenderTarget> Display<T> {
    pub fn new(canvas: Canvas<T>) -> Self {
        Display { canvas }
    }
}
