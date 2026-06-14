pub trait Collision {
    type Collider;

    fn collides(&self, collider: Self::Collider) -> bool;
}
