import dataclasses
import unittest

from inject import cli, injectors, link, recipes, steam
from inject.notimpl import NotBuilt


class Skeleton(unittest.TestCase):
    def test_recipe_has_the_eight_keys(self):
        names = {f.name for f in dataclasses.fields(recipes.Recipe)}
        self.assertTrue({"status", "route", "engine", "api", "mod", "free", "safe", "checked"} <= names)

    def test_cli_parses_the_three_commands(self):
        p = cli.build_parser()
        self.assertEqual(p.parse_args(["list"]).command, "list")
        self.assertEqual(p.parse_args(["inject", "2651280"]).appid, 2651280)
        self.assertEqual(p.parse_args(["start-3d"]).command, "start-3d")

    def test_stubs_say_not_built(self):
        for call in (lambda: steam.installed_games(), lambda: link.start_3d_commands()):
            with self.assertRaises(NotBuilt):
                call()

    def test_wivrn_port(self):
        self.assertEqual(link.WIVRN_PORT, 9757)


if __name__ == "__main__":
    unittest.main()
