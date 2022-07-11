#[derive(Clone, Copy, PartialEq, Default, Eq)]
#[allow(dead_code)]
pub enum MapDirection {
    #[default]
    Down,
    Up,
    Left,
    Right,
}

pub struct Directional<T> {
    up: T,
    down: T,
    left: T,
    right: T,
}

impl<T> Directional<T> {
    pub fn map<U>(self, f: impl Fn(T) -> U) -> Directional<U> {
        Directional {
            up: f(self.up),
            down: f(self.down),
            left: f(self.left),
            right: f(self.right),
        }
    }
}

impl<T: Clone> Directional<T> {
    pub fn all(value: T) -> Self {
        Self {
            up: value.clone(),
            down: value.clone(),
            left: value.clone(),
            right: value.clone(),
        }
    }
}

impl<T: Clone> Clone for Directional<T> {
    fn clone(&self) -> Self {
        Self {
            up: self.up.clone(),
            down: self.down.clone(),
            left: self.left.clone(),
            right: self.right.clone(),
        }
    }
}

impl<T> std::ops::Index<MapDirection> for Directional<T> {
    type Output = T;

    fn index(&self, index: MapDirection) -> &Self::Output {
        match index {
            MapDirection::Up => &self.up,
            MapDirection::Down => &self.down,
            MapDirection::Left => &self.left,
            MapDirection::Right => &self.right,
        }
    }
}
