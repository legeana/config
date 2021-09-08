set -e MANPATH
manpath | sed 's|:| |g;s|^|set -gx MANPATH |' | source
