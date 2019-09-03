if $UID != 0
    set modeline
endif
set background=dark
"cmap W w
"cmap Q q
syntax on
"set cindent
set number
set hidden
"set mouse=a
set notitle
set list
set listchars=tab:>.,trail:$,extends:#,nbsp:.
" установить keymap, чтобы по Ctrl+^ переключался на русский и обратно
"set keymap=russian-jcukenwin
" по умолчанию - латинская раскладка
"set iminsert=0
" по умолчанию - латинская раскладка при поиске
"set imsearch=0
set ignorecase
set smartcase
set hlsearch
set incsearch
"set textwidth=80
" минимальная высота окна пусть будет 0 (по умолчанию - 1)
set winminheight=0
" всегда делать активное окно максимального размера
set noequalalways
set winheight=9999
" установить шрифт Courier New Cyr
set guifont=courier_new:h10:cRUSSIAN
" настраиваю для работы с русскими словами (чтобы w, b, * понимали
" русские слова)
set iskeyword=@,48-57,_,192-255

" encryption
set cryptmethod=blowfish

" tabs
"set tabstop=4
" spaces
set expandtab
set shiftwidth=4
set softtabstop=4
set smarttab

" i3 arrows
noremap ; l
noremap l k
noremap k j
noremap j h

" Google style
autocmd Filetype cpp setlocal shiftwidth=2 softtabstop=2
autocmd Filetype proto setlocal shiftwidth=2 softtabstop=2
autocmd Filetype python setlocal shiftwidth=2 softtabstop=2
autocmd Filetype go setlocal noexpandtab shiftwidth=4 tabstop=4 softtabstop=4

set spell
hi clear SpellBad
hi SpellBad cterm=underline ctermfg=red

set fileencodings=utf-8,ucs-bom,cp1251,koi8-r,latin1

" Tell vim to remember certain things when we exit
"  '10  :  marks will be remembered for up to 10 previously edited files
"  "100 :  will save up to 100 lines for each register
"  :20  :  up to 20 lines of command-line history will be remembered
"  %    :  saves and restores the buffer list
"  n... :  where to save the viminfo files
set viminfo='10,\"100,:20,%,n~/.viminfo

function! ResCur()
  if line("'\"") <= line("$")
    normal! g`"
    return 1
  endif
endfunction

augroup resCur
  autocmd!
  autocmd BufWinEnter * call ResCur()
augroup END

function! MyRetab()
    set tabstop=4
    retab
    execute '%s/ \+$//gc'
endfunction

if &term =~ '^\(tmux\|screen\)-'
    " tmux will send xterm-style keys when its xterm-keys option is on
    execute "set <xUp>=\e[1;*A"
    execute "set <xDown>=\e[1;*B"
    execute "set <xRight>=\e[1;*C"
    execute "set <xLeft>=\e[1;*D"
endif