function tmc
    set -l session default
    if set -q argv[1]
        set session $argv[1]
    end
    tmux new -s $session -c $HOME
end
