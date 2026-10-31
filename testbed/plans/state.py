"""State written past the UI: a resource replaced and restored from a snapshot, one of its fields
written by reflection path, a component replaced whole and by one field, and a restart that
forgets them."""

BIN = "testbed"
CLEAR = "bevy_camera::clear_color::ClearColor"
NODE = "bevy_ui::ui_node::Node"
TEXT = "bevy_ui::widget::text::Text"
RED = {"Srgba": {"red": 0.8, "green": 0.1, "blue": 0.1, "alpha": 1.0}}


def run(driver):
    driver.present("Menu")  # wait for the scene to finish spawning

    # A snapshot, a change, and the snapshot put back whole.
    saved = driver.get_resource(CLEAR)
    driver.set_resource(CLEAR, RED)
    assert driver.get_resource(CLEAR, at="Srgba.red") == 0.800000011920929
    # One field: the reflection path steps into the newtype and the variant (`0.0`), where the
    # value as BRP shows it has neither.
    driver.set_resource(CLEAR, 0.5, at="0.0.green")
    assert driver.get_resource(CLEAR, at="Srgba.green") == 0.5
    driver.screenshot("cleared")
    driver.set_resource(CLEAR, saved)
    assert driver.get_resource(CLEAR) == saved

    # A component, whole and by one field.
    driver.set("Menu/Typed", TEXT, "Set directly")
    driver.expect("Menu/Typed", TEXT, "Set directly")
    driver.set("Menu/Start", NODE, {"Px": 200.0}, at="width")
    driver.expect("Menu/Start", NODE, {"Px": 200.0}, at="width")
    driver.screenshot("written")

    # A second launch starts over, since the testbed saves nothing.
    driver.restart()
    driver.expect("Menu/Typed", TEXT, "")
    driver.expect("Menu/Start", NODE, "Auto", at="width")
