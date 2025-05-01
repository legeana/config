use crate::tag_util::UidGetter;

pub(crate) struct SysUidGetter;

impl UidGetter for SysUidGetter {
    fn getuid() -> Option<u32> {
        None
    }
}
