"""Start opens the panel, typing reaches the text field, and Escape closes the panel."""

from _testbed_driver import TestbedDriver

BIN = "testbed"
DRIVER = TestbedDriver  # a step of the testbed's own, `open_panel`
TEXT = "bevy_ui::widget::text::Text"


def run(driver):
    driver.present("Menu")  # wait for the scene to finish spawning
    driver.screenshot("menu")
    driver.open_panel()  # clicks Start, a UI node by its path of Names, and waits for the panel
    driver.type("Hi!")
    driver.expect("Menu/Typed", TEXT, "Hi!")
    driver.key("Escape")
    driver.absent("Panel")
