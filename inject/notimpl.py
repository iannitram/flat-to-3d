"""The one error a stub raises, so tests and callers can tell 'not built yet' apart."""


class NotBuilt(NotImplementedError):
    """Raised by a skeleton stub; the message names the issue that fills it in."""
