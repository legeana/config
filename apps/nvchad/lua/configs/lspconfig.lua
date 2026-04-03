-- load defaults i.e lua_lsp
require("nvchad.configs.lspconfig").defaults()
local languages = require "configs.languages"

vim.lsp.enable(languages.lsp)

-- read :h vim.lsp.config for changing options of lsp servers
