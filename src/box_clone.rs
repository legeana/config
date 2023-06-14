pub trait BoxClone<T> {
    fn box_clone(&self) -> Box<T>;
}

impl<T> BoxClone<T> for T
where
    T: Clone,
{
    fn box_clone(&self) -> Box<T> {
        Box::new(self.clone())
    }
}
