function gpg-agent --description 'Run gpg-agent'
    set -xg GPG_TTY (tty)
    gpg-connect-agent /bye
    set -xg SSH_AUTH_SOCK (gpgconf --list-dirs agent-ssh-socket)
    gpg-connect-agent updatestartuptty /bye
    ssh-add-if-necessary
end
