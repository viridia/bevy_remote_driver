"""The testbed's own steps, for its plans: `from _testbed_driver import TestbedDriver`."""

from bevy_remote_driver import Driver, step


class TestbedDriver(Driver):
    @step
    def open_panel(self):
        """Clicks Start and waits until the panel is shown."""
        self.click("Menu/Start")
        self.present("Panel")
