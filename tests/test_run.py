import json
import tempfile
import unittest
from pathlib import Path

from bevy_remote_driver.run import PlanError, load, plans_in


class Recorder:
    """Stands in for a `Driver`, recording the steps a plan calls."""

    def __init__(self):
        self.calls = []

    def __getattr__(self, name):
        return lambda *args, **kwargs: self.calls.append((name, args, kwargs))


class Loading(unittest.TestCase):
    def setUp(self):
        self.folder = tempfile.TemporaryDirectory()
        self.dir = Path(self.folder.name)

    def tearDown(self):
        self.folder.cleanup()

    def write(self, name, text):
        path = self.dir / name
        path.write_text(text)
        return path

    def test_a_json_plan_runs_its_steps(self):
        steps = [{"click": "Menu/Start", "hold": 3}, {"absent": "Panel"}]
        plan = self.write("plan.json", json.dumps({"bin": "app", "steps": steps}))
        target, package, features, run = load(plan)
        self.assertEqual((target, package, features), (("bin", "app"), None, None))
        driver = Recorder()
        run(driver)
        self.assertEqual(
            driver.calls,
            [("click", ("Menu/Start",), {"hold": 3}), ("absent", ("Panel",), {})],
        )

    def test_a_json_step_names_exactly_one_step(self):
        steps = [{"click": "A", "absent": "B"}]
        plan = self.write("plan.json", json.dumps({"example": "app", "steps": steps}))
        _, _, _, run = load(plan)
        with self.assertRaisesRegex(PlanError, "names 2 steps"):
            run(Recorder())

    def test_a_python_plan_names_its_target(self):
        plan = self.write(
            "plan.py",
            'EXAMPLE = "app"\nPACKAGE = "pkg"\nFEATURES = ["f"]\n'
            'def run(driver):\n    driver.present("Menu")\n',
        )
        target, package, features, run = load(plan)
        self.assertEqual((target, package, features), (("example", "app"), "pkg", ["f"]))
        driver = Recorder()
        run(driver)
        self.assertEqual(driver.calls, [("present", ("Menu",), {})])

    def test_a_plan_names_exactly_one_target(self):
        both = self.write("both.json", json.dumps({"bin": "a", "example": "b", "steps": []}))
        neither = self.write("neither.py", "def run(driver):\n    pass\n")
        for plan in (both, neither):
            with self.assertRaisesRegex(PlanError, "exactly one"):
                load(plan)

    def test_a_python_plan_that_does_not_load_is_refused(self):
        plan = self.write("broken.py", "import no_such_module\n")
        with self.assertRaisesRegex(PlanError, "did not load"):
            load(plan)

    def test_a_missing_plan_is_refused(self):
        with self.assertRaisesRegex(PlanError, "no plan"):
            load(self.dir / "missing.py")

    def test_a_directory_stands_for_its_plans_in_name_order(self):
        for name in ("b.py", "a.json", "notes.txt"):
            self.write(name, "")
        self.assertEqual(plans_in([self.dir]), [self.dir / "a.json", self.dir / "b.py"])


if __name__ == "__main__":
    unittest.main()
