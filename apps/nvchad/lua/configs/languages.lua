local M = {}

-- ## LSP
-- https://github.com/neovim/nvim-lspconfig/blob/master/doc/configs.md
-- :help lspconfig-all
--
-- ## Treesitter
-- https://github.com/nvim-treesitter/nvim-treesitter/blob/master/README.md#supported-languages
local packages = {
  { lsp = "clangd", ts = {"c", "cpp"} },
  { lsp = "cssls", ts = {"css"} },
  { lsp = "gopls", ts = {"go"} },
  { lsp = "html", ts = {"html"} },
  -- { lsp = "java_language_server", ts = {"java"} },
  { lsp = "pylsp", ts = {"python"} },
  { lsp = "rust_analyzer", ts = {"rust"} },
  { lsp = "taplo", ts = {"toml"} },
  { lsp = "tsserver", ts = {"typescript"} },
}

M.lsp = vim.tbl_map(function(e) return e.lsp end, packages)
M.treesitter = vim.iter(vim.tbl_map(function(e) return e.ts end, packages)):flatten()

return M
