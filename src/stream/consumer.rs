

/// slice/d Consumer Groups have some standardized fields to control
/// it's overall behavior. A naming convention is utilized over the
/// standard Redis Streams listpack format.
pub struct CGRecord {
    deadline: Option<u64>,
    delay: Option<i64>,
}

impl CGRecord {
}