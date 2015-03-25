export ZSH_LOADED="$ZSH_LOADED:USER_PROFILE"

if [[ -f ~/.zlocalprofile ]]
then
    source ~/.zlocalprofile
fi

export ZPROFILE_LOADED=1
