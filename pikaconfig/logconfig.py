import logging


def init(verbose: bool):
  logging.basicConfig(
      format='%(message)s',
      level=logging.INFO if verbose else logging.WARNING,
  )
