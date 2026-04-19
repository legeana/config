# only use if the socket is available and not in an ssh session
if test -z "$SSH_TTY";
        and test -z "$SSH_CONNECTION";
        and test -z "$SSH_CLIENT";
        and test -z "$TMUX"
    if test -S "$HOME/.1password/agent.sock"
        set -gx SSH_AUTH_SOCK "$HOME/.1password/agent.sock"
    else if test -S "$XDG_RUNTIME_DIR/ssh-agent.socket"
        set -gx SSH_AUTH_SOCK "$XDG_RUNTIME_DIR/ssh-agent.socket"
    end
end
