function make_qmake
    qmake -project '\''CONFIG -= qt'\'' && qmake -Wall $argv
end
