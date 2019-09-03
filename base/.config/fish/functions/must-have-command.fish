function must-have-command --description 'Does nothing if command exists and exits otherwise.'
    command -s $argv[1] >/dev/null
    or exit
end
