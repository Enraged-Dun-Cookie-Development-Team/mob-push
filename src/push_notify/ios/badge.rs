pub enum IosBadgeType {
    Abs(u32),
    Adding(i32),
}

pub trait IosBadge {
    fn get_badge(&self) -> Option<IosBadgeType> {
        None
    }
}


