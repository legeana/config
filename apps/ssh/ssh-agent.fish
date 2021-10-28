# only use on systemd distros
if command --search --quiet systemctl
    set -gx SSH_AUTH_SOCK "$XDG_RUNTIME_DIR/ssh-agent.socket"
end
