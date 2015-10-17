function assh
    ssh -o UserKnownHostsFile=/dev/null -o StrictHostKeyChecking=no $argv
end

function df
    command df -h $argv
end

function dirf
    find . -type d | sed -e "s/[^-][^\/]*\//  |/g" -e "s/|\([^ ]\)/|-\1/" $argv
end

function du
    command du -sh $argv
end

function emacs
    command emacs -nw
end

function fbi
    fbi -a $argv
end

function feh
    command feh --scale-down $argv
end

function rplayer
    mplayer -loop 0 -shuffle $argv
end

function psc
    ps xawf -eo pid,user,cgroup,args $argv
end

function rabbitmqctl
    sudo -u rabbitmq rabbitmqctl $argv
end
