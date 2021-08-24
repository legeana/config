import logging


def init():
  logging.basicConfig(
      format='%(message)s',
      level=logging.INFO,
  )
