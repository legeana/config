if [ -n "$DISPLAY" ]
    set -g MPLAYER_PROFILE x
else if [ -n "$TMUX" -o -n "$SSH_CLIENT" -o "$TERM" = screen ]
    set -g MPLAYER_PROFILE audio
else
    set -g MPLAYER_PROFILE console
end
function mplayer
    command mplayer -profile "$MPLAYER_PROFILE" $argv
end
