import struct
import tempfile
import unittest
import zlib
from pathlib import Path

from bevy_remote_driver import Between
from bevy_remote_driver.driver import field_path, path_of, reflect_path, subset, walk, write_png


class Selectors(unittest.TestCase):
    def test_a_string_splits_on_slashes(self):
        self.assertEqual(path_of("Settings/Audio/Volume"), ["Settings", "Audio", "Volume"])

    def test_a_list_is_taken_as_it_is(self):
        self.assertEqual(path_of(["Menu", 0]), ["Menu", 0])
        self.assertEqual(path_of(["A/B"]), ["A/B"])


class Paths(unittest.TestCase):
    def test_a_reflection_path_becomes_keys_and_indices(self):
        self.assertEqual(field_path("limbs[0].angle.shape"), ["limbs", 0, "angle", "shape"])

    def test_a_list_is_taken_as_it_is(self):
        self.assertEqual(field_path(["a.b", 1]), ["a.b", 1])

    def test_a_reflection_path_for_mutate_starts_with_a_dot(self):
        self.assertEqual(reflect_path("limbs[0].angle"), ".limbs[0].angle")
        self.assertEqual(reflect_path(["limbs", 0, "angle"]), ".limbs[0].angle")

    def test_walk_reaches_the_part(self):
        value = {"limbs": [{"angle": 1}, {"angle": 2}]}
        self.assertEqual(walk(value, ["limbs", 1, "angle"]), (2, None))

    def test_walk_says_where_it_stopped(self):
        value = {"limbs": [{"angle": 1}]}
        self.assertEqual(walk(value, ["limbs", 3, "angle"]), (None, ["limbs", 3]))
        self.assertEqual(walk(value, ["legs"]), (None, ["legs"]))
        self.assertEqual(walk(value, [0]), (None, [0]))


class Subset(unittest.TestCase):
    def test_only_the_keys_expected_are_compared(self):
        self.assertTrue(subset({"a": {"b": 1}}, {"a": {"b": 1, "c": 2}, "d": 3}))

    def test_a_missing_or_different_key_fails(self):
        self.assertFalse(subset({"a": 1}, {"b": 1}))
        self.assertFalse(subset({"a": {"b": 1}}, {"a": {"b": 2}}))
        self.assertFalse(subset({"a": {"b": 1}}, {"a": 1}))

    def test_lists_are_compared_whole(self):
        self.assertFalse(subset([1], [1, 2]))

    def test_a_matcher_matches_by_its_own_test(self):
        self.assertTrue(subset({"n": Between(1, 10)}, {"n": 5}))
        self.assertFalse(subset({"n": Between(1, 10)}, {"n": 11}))


class BetweenTest(unittest.TestCase):
    def test_both_ends_are_included(self):
        self.assertEqual(Between(1, 2), 1)
        self.assertEqual(Between(1, 2), 2.0)
        self.assertNotEqual(Between(1, 2), 2.5)

    def test_only_numbers_match(self):
        self.assertNotEqual(Between(0, 2), True)
        self.assertNotEqual(Between(0, 2), "1")
        self.assertNotEqual(Between(0, 2), None)


class Png(unittest.TestCase):
    def test_the_file_holds_the_pixels(self):
        rgba = bytes(range(2 * 3 * 4))
        with tempfile.TemporaryDirectory() as folder:
            path = Path(folder) / "out.png"
            write_png(path, 2, 3, rgba)
            data = path.read_bytes()
        self.assertEqual(data[:8], b"\x89PNG\r\n\x1a\n")
        chunks, at = {}, 8
        while at < len(data):
            (length,) = struct.unpack(">I", data[at : at + 4])
            tag, body = data[at + 4 : at + 8], data[at + 8 : at + 8 + length]
            (crc,) = struct.unpack(">I", data[at + 8 + length : at + 12 + length])
            self.assertEqual(crc, zlib.crc32(tag + body), tag)
            chunks[tag] = body
            at += 12 + length
        self.assertEqual(struct.unpack(">IIBBBBB", chunks[b"IHDR"]), (2, 3, 8, 6, 0, 0, 0))
        rows = zlib.decompress(chunks[b"IDAT"])
        # Each row is its filter byte, none, then its pixels.
        self.assertEqual(rows, b"".join(b"\x00" + rgba[y * 8 : (y + 1) * 8] for y in range(3)))
        self.assertEqual(chunks[b"IEND"], b"")


if __name__ == "__main__":
    unittest.main()
