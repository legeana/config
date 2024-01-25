# See functions/tm.fish.
set -g _TM_TMP "$HOME/.local/state/tmux-ssh-agent"
set -g _TM_PLACEHOLDER /dev/null

function _tm_env_for_session -a session -a env
    echo "$_TM_TMP/$USER.$session.$env"
end

function _set_up_tmux_environment
    set -l session (tmux display-message -p '#S')
    set -gx SSH_AUTH_SOCK (_tm_env_for_session "$session" "SSH_AUTH_SOCK")
    set -e FWD_SSH_AUTH_SOCK
end

if set -q TMUX
    _set_up_tmux_environment
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
function _tm_forward_socket -a session -a env -a value
    mkdir -p -m 700 "$_TM_TMP"
    set path (_tm_env_for_session "$session" "$env")
    set dst "$value"
    if test -z "$dst"
        set dst $_TM_PLACEHOLDER
    end
    ln -snf -- $dst $path
end

function _tm_forward -a session
    _tm_cleanup
    # Restore FWD_SSH_AUTH_SOCK and forward it as SSH_AUTH_SOCK if possible.
    # FWD_SSH_AUTH_SOCK if set points to the original agent socket.
    if ! set -q FWD_SSH_AUTH_SOCK || ! test -S $FWD_SSH_AUTH_SOCK
        _tm_forward_socket $session SSH_AUTH_SOCK "$SSH_AUTH_SOCK"
    else
        _tm_forward_socket $session SSH_AUTH_SOCK "$FWD_SSH_AUTH_SOCK"
    end
end
