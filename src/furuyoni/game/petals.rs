#[derive(Debug)]
pub(crate) struct Petals {
    count: i32,
}

impl Petals {
    pub(crate) fn new(n: i32) -> Self {
        Self { count: n }
    }
    pub(crate) fn take(&mut self, n: i32) -> Petals {
        let take_amount = std::cmp::min(self.count, n);
        self.count -= take_amount;
        Petals::new(take_amount)
    }
    pub(crate) fn get_count(&self) -> i32 {
        self.count
    }
}

impl std::ops::Add<Petals> for Petals {
    type Output = Petals;

    fn add(mut self, rhs: Petals) -> Petals {
        self += rhs;
        self
    }
}

impl std::ops::AddAssign<Petals> for Petals {
    fn add_assign(&mut self, rhs: Petals) {
        self.count += rhs.count
    }
}
