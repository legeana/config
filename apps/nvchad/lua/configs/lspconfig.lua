local configs = require "nvchad.configs.lspconfig"

local on_attach = configs.on_attach
local on_init = configs.on_init
local capabilities = configs.capabilities

local lspconfig = require "lspconfig"
local servers = {
  -- server_configurations.md -> Mason
  -- See https://github.com/neovim/nvim-lspconfig/blob/master/doc/server_configurations.md
  clangd="clangd",
  cssls="css-lsp",
  gopls="gopls",
  html="html-lsp",
  pylsp="python-lsp-server",
  rust_analyzer="rust-analyzer",
}
local LSP = vim.tbl_keys(servers)
local MASON = vim.tbl_values(servers)

for _, lsp in ipairs(LSP) do
  lspconfig[lsp].setup {
    on_init = on_init,
    on_attach = on_attach,
    capabilities = capabilities,
  }
end

return {
    LSP = LSP,
    MASON = MASON,
}
