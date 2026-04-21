return {
  {
    "nvim-mini/mini.pairs",
    opts = {
      mappings = {
        -- These pairs frequently misbehave.
        ['"'] = false,
        ["'"] = false,
        ["`"] = false,
      },
    },
  },
}
