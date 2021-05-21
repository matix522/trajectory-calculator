use std::ops::Add;

#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct NotNaNf64(pub f64);

pub type Score = NotNaNf64;

pub const INFINITY: NotNaNf64 = NotNaNf64(f64::INFINITY);
pub const NEG_INFINITY: NotNaNf64 = NotNaNf64(f64::NEG_INFINITY);

impl NotNaNf64 {
    pub const fn new(f: f64) -> Self {
        if f.is_nan() {
            panic!("Value was NaN");
        }
        NotNaNf64(f)
    }
    pub const fn new_checked(f: f64) -> Option<Self> {
        if f.is_nan() {
            None
        } else {
            Some(NotNaNf64(f))
        }
    }
}

impl PartialEq for NotNaNf64 {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for NotNaNf64 {}

impl PartialOrd for NotNaNf64 {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for NotNaNf64 {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        f64::partial_cmp(&self.0, &other.0).unwrap()
    }
}

impl Add<NotNaNf64> for NotNaNf64 {
    fn add(self, other: NotNaNf64) -> <Self as std::ops::Add<NotNaNf64>>::Output {
        NotNaNf64::new_checked(self.0 + other.0)
    }
    type Output = Option<NotNaNf64>;
}
