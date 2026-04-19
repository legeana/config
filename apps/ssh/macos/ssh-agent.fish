if test -z "$SSH_TTY"
        and test -z "$SSH_CONNECTION"
        and test -z "$SSH_CLIENT"
        and test -z "$TMUX"
    for sock in \
            "$HOME/Library/Group Containers/2BUA8C4S2C.com.1password/t/agent.sock"
        if test -S "$sock"
            set -gx SSH_AUTH_SOCK "$sock"
            break
        end
    end
end
