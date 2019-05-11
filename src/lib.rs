pub struct Generator<T: std::ops::Add> {
    last_returned: T,
    increment: T,
}

impl <T: std::ops::Add> Generator <T> {
    pub fn new<U>(v: U, i: U) -> Generator<U>
    where U: std::ops::Add<Output=U> + Copy + Clone {
        return Generator{last_returned: v, increment: i}
    }

    pub fn generate(&mut self) -> T
    where T: std::ops::Add<Output=T> + Copy + Clone 
    {
        self.last_returned = self.last_returned + self.increment;
        return self.last_returned
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_returns_something() {
        let mut g = Generator::<i32>::new(0, 1);
        g.generate();
    }

    #[test]
    fn it_returns_increasing_values() {
        let mut g = Generator::<i32>::new(0,1);

        let (first, second) = (g.generate(), g.generate());

        assert!(first < second)
    }
}
