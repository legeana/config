function tm -a session
    if set -q TMUX
        echo Already in tmux session >&2
        verbose-eval tmux lsc
        return 1
    end
    if ! command -q tmux
        echo "tmux not found" >&2
        return 1
    end
    if test -z "$session"
        set session default
    end
    _tm_forward $session  # See conf.d/30_tmux.fish.
    _tm_attach $session
    or _tm_tmuxinator $session
    or _tm_tmuxp $session
    or _tm_tmux $session
end

function _tm_attach -a session
    if ! contains $session (tmux list-sessions | cut -f1 -d:)
        return 1
    end
    verbose-eval tmux attach -t $session
end

function _tm_tmuxinator -a session
    if ! command -q tmuxinator
        return 1
    end
    if ! contains $session (tmuxinator list -n | tail -n +2)
        return 1
    end
    verbose-eval tmuxinator start $session
end

function _tm_tmuxp -a session
    if ! command -q tmuxp
        return 1
    end
    if ! contains $session (tmuxp ls)
        return 1
    end
    verbose-eval tmuxp load $session
end

function _tm_tmux -a session
    verbose-eval tmux new-session -s $session -c $HOME
end
