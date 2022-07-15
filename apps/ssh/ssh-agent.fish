# only use if the socket is available and not in an ssh session
if test -S "$XDG_RUNTIME_DIR/ssh-agent.socket";
        and test -z "$SSH_TTY";
        and test -z "$SSH_CONNECTION";
        and test -z "$SSH_CLIENT";
        and test -z "$TMUX"
    set -gx SSH_AUTH_SOCK "$XDG_RUNTIME_DIR/ssh-agent.socket"
end
