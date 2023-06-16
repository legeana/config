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

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone)]
    struct Test {
        x: String,
    }

    #[test]
    fn test_box_clone() {
        let t = Test {
            x: "test".to_owned(),
        };
        let b = t.box_clone();
        assert_eq!(b.x, "test".to_owned());
    }
}
