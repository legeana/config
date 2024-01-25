function tm -a session
    if set -q TMUX
        echo Already in tmux session >&2
        verbose-eval tmux lsc
        return 1
    end
    if test -z "$session"
        set session default
    end
    _tm_forward $session  # See conf.d/30_tmux.fish.
    tmux attach -t $session
    or tmuxinator start $session
    or tmuxp load $session
    or tmux new-session -s $session -c $HOME
end
