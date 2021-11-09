# only use on systemd distros
if test -f ~/.config/systemd/user/default.target.wants/ssh-agent.service
    set -gx SSH_AUTH_SOCK "$XDG_RUNTIME_DIR/ssh-agent.socket"
end
