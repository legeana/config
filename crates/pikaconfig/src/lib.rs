#![allow(
    // This lint is too opinionated.
    // In situations where enum name matches outside class
    // the consistency is more important than repetition.
    clippy::enum_variant_names,
)]

mod annotated_path;
mod command;
mod empty_struct;
mod file_util;
pub mod layout;
pub mod module;
pub mod package;
pub mod repository;
mod string_list;
mod symlink_util;
mod tag_criteria;
pub mod tag_util;
mod tera_helper;
mod tera_helpers;
pub mod uninstaller;
