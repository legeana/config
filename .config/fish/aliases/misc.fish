function assh --wraps=ssh
    ssh -o UserKnownHostsFile=/dev/null -o StrictHostKeyChecking=no $argv
end

function df --wraps=df
    command df -h $argv
end

function dirf
    find . -type d | sed -e "s/[^-][^\/]*\//  |/g" -e "s/|\([^ ]\)/|-\1/" $argv
end

function du --wraps=du
    command du -sh $argv
end

function emacs --wraps=emacs
    command emacs -nw
end

function fbi --wraps=fbi
    command fbi -a $argv
end

function feh --wraps=feh
    command feh --scale-down $argv
end

function rplayer --wraps=mplayer
    mplayer -loop 0 -shuffle $argv
end

function psc
    ps xawf -eo pid,user,cgroup,args $argv
end

function most
    command most -w
end
