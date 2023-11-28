use std::marker::PhantomData;

use crate::drect::DRect;

pub trait Space {
    type Scalar;
}

pub struct Screen {
    rect: DRect,
}
impl Space for Screen {
    type Scalar = u32;
}

pub struct World {
    rect: DRect,
}
impl Space for World {
    type Scalar = f32;
}

pub struct Sim {
    rect: DRect,
}
impl Space for Sim {
    type Scalar = f64;
}

pub struct Coord2<T: Space> {
    data: [T::Scalar; 2],
    marker: PhantomData<T>,
}

impl<T: Space> Coord2<T> {
    pub fn x(&self) -> &T::Scalar {
        &self.data[0]
    }
    pub fn y(&self) -> &T::Scalar {
        &self.data[1]
    }
}
