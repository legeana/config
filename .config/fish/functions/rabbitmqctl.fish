function rabbitmqctl
    sudo -u rabbitmq fish -c 'cd $HOME; and rabbitmqctl '"$argv"
end
