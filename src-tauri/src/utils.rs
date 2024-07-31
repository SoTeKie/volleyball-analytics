pub trait Tappable<T> 
where Self: Sized
{
    fn tap_some(self, f: impl FnOnce(&T)) -> Self;
    fn utap_some(self, f: impl FnOnce()) -> Self;
    fn tap_nome(self, f: impl FnOnce()) -> Self;
}

impl<T> Tappable<T> for Option<T> {
    fn tap_some(self, f: impl FnOnce(&T)) -> Self {
        if let Some(ref val) = self {
            f(val)
        };
        self
    }

    fn utap_some(self, f: impl FnOnce()) -> Self {
        if self.is_some() {
            f()
        };
        self
    }

    fn tap_nome(self, f: impl FnOnce()) -> Self {
        if self.is_none() {
            f()
        };
        self
    }
}

pub trait Discardable {
    fn unit(&self) -> () {
        ()
    }
}

impl<T> Discardable for T {}

pub fn unit<A>(_val: A) -> () {
    ()
}
