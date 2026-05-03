function add_plugin_d -d 'Adds all plugins from given directories to fish'
    for plugin_dir in $argv
        for plugin in $plugin_dir/*
            add_plugin $plugin
        end
    end
end
