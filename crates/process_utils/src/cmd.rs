#[macro_export]
macro_rules! cmd {
    [@new $program:expr] => { $crate::Command::new($program) };

    [@push_down () -> ($($body:tt)*)] => { $($body)* };
    [@push_down ([] $(,)*) -> ($($body:tt)*)] => { $($body)* };
    [@push_down ([$($arg:expr),* $(,)*] $(,)*) -> ($($body:tt)*)] => {
        $($body)* $(.arg($arg))*
    };
    [@push_down ([], $($tail:tt)*) -> $body:tt] => {
        cmd![
            @push_down
            ($($tail)*)
            ->
            $body
        ]
    };
    [@push_down ([$($arg:expr),* $(,)*], $($tail:tt)*) -> ($($body:tt)*)] => {
        cmd![
            @push_down
            ($($tail)*)
            ->
            ($($body)* $(.arg($arg))*)
        ]
    };
    [@push_down ($arg:expr) -> ($($body:tt)*)] => {
        $($body)*.args($arg)
    };
    [@push_down ($arg:expr, $($tail:tt)*) -> ($($body:tt)*)] => {
        cmd![
            @push_down
            ($($tail)*)
            ->
            ($($body)*.args($arg))
        ]
    };

    // cmd!(["program"])
    ([$program:expr]) => { cmd![@new $program] };
    // cmd!(["program", ...])
    ([$program:expr, $($tail:tt)*]) => {
        cmd![
            @push_down
            ([$($tail)*])
            ->
            (cmd![@new $program])
        ]
    };
    // cmd!(["program"], ...)
    ([$program:expr], $($tail:tt)*) => {
        cmd![
            @push_down
            ($($tail)*)
            ->
            (cmd![@new $program])
        ]
    };
    // cmd!(["program", ...], ...)
    ([$program:expr, $($arg:expr),* $(,)*], $($tail:tt)*) => {
        cmd![
            @push_down
            ($($tail)*)
            ->
            (cmd![@new $program] $(.arg($arg))*)
        ]
    };
    // cmd!("program")
    ($program:expr) => { cmd![@new $program] };
    // cmd!("program", ...)
    ($program:expr, $($tail:tt)*) => {
        cmd![
            @push_down
            ($($tail)*)
            ->
            (cmd![@new $program])
        ]
    };
}

#[cfg(test)]
mod tests {
    use std::ffi::OsStr;
    use std::path::Path;

    use pretty_assertions::assert_eq;

    #[test]
    fn test_cmd_empty() {
        let cmd = cmd!("program");

        let std_cmd = cmd.finalise();
        assert_eq!(std_cmd.get_program(), "program");
        assert_eq!(std_cmd.get_args().collect::<Vec<_>>(), &[] as &[&str]);
    }

    #[test]
    fn test_cmd_single_arg() {
        let cmd = cmd!("program", ["arg-1"]);

        let std_cmd = cmd.finalise();
        assert_eq!(std_cmd.get_program(), "program");
        assert_eq!(std_cmd.get_args().collect::<Vec<_>>(), &["arg-1"]);
    }

    #[test]
    fn test_cmd_single_arg_trailing_comma() {
        let cmd = cmd!("program", ["arg-1"],);

        let std_cmd = cmd.finalise();
        assert_eq!(std_cmd.get_program(), "program");
        assert_eq!(std_cmd.get_args().collect::<Vec<_>>(), &["arg-1"]);
    }

    #[test]
    fn test_cmd_multiple_args() {
        let cmd = cmd!("program", ["arg-1", "arg-2", "arg-3"]);

        let std_cmd = cmd.finalise();
        assert_eq!(std_cmd.get_program(), "program");
        assert_eq!(
            std_cmd.get_args().collect::<Vec<_>>(),
            &["arg-1", "arg-2", "arg-3"],
        );
    }

    #[test]
    fn test_cmd_different_arg_types() {
        let arg1 = "arg-1";
        let arg2 = Path::new("arg-2");
        let arg3 = OsStr::new("arg-3");

        let cmd = cmd!("program", [arg1, arg2, arg3]);

        let std_cmd = cmd.finalise();
        assert_eq!(std_cmd.get_program(), "program");
        assert_eq!(
            std_cmd.get_args().collect::<Vec<_>>(),
            &["arg-1", "arg-2", "arg-3"],
        );
    }

    #[test]
    fn test_cmd_multiple_args_trailing_comma() {
        let cmd = cmd!("program", ["arg-1", "arg-2", "arg-3"],);

        let std_cmd = cmd.finalise();
        assert_eq!(std_cmd.get_program(), "program");
        assert_eq!(
            std_cmd.get_args().collect::<Vec<_>>(),
            &["arg-1", "arg-2", "arg-3"],
        );
    }

