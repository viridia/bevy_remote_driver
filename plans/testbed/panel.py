"""Start opens the panel, typing reaches the text field, and Escape closes the panel."""

EXAMPLE = "testbed"
TEXT = "bevy_ui::widget::text::Text"


def run(driver):
    driver.present("Menu", ready=True)  # wait for the scene to finish spawning
    driver.screenshot("menu")
    driver.click("Menu/Start")  # a UI node, by its path of Names
    driver.present("Panel")  # waits until it appears
    driver.type("Hi!")
    driver.expect("Menu/Typed", TEXT, "Hi!")
    driver.key("Escape")
    driver.absent("Panel")
