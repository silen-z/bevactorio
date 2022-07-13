#[derive(Clone, Copy, PartialEq, Default, Eq)]
#[allow(dead_code)]
pub enum MapDirection {
    #[default]
    Up,
    Down,
    Left,
    Right,
}

pub struct Directional<T> {
    pub up: T,
    pub down: T,
    pub left: T,
    pub right: T,
}

impl<S> PartialEq<S> for MapDirection
where
    S: AsRef<str>,
{
    fn eq(&self, other: &S) -> bool {
        match self {
            Self::Up => other.as_ref() == "up",
            Self::Down => other.as_ref() == "down",
            Self::Left => other.as_ref() == "left",
            Self::Right => other.as_ref() == "right",
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
