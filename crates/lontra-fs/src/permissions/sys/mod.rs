cfg_if::cfg_if! {
    if #[cfg(windows)] {
        mod windows;
        use windows as platform;
    } else if #[cfg(unix)] {
        mod unix;
        use unix as platform;
    }
}

pub(super) use platform::SysPermissions;
