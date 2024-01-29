# See functions/tm.fish.
set -g _TM_TMP "$HOME/.local/state/tmux-ssh-agent"
set -g _TM_PLACEHOLDER /dev/null

function _tm_env_for_session -a session -a tty -a env
    set -l tty (string replace -a / _ "$tty")
    echo "$_TM_TMP/$USER.$session.$tty.$env"
end

function _set_up_tmux_environment --on-event fish_preexec
    if ! set -q TMUX
        return
    end
    set -l session (tmux display-message -p '#{session_name}')
    set -l tty (tmux display-message -p '#{client_tty}')
    set -gx SSH_AUTH_SOCK (_tm_env_for_session "$session" "$tty" "SSH_AUTH_SOCK")
    set -e FWD_SSH_AUTH_SOCK
end

function _tm_cleanup
    for f in "$_TM_TMP/"*
        if ! test -L $f
            continue
        end
        if ! test -e $f
            rm -f $f
        end
        if test "$(readlink -- $f)" = $_TM_PLACEHOLDER
            rm -f $f
        end
    end
end

# The functions below operate under the assumption that tmux will not be used
# by more than one user simultaneously. The last connection wins.
function _tm_forward_socket -a session -a tty -a env -a value
    mkdir -p -m 700 "$_TM_TMP"
    set path (_tm_env_for_session "$session" "$tty" "$env")
    set dst "$value"
    if test -z "$dst"
        set dst $_TM_PLACEHOLDER
    end
    ln -snf -- $dst $path
end

function _tm_forward -a session
    _tm_cleanup
    set -l tty (tty)
    # Restore FWD_SSH_AUTH_SOCK and forward it as SSH_AUTH_SOCK if possible.
    # FWD_SSH_AUTH_SOCK if set points to the original agent socket.
    if ! set -q FWD_SSH_AUTH_SOCK || ! test -S $FWD_SSH_AUTH_SOCK
        _tm_forward_socket $session "$tty" SSH_AUTH_SOCK "$SSH_AUTH_SOCK"
    else
        _tm_forward_socket $session "$tty" SSH_AUTH_SOCK "$FWD_SSH_AUTH_SOCK"
    end
end
