use crate::preludes::core::*;
use crate::preludes::math::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, reflect::Reflect)]
pub struct USize2(pub UVec2);
impl USize2 {
    pub fn width(&self) -> u32 {
        self.0.x
    }
    pub fn height(&self) -> u32 {
        self.0.y
    }
    pub fn total_size(&self) -> usize {
        self.0.x as usize * self.0.y as usize
    }
    pub fn bounds_check(&self, vec: UVec2) -> bool {
        vec.x < self.width() && vec.y < self.height()
    }
}
impl From<UVec2> for USize2 {
    fn from(value: UVec2) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default, reflect::Reflect)]
pub struct DSize2(pub DVec2);
impl DSize2 {
    pub fn new(value: DVec2) -> Self {
        value.into()
    }
    pub fn width(&self) -> f64 {
        self.0.x
    }
    pub fn height(&self) -> f64 {
        self.0.y
    }
    pub fn contains(&self, vec: DVec2) -> bool {
        vec.x >= 0.0 && vec.y >= 0.0 && vec.x <= self.width() && vec.y <= self.height()
    }
}
impl From<DVec2> for DSize2 {
    fn from(value: DVec2) -> Self {
        assert!(
            value.is_finite() && value.x >= 0.0 && value.y >= 0.0,
            "{value} is positive"
        );
        Self(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default, reflect::Reflect)]
pub struct Size2(pub Vec2);
impl Size2 {
    pub fn width(&self) -> f32 {
        self.0.x
    }
    pub fn height(&self) -> f32 {
        self.0.y
    }
    pub fn contains(&self, vec: Vec2) -> bool {
        vec.x >= 0.0 && vec.y >= 0.0 && vec.x <= self.width() && vec.y <= self.height()
    }
}
impl From<Vec2> for Size2 {
    fn from(value: Vec2) -> Self {
        assert!(
            value.is_finite() && value.x >= 0.0 && value.y >= 0.0,
            "{value} is positive"
        );
        Self(value)
    }
}
