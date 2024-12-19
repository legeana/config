local M = {}

-- ## LSP
-- https://github.com/neovim/nvim-lspconfig/blob/master/doc/configs.md
-- :help lspconfig-all
--
-- ## Treesitter
-- https://github.com/nvim-treesitter/nvim-treesitter/blob/master/README.md#supported-languages
local packages = {
  -- Nvchad comes with a preconfigured lua_ls and treesitter configs.
  { lsp = "bashls", ts = {"bash"} },
  { lsp = "clangd", ts = {"c", "cpp"} },
  { lsp = "cssls", ts = {"css"} },
  { lsp = "fish_lsp", ts = {"fish"} },
  { lsp = "gopls", ts = {"go"} },
  { lsp = "html", ts = {"html"} },
  -- { lsp = "java_language_server", ts = {"java"} },
  { lsp = "pylsp", ts = {"python"} },
  { lsp = "rust_analyzer", ts = {"rust"} },
  { lsp = "taplo", ts = {"toml"} },
  { lsp = "ts_ls", ts = {"typescript"} },
  { lsp = "vimls", ts = {"vim"} },
  { lsp = "yamlls", ts = {"yaml"} },
}

M.lsp = vim.tbl_map(function(e) return e.lsp end, packages)
M.treesitter = vim.iter(vim.tbl_map(function(e) return e.ts end, packages)):flatten()

return M
