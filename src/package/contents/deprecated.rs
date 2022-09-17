use crate::package::contents::parser;
use crate::package::contents::util::check_command;
use crate::package::contents::Configuration;

pub struct DeprecatedParser;

const COMMAND: &str = "deprecated commands, do not use";

impl parser::Parser for DeprecatedParser {
    fn name(&self) -> &'static str {
        COMMAND
    }
    fn help(&self) -> &'static str {
        "DEPRECATED:
           - install_system_package
           - install_pacman_package
           - install_apt_package
           - install_brew_package
           - install_pip_user_package
           - sudo"
    }
    fn parse(
        &self,
        _state: &mut parser::State,
        configuration: &mut Configuration,
        args: &[&str],
    ) -> parser::Result<()> {
        if check_command("install_system_package", args).is_ok() {
            log::warn!(
                "{:?}: install_system_package is unsupported",
                configuration.root
            );
            return Ok(());
        }
        if check_command("install_pacman_package", args).is_ok() {
            log::warn!(
                "{:?}: install_pacman_package is unsupported",
                configuration.root
            );
            return Ok(());
        }
        if check_command("install_apt_package", args).is_ok() {
            log::warn!(
                "{:?}: install_apt_package is unsupported",
                configuration.root
            );
            return Ok(());
        }
        if check_command("install_brew_package", args).is_ok() {
            log::warn!(
                "{:?}: install_brew_package is unsupported",
                configuration.root
            );
            return Ok(());
        }
        if check_command("install_pip_user_package", args).is_ok() {
            log::warn!(
                "{:?}: install_pip_user_package is unsupported",
                configuration.root
            );
            return Ok(());
        }
        if check_command("sudo", args).is_ok() {
            log::warn!("{:?}: sudo is unsupported", configuration.root);
            return Ok(());
        }
        return check_command(COMMAND, args).map(|_| ());
    }
}
