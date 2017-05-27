function load_plugins
    for plugin in $argv
        if test -d $plugin/functions
            add_to_function_path $plugin/functions
        end
        if test -d $plugin/completions
            add_to_complete_path $plugin/completions
        end
        if test -d $plugin/config.d
            load_d $plugin/config.d
        end
    end
end
