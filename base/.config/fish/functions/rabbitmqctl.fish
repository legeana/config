function rabbitmqctl
    sudo -u rabbitmq sh -c 'cd $HOME && rabbitmqctl "$@"' - $argv
end
