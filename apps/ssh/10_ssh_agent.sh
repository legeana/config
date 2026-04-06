systemctl --user import-environment DISPLAY
systemctl --user restart ssh-agent.service

export SSH_AUTH_SOCK="$XDG_RUNTIME_DIR/ssh-agent.socket"
