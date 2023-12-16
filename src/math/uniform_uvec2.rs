use bevy::math::UVec2;
use rand::{
    distributions::{
        uniform::{SampleBorrow, SampleUniform, UniformInt, UniformSampler},
        Distribution, Uniform,
    },
    Rng,
};

#[derive(Clone, Copy, Debug)]
pub struct UVec2Sample(pub UVec2);

#[derive(Clone, Copy, Debug)]
pub struct UniformUVec2 {
    x: UniformInt<u32>,
    y: UniformInt<u32>,
}

impl UniformSampler for UniformUVec2 {
    type X = UVec2Sample;
    fn new<B1, B2>(low: B1, high: B2) -> Self
    where
        B1: SampleBorrow<Self::X> + Sized,
        B2: SampleBorrow<Self::X> + Sized,
    {
        UniformUVec2 {
            x: UniformInt::<u32>::new(low.borrow().0.x, high.borrow().0.x),
            y: UniformInt::<u32>::new(low.borrow().0.y, high.borrow().0.y),
        }
    }
    fn new_inclusive<B1, B2>(low: B1, high: B2) -> Self
    where
        B1: SampleBorrow<Self::X> + Sized,
        B2: SampleBorrow<Self::X> + Sized,
    {
        UniformUVec2 {
            x: UniformInt::new_inclusive(low.borrow().0.x, high.borrow().0.x),
            y: UniformInt::new_inclusive(low.borrow().0.y, high.borrow().0.y),
        }
    }
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Self::X {
        UVec2Sample(UVec2::new(self.x.sample(rng), self.y.sample(rng)))
    }
}

impl SampleUniform for UVec2Sample {
    type Sampler = UniformUVec2;
}
