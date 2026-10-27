"""Inject a game by its recipe's stereo route. 1.0 has one route: the Spider-Man 2 port."""
from __future__ import annotations
from .notimpl import NotBuilt
from .recipes import Recipe
from .steam import Game


def inject(game: Game, recipe: Recipe) -> str:
    """Set the game up to launch with its stereo route; returns a one-line summary.
    Refuses a recipe whose safe is not "yes" or whose status is not works-now."""
    raise NotBuilt("injectors.inject: see the inject issue")
