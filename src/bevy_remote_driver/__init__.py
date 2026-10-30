"""Drive a running Bevy app from outside, over the Bevy Remote Protocol, for end-to-end tests.

A plan imports what it uses from here: `from bevy_remote_driver import Between`.
"""

from .driver import Between, Driver

__all__ = ["Between", "Driver"]
