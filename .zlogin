export ZSH_LOADED="$ZSH_LOADED:USER_LOGIN"

export ZLOGIN_LOADED=1

return 0
# TODO
clear
stty dec new cr0 -tabs
ttyctl -f  # freeze the terminal modes... can't change without a ttyctl -u
mesg y
uptime
fortune
log
from 2>/dev/null
cat notes
msgs -fp
