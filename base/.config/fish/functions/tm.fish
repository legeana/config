function tm
    set -l session default
    if set -q argv[1]
        set session $argv[1]
    end
    tmux attach -t $session ; or tmux new-session -s $session -c $HOME
end
