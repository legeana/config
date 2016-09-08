set -e MANPATH

env -i HOME=$HOME sh -l -c 'source /etc/profile ; printenv' | \
    sed -e '/^PWD=/d;
            /^SHLVL=/d;
            /^_/d;
            /PATH/s/:/ /g;
            s/"/\\"/g;
            s/\\\\/\\\\/g;
            s/\(^[^=]*\)=\(.*\)$/\1 "\2"/;
            s/^/set -gx /' | \
    source

manpath | sed 's|:| |g;s|^|set -gx MANPATH |' | source
