use std::convert::{From, TryFrom};

static INVALID_CONVERTSION_MESSAGE: &'static str = "Cannot convert stack entry";

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum StackEntry {
    Unused,
    I32Entry(u32),
    I64Entry(u64),
    F32Entry(f32),
    F64Entry(f64),
}

impl From<u32> for StackEntry {
    fn from(i: u32) -> StackEntry {
        StackEntry::I32Entry(i)
    }
}

impl TryFrom<StackEntry> for u32 {
    type Error = &'static str;

    fn try_from(i: StackEntry) -> Result<Self, Self::Error> {
        match i {
            StackEntry::I32Entry(u) => Ok(u),
            // Should this handle the case where it is an I64Entry and the value fits? That would simplify
            // some things, but may complicate other things by not being strict enough
            _ => Err(INVALID_CONVERTSION_MESSAGE),
        }
    }
}

impl From<i32> for StackEntry {
    fn from(i: i32) -> StackEntry {
        Self::from(unsafe { std::mem::transmute::<i32, u32>(i) })
    }
}

impl TryFrom<StackEntry> for i32 {
    type Error = &'static str;

    fn try_from(i: StackEntry) -> Result<Self, Self::Error> {
        u32::try_from(i).map(|i| unsafe { std::mem::transmute::<u32, i32>(i) })
    }
}

impl From<u64> for StackEntry {
    fn from(i: u64) -> StackEntry {
        StackEntry::I64Entry(i)
    }
}

impl TryFrom<StackEntry> for u64 {
    type Error = &'static str;

    fn try_from(i: StackEntry) -> Result<Self, Self::Error> {
        match i {
            StackEntry::I64Entry(u) => Ok(u),
            // Should this handle the case where it is an I32Entry? That would simplify
            // some things, but may complicate other things by not being strict enough
            _ => Err(INVALID_CONVERTSION_MESSAGE),
        }
    }
}

impl From<i64> for StackEntry {
    fn from(i: i64) -> Self {
        Self::from(unsafe { std::mem::transmute::<i64, u64>(i) })
    }
}

impl TryFrom<StackEntry> for i64 {
    type Error = &'static str;

    fn try_from(i: StackEntry) -> Result<Self, Self::Error> {
        u64::try_from(i).map(|i| unsafe { std::mem::transmute::<u64, i64>(i) })
    }
}

impl From<f32> for StackEntry {
    fn from(i: f32) -> Self {
        Self::F32Entry(i)
    }
}

impl TryFrom<StackEntry> for f32 {
    type Error = &'static str;

    fn try_from(i: StackEntry) -> Result<Self, Self::Error> {
        match i {
            StackEntry::F32Entry(f) => Ok(f),
            _ => Err(INVALID_CONVERTSION_MESSAGE),
        }
    }
}

impl From<f64> for StackEntry {
    fn from(i: f64) -> Self {
        Self::F64Entry(i)
    }
}

impl TryFrom<StackEntry> for f64 {
    type Error = &'static str;

