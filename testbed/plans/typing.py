"""Each character typed shows up before the next one is sent."""

BIN = "testbed"

TEXT = "a-Z?"
TYPED = "bevy_ui::widget::text::Text"


def run(driver):
    driver.present("Menu")
    for end in range(1, len(TEXT) + 1):
        driver.type(TEXT[end - 1])
        driver.expect("Menu/Typed", TYPED, TEXT[:end], timeout={"frames": 5})