    #[test]
    fn test_cmd_args() {
        let cmd = cmd!("program", vec!["arg-1", "arg-2"]);

        let std_cmd = cmd.finalise();
        assert_eq!(std_cmd.get_program(), "program");
        assert_eq!(std_cmd.get_args().collect::<Vec<_>>(), &["arg-1", "arg-2"]);
    }

    #[test]
    fn test_cmd_args_trailing_comma() {
        let cmd = cmd!("program", vec!["arg-1", "arg-2"],);

        let std_cmd = cmd.finalise();
        assert_eq!(std_cmd.get_program(), "program");
        assert_eq!(std_cmd.get_args().collect::<Vec<_>>(), &["arg-1", "arg-2"]);
    }

    #[test]
    fn test_cmd_args_before_arg() {
        let cmd = cmd!("program", vec!["arg-1", "arg-2"], ["arg-3"]);

        let std_cmd = cmd.finalise();
        assert_eq!(std_cmd.get_program(), "program");
        assert_eq!(
            std_cmd.get_args().collect::<Vec<_>>(),
            &["arg-1", "arg-2", "arg-3"],
        );
    }

    #[test]
    fn test_cmd_args_after_arg() {
        let cmd = cmd!("program", ["arg-1"], vec!["arg-2", "arg-3"]);

        let std_cmd = cmd.finalise();
        assert_eq!(std_cmd.get_program(), "program");
        assert_eq!(
            std_cmd.get_args().collect::<Vec<_>>(),
            &["arg-1", "arg-2", "arg-3"],
        );
    }

    #[test]
    fn test_cmd_wrapped_program() {
        let cmd = cmd!(["program"]);

        let std_cmd = cmd.finalise();
        assert_eq!(std_cmd.get_program(), "program");
        assert_eq!(std_cmd.get_args().collect::<Vec<_>>(), &[] as &[&str]);
    }

    #[test]
    fn test_cmd_wrapped_program_and_args() {
        let cmd = cmd!(["program", "arg-1", "arg-2", "arg-3"]);

        let std_cmd = cmd.finalise();
        assert_eq!(std_cmd.get_program(), "program");
        assert_eq!(
            std_cmd.get_args().collect::<Vec<_>>(),
            &["arg-1", "arg-2", "arg-3"],
        );
    }

    #[test]
    fn test_cmd_wrapped_program_and_args_trailing_wrapped_comma() {
        let cmd = cmd!(["program", "arg-1", "arg-2", "arg-3",]);

        let std_cmd = cmd.finalise();
        assert_eq!(std_cmd.get_program(), "program");
        assert_eq!(
            std_cmd.get_args().collect::<Vec<_>>(),
            &["arg-1", "arg-2", "arg-3"],
        );
    }

    #[test]
    fn test_cmd_wrapped_program_and_args_trailing_unwrapped_comma() {
        let cmd = cmd!(["program", "arg-1", "arg-2", "arg-3"],);

        let std_cmd = cmd.finalise();
        assert_eq!(std_cmd.get_program(), "program");
        assert_eq!(
            std_cmd.get_args().collect::<Vec<_>>(),
            &["arg-1", "arg-2", "arg-3"],
        );
    }

    #[test]
    fn test_cmd_wrapped_program_and_args_trailing_commas() {
        let cmd = cmd!(["program", "arg-1", "arg-2", "arg-3",],);

        let std_cmd = cmd.finalise();
        assert_eq!(std_cmd.get_program(), "program");
        assert_eq!(
            std_cmd.get_args().collect::<Vec<_>>(),
            &["arg-1", "arg-2", "arg-3"],
        );
    }

    #[test]
    fn test_cmd_wrapped_program_trailing_args() {
        let cmd = cmd!(["program"], ["arg-1", "arg-2", "arg-3"]);

        let std_cmd = cmd.finalise();
        assert_eq!(std_cmd.get_program(), "program");
        assert_eq!(
            std_cmd.get_args().collect::<Vec<_>>(),
            &["arg-1", "arg-2", "arg-3"],
        );
    }

    #[test]
    fn test_cmd_wrapped_program_and_args_trailing_args() {
        let cmd = cmd!(["program", "arg-1", "arg-2"], ["arg-3"]);

        let std_cmd = cmd.finalise();
        assert_eq!(std_cmd.get_program(), "program");
        assert_eq!(
            std_cmd.get_args().collect::<Vec<_>>(),
            &["arg-1", "arg-2", "arg-3"],
        );
    }

    #[test]
    fn test_cmd_wrapped_program_and_args_trailing_comma_trailing_args() {
        let cmd = cmd!(["program", "arg-1", "arg-2",], ["arg-3"]);

        let std_cmd = cmd.finalise();
        assert_eq!(std_cmd.get_program(), "program");
        assert_eq!(
            std_cmd.get_args().collect::<Vec<_>>(),
            &["arg-1", "arg-2", "arg-3"],
        );
    }
}
