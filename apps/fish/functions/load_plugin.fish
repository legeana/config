function load_plugin
    for plugin in $argv
        if test -d $plugin/functions
            add_to_function_path $plugin/functions
        end
        if test -d $plugin/completions
            add_to_complete_path $plugin/completions
        end
        if test -d $plugin/conf.d
            source_d $plugin/conf.d
        end
        # TODO: themes
        # It is currently impossible to add themes from outside of
        # $__fish_config_dir via a path variable.
        # See <https://github.com/fish-shell/fish-shell/issues/9456>.
    end
end
