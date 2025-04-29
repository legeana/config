#[cfg(windows)]
mod windows;

#[cfg(unix)]
mod unix;

#[cfg(windows)]
pub(super) use windows::WindowsConverter as SysConverter;

#[cfg(unix)]
pub(super) use unix::UnixConverter as SysConverter;