    fn try_from(i: StackEntry) -> Result<Self, Self::Error> {
        match i {
            StackEntry::F64Entry(f) => Ok(f),
            _ => Err(INVALID_CONVERTSION_MESSAGE),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_stack_entry() {
        assert_eq!(StackEntry::from(32u32), StackEntry::I32Entry(32));
        assert_eq!(StackEntry::from(32i32), StackEntry::I32Entry(32));
        assert_eq!(
            StackEntry::from(0xFFFFFFFFu32),
            StackEntry::I32Entry(0xFFFFFFFF)
        );
        assert_eq!(StackEntry::from(-1i32), StackEntry::I32Entry(0xFFFFFFFF));
        assert_eq!(StackEntry::from(-32i32), StackEntry::I32Entry(0xFFFFFFE0));

        assert_eq!(StackEntry::from(32u64), StackEntry::I64Entry(32));
        assert_eq!(StackEntry::from(32i64), StackEntry::I64Entry(32));
        assert_eq!(
            StackEntry::from(0xFFFFFFFFFFFFFFFFu64),
            StackEntry::I64Entry(0xFFFFFFFFFFFFFFFF)
        );
        assert_eq!(
            StackEntry::from(-1i64),
            StackEntry::I64Entry(0xFFFFFFFFFFFFFFFF)
        );
        assert_eq!(
            StackEntry::from(-32i64),
            StackEntry::I64Entry(0xFFFFFFFFFFFFFFE0)
        );

        assert_eq!(StackEntry::from(32.0f32), StackEntry::F32Entry(32.0));

        assert_eq!(StackEntry::from(32.0f64), StackEntry::F64Entry(32.0));

        assert_eq!(
            u32::try_from(StackEntry::Unused),
            Err(INVALID_CONVERTSION_MESSAGE)
        );
        assert_eq!(u32::try_from(StackEntry::I32Entry(32)), Ok(32));
        assert_eq!(
            u32::try_from(StackEntry::I32Entry(0xFFFFFFFF)),
            Ok(0xFFFFFFFF)
        );
        assert_eq!(i32::try_from(StackEntry::I32Entry(0xFFFFFFFF)), Ok(-1));
        assert_eq!(
            u32::try_from(StackEntry::I32Entry(0xFFFFFFE0)),
            Ok(0xFFFFFFE0)
        );
        assert_eq!(i32::try_from(StackEntry::I32Entry(0xFFFFFFE0)), Ok(-32));
        assert_eq!(
            u32::try_from(StackEntry::I64Entry(32)),
            Err(INVALID_CONVERTSION_MESSAGE)
        );
        assert_eq!(
            u32::try_from(StackEntry::I64Entry(32)),
            Err(INVALID_CONVERTSION_MESSAGE)
        );
        assert_eq!(
            u32::try_from(StackEntry::F32Entry(32.0)),
            Err(INVALID_CONVERTSION_MESSAGE)
        );
        assert_eq!(
            u32::try_from(StackEntry::F64Entry(32.0)),
            Err(INVALID_CONVERTSION_MESSAGE)
        );

        assert_eq!(
            u64::try_from(StackEntry::Unused),
            Err(INVALID_CONVERTSION_MESSAGE)
        );
        assert_eq!(
            u64::try_from(StackEntry::I32Entry(32)),
            Err(INVALID_CONVERTSION_MESSAGE)
        );
        assert_eq!(u64::try_from(StackEntry::I64Entry(32)), Ok(32));
        assert_eq!(
            u64::try_from(StackEntry::I64Entry(0xFFFFFFFFFFFFFFFF)),
            Ok(0xFFFFFFFFFFFFFFFF)
        );
        assert_eq!(
            i64::try_from(StackEntry::I64Entry(0xFFFFFFFFFFFFFFFF)),
            Ok(-1)
        );
        assert_eq!(
            u64::try_from(StackEntry::I64Entry(0xFFFFFFFFFFFFFFE0)),
            Ok(0xFFFFFFFFFFFFFFE0)
        );
        assert_eq!(
            i64::try_from(StackEntry::I64Entry(0xFFFFFFFFFFFFFFE0)),
            Ok(-32)
        );
        assert_eq!(
            u64::try_from(StackEntry::F32Entry(32.0)),
            Err(INVALID_CONVERTSION_MESSAGE)
        );
        assert_eq!(
            u64::try_from(StackEntry::F64Entry(32.0)),
            Err(INVALID_CONVERTSION_MESSAGE)
        );

        assert_eq!(
            f32::try_from(StackEntry::Unused),
            Err(INVALID_CONVERTSION_MESSAGE)
        );
        assert_eq!(
            f32::try_from(StackEntry::I32Entry(32)),
            Err(INVALID_CONVERTSION_MESSAGE)
        );
        assert_eq!(
            f32::try_from(StackEntry::I64Entry(32)),
            Err(INVALID_CONVERTSION_MESSAGE)
        );
        assert_eq!(f32::try_from(StackEntry::F32Entry(32.0)), Ok(32.0));
        assert_eq!(
            f32::try_from(StackEntry::F64Entry(32.0)),
            Err(INVALID_CONVERTSION_MESSAGE)
        );

        assert_eq!(
            f64::try_from(StackEntry::Unused),
            Err(INVALID_CONVERTSION_MESSAGE)
        );
        assert_eq!(
            f64::try_from(StackEntry::I32Entry(32)),
            Err(INVALID_CONVERTSION_MESSAGE)
        );
        assert_eq!(
            f64::try_from(StackEntry::I64Entry(32)),
            Err(INVALID_CONVERTSION_MESSAGE)
        );
        assert_eq!(
            f64::try_from(StackEntry::F32Entry(32.0)),
            Err(INVALID_CONVERTSION_MESSAGE)
        );
        assert_eq!(f64::try_from(StackEntry::F64Entry(32.0)), Ok(32.0));
    }
}
