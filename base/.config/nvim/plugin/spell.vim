set spell
set spelllang=en
set spellfile=$HOME/.config/nvim/spell/en.utf-8.add
hi clear SpellBad
hi SpellBad cterm=underline ctermfg=red

function! s:regen()
    let spellfiles = split(&spellfile, ',')
    for spellfile in spellfiles
        execute 'mkspell! ' . spellfile
    endfor
endfunction

command! RegenSpellFiles call s:regen()
