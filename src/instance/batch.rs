use hex::{glium::texture::RawImage2d, id};
use std::{rc::Rc, sync::atomic::AtomicUsize};

pub fn bid() -> usize {
    static COUNT: AtomicUsize = AtomicUsize::new(0);

    id(&COUNT)
}

#[derive(Clone)]
pub struct Batch<'a> {
    pub buffer: Rc<RawImage2d<'a, u8>>,
    id: usize,
}

impl<'a> Batch<'a> {
    pub fn new(buffer: Rc<RawImage2d<'a, u8>>) -> Self {
        Self { buffer, id: bid() }
    }

    pub fn get(&self) -> usize {
        self.id
    }
}
