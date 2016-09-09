set -e MANPATH

env -i HOME=$HOME sh -l -c 'source /etc/profile ; printenv' | \
    sed -e '/^PWD=/d;
            /^SHLVL=/d;
            /^_/d;
            s/"/\\"/g;
            s/\\\\/\\\\/g;
            s/\(^[^=]*\)=\(.*\)$/\1 "\2"/;
            /PATH/s/:/" "/g;
            s/^/set -gx /' | \
    source

manpath | sed 's|:| |g;s|^|set -gx MANPATH |' | source
