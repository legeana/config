set spell
set spelllang=en
set spelloptions=camel
set spellfile^=$HOME/.config/vim-spell/draft/en.utf-8.add,$HOME/.config/vim-spell/committed/en.utf-8.add
hi clear SpellBad
" gui* options are used if termguicolors is set.
hi SpellBad cterm=underline ctermfg=red gui=underline guifg=red
set spellcapcheck=

function! s:regen()
    let spellfiles = split(&spellfile, ',')
    for spellfile in spellfiles
        execute 'mkspell! ' . spellfile
    endfor
endfunction

command! RegenSpellFiles call s:regen()
