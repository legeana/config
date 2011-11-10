if $UID!=0
	set modeline
endif
set background=dark
cmap W w
cmap Q q
syntax on
"set cindent
set number
" установить keymap, чтобы по Ctrl+^ переключался на русский и обратно
"set keymap=russian-jcukenwin 
" по умолчанию - латинская раскладка
"set iminsert=0
" по умолчанию - латинская раскладка при поиске
"set imsearch=0
" игнорировать регистр при поиске
set ic
" подсвечивать поиск
set hls
" использовать инкрементальный поиск
set is
" ширина текста 
"set textwidth=70
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
" задать размер табуляции в четыре пробела
"set ts=4

" tabs
"set tabstop=4
"set shiftwidth=4
"set expandtab
"set smarttab

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

