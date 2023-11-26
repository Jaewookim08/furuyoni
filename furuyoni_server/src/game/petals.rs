
// max추가, Add 삭제하고 transfer 추가.
#[derive(Debug)]
pub(crate) struct Petals {
    count: u32,
}

impl Petals {
    pub(crate) fn new(n: u32) -> Self {
        Self { count: n }
    }
    pub(crate) fn take_available(&mut self, n: u32) -> Petals {
        let take_amount = std::cmp::min(self.count, n);
        self.count -= take_amount;
        Petals::new(take_amount)
    }
    pub(crate) fn take(&mut self, n: u32) -> Result<Petals, ()> {
        if self.count < n {
            Err(())
        } else {
            self.count -= n;
            Ok(Petals::new(n))
        }
    }

    pub(crate) fn get_count(&self) -> u32 {
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
