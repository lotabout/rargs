#!/usr/bin/env python3
# -*- coding: utf-8 -*-

import unittest
import subprocess
import sys
import os

SCRIPT_PATH = os.path.realpath(__file__)
BASE = os.path.expanduser(os.path.join(os.path.dirname(SCRIPT_PATH), '../'))
os.chdir(BASE)
RARGS = f"{BASE}/target/release/rargs"

class TestRargs(unittest.TestCase):
    if sys.platform == 'darwin':
        SHELL = 'bash'
    elif sys.platform == 'linux':
        SHELL = 'sh'

    def _execute(self, command):
        cmd = subprocess.run([self.SHELL, '-c', command], stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        output = cmd.stdout
        output = output.decode('utf8') if output is not None else None
        err = cmd.stderr
        err = err.decode('utf8') if err is not None else None
        return (output, err)

    def _rargs(self, *, input, args='', command):
        cmd = f"{input} | {RARGS} {args} {command}"
        return self._execute(cmd)

    def test_regex_should_match(self):
        # echo '2018-01-20' | rargs '^(?P<year>\d{4})-(\d{2})-(\d{2})$' echo -n {1} {2} {3}
        # => a\nb\nc\nd\n
        output, _ = self._rargs(input=r"echo '2018-01-20'",
                                args="""-p '^(?P<year>\d{4})-(\d{2})-(\d{2})$'""",
                                command='echo -n {1} {2} {3}')
        self.assertEqual(output, '2018 01 20')

    def test_regex_group_name_should_match(self):
        # echo '2018-01-20' | rargs '^(?P<year>\d{4})-(\d{2})-(\d{2})$' echo -n {year} {2} {3}
        # => a\nb\nc\nd\n
        output, _ = self._rargs(input=r"echo '2018-01-20'",
                                args="""-p '^(?P<year>\d{4})-(\d{2})-(\d{2})$'""",
                                command='echo -n {year} {2} {3}')
        self.assertEqual(output, '2018 01 20')

    def test_negtive_regex_group_should_work(self):
        # echo '2018-01-20' | rargs '^(?P<year>\d{4})-(\d{2})-(\d{2})$' echo -n {-3} {-2} {-1}
        # => a\nb\nc\nd\n
        output, _ = self._rargs(input=r"echo '2018-01-20'",
                                args=r" -p '^(?P<year>\d{4})-(\d{2})-(\d{2})$'",
                                command='echo -n {-3} {-2} {-1}')
        self.assertEqual(output, '2018 01 20')

    def test_read0_short(self):
        find_output, _ = self._execute('find . -d 1')
        output, _ = self._rargs(input="find . -d 1 -print0",
                                args='-0',
                                command='echo {}')
        self.assertEqual(output, find_output)

    def test_read0_long(self):
        find_output, _ = self._execute('find . -d 1')
        output, _ = self._rargs(input="find . -d 1 -print0",
                                args='--read0',
                                command='echo {}')
        self.assertEqual(output, find_output)

    def test_no_read0(self):
        _, err = self._rargs(input=r"echo -n 'a\0b'",
                             command='echo -n X{}X')
        self.assertNotEqual(err, None)

    def test_default_delimiter(self):
        output, _ = self._rargs(input=r"echo -n 'a b  c'",
                                command='echo -n X{1},{2},{3}X')
        self.assertEqual(output, "Xa,b,cX")

    def test_delimiter(self):
        output, _ = self._rargs(input=r"echo -n 'a,b,,c'",
                                args='-d ,',
                                command='echo -n X{1},{2},{3},{4}X')
        self.assertEqual(output, "Xa,b,,cX")

    def test_range_left_inf(self):
        output, _ = self._rargs(input=r"echo -n '1,2,3,4,5,6'",
                                args='-d ,',
                                command='echo -n X{..3}X')
        self.assertEqual(output, "X1 2 3X")

        output, _ = self._rargs(input=r"echo -n '1,2,3,4,5,6'",
                                args='-d ,',
                                command='echo -n X{..-2}X')
        self.assertEqual(output, "X1 2 3 4 5X")

        output, _ = self._rargs(input=r"echo -n '1,2,3,4,5,6'",
                                args='-d ,',
                                command='echo -n X{..0}X')
        self.assertEqual(output, "XX")

    def test_range_right_inf(self):
        output, _ = self._rargs(input=r"echo -n '1,2,3,4,5,6'",
                                args='-d ,',
                                command='echo -n X{3..}X')
        self.assertEqual(output, "X3 4 5 6X")

        output, _ = self._rargs(input=r"echo -n '1,2,3,4,5,6'",
                                args='-d ,',
                                command='echo -n X{-2..}X')
        self.assertEqual(output, "X5 6X")

        output, _ = self._rargs(input=r"echo -n '1,2,3,4,5,6'",
                                args='-d ,',
                                command='echo -n X{7..}X')
        self.assertEqual(output, "XX")

    def test_range_both(self):
        output, _ = self._rargs(input=r"echo -n '1,2,3,4,5,6'",
                                args='-d ,',
                                command='echo -n "X{3..3}X"')
        self.assertEqual(output, "X3X")

        output, _ = self._rargs(input=r"echo -n '1,2,3,4,5,6'",
                                args='-d ,',
                                command='echo -n "X{3..4}X"')
        self.assertEqual(output, "X3 4X")

        output, _ = self._rargs(input=r"echo -n '1,2,3,4,5,6'",
                                args='-d ,',
                                command='echo -n "X{3..7}X"')
        self.assertEqual(output, "X3 4 5 6X")

        output, _ = self._rargs(input=r"echo -n '1,2,3,4,5,6'",
                                args='-d ,',
                                command='echo -n "X{4..3}X"')
        self.assertEqual(output, "XX")

    def test_field_separator(self):
        output, _ = self._rargs(input=r"echo -n '1,2,3,4,5,6'",
                                args='-d ,',
                                command='echo -n "X{3..4:_}X"')
        self.assertEqual(output, "X3_4X")

        output, _ = self._rargs(input=r"echo -n '1,2,3,4,5,6'",
                                args='-d ,',
                                command='echo -n "X{3..4:-}X"')
        self.assertEqual(output, "X3-4X")

    def test_global_field_separator(self):
        output, _ = self._rargs(input=r"echo -n '1,2,3,4,5,6'",
                                args='-d , -s /',
                                command='echo -n "X{3..4}X"')
        self.assertEqual(output, "X3/4X")

        output, _ = self._rargs(input=r"echo -n '1,2,3,4,5,6'",
                                args='-d , -s /',
                                command='echo -n "X{3..4:,}X"')
        self.assertEqual(output, "X3,4X")

if __name__ == '__main__':
    unittest.main()
