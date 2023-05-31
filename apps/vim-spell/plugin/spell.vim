if exists('g:enable_spelunker_vim') && g:enable_spelunker_vim
    set nospell
    let g:spelunker_spell_bad_group='SpellBad'
    let g:spelunker_complex_or_compound_word_group='SpellBad'
    map zg Zg
    map zw Zw
else
    set spell
end
set spelllang=en
set spelloptions=camel
set spellfile^=$HOME/.config/vim-spell/draft/en.utf-8.add,$HOME/.config/vim-spell/committed/en.utf-8.add
hi clear SpellBad
hi SpellBad cterm=underline ctermfg=red
set spellcapcheck=

function! s:regen()
    let spellfiles = split(&spellfile, ',')
    for spellfile in spellfiles
        execute 'mkspell! ' . spellfile
    endfor
endfunction

command! RegenSpellFiles call s:regen()

" https://github.com/kamykn/spelunker.vim/issues/71#issuecomment-1023835797
autocmd BufRead * if getfsize(@%) > 100000 | let g:spelunker_check_type = 2 | endif
