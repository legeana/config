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
set spellfile=$HOME/.config/nvim-local/spell/en.utf-8.add,$HOME/.config/nvim/spell/en.utf-8.add
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
