#!/usr/bin/env python3
# -*- coding: utf-8 -*-

import unittest
import subprocess
import sys
import os

SCRIPT_PATH = os.path.realpath(__file__)
BASE = os.path.expanduser(os.path.join(os.path.dirname(SCRIPT_PATH), '../'))
os.chdir(BASE)
RARGS = f"{BASE}/target/debug/rargs"

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

    def _echo(self):
        return 'gecho' if sys.platform == 'darwin' else 'echo'

    def test_echo(self):
        # echo -e 'a\\nb\nc\\nd' | rargs pattern echo -e {}
        # => a\nb\nc\nd\n
        output, _ = self._rargs(input=r"echo -e 'a\\nb\nc\\nd'",
                                command='{} -e {{}}'.format(self._echo()))
        self.assertEqual(output, 'a\nb\nc\nd\n')

    def test_regex_should_match(self):
        # echo '2018-01-20' | rargs '^(?P<year>\d{4})-(\d{2})-(\d{2})$' echo -e {1} {2} {3}
        # => a\nb\nc\nd\n
        output, _ = self._rargs(input=r"echo '2018-01-20'",
                                args="""-p '^(?P<year>\d{4})-(\d{2})-(\d{2})$'""",
                                command='{} -e {{1}} {{2}} {{3}}'.format(self._echo()))
        self.assertEqual(output, '2018 01 20\n')

    def test_regex_group_name_should_match(self):
        # echo '2018-01-20' | rargs '^(?P<year>\d{4})-(\d{2})-(\d{2})$' echo -e {year} {2} {3}
        # => a\nb\nc\nd\n
        output, _ = self._rargs(input=r"echo '2018-01-20'",
                                args="""-p '^(?P<year>\d{4})-(\d{2})-(\d{2})$'""",
                                command='{} -e {{year}} {{2}} {{3}}'.format(self._echo()))
        self.assertEqual(output, '2018 01 20\n')

    def test_negtive_regex_group_should_work(self):
        # echo '2018-01-20' | rargs '^(?P<year>\d{4})-(\d{2})-(\d{2})$' echo -e {-3} {-2} {-1}
        # => a\nb\nc\nd\n
        output, _ = self._rargs(input=r"echo '2018-01-20'",
                                args=r" -p '^(?P<year>\d{4})-(\d{2})-(\d{2})$'",
                                command='{} -e {{-3}} {{-2}} {{-1}}'.format(self._echo()))
        self.assertEqual(output, '2018 01 20\n')

    def test_read0_short(self):
        output, _ = self._rargs(input=r"echo -e 'a\0b'",
                                args='-0',
                                command='{} -e X{{}}X'.format(self._echo()))
        self.assertEqual(output, 'XaX\nXbX\n')

    def test_read0_long(self):
        output, _ = self._rargs(input=r"echo -e 'a\0b'",
                                args='--read0',
                                command='{} -e X{{}}X'.format(self._echo()))
        self.assertEqual(output, 'XaX\nXbX\n')

    def test_no_read0(self):
        _, err = self._rargs(input=r"echo -e 'a\0b'",
                             command='{} -e X{{}}X'.format(self._echo()))
        self.assertNotEqual(err, None)

    def test_default_delimiter(self):
        output, _ = self._rargs(input=r"echo -e 'a b  c'",
                                command='{} -e X{{1}},{{2}},{{3}}X'.format(self._echo()))
        self.assertEqual(output, "Xa,b,cX\n")

    def test_delimiter(self):
        output, _ = self._rargs(input=r"echo -e 'a,b,,c'",
                                args='-d ,',
                                command='{} -e X{{1}},{{2}},{{3}},{{4}}X'.format(self._echo()))
        self.assertEqual(output, "Xa,b,,cX\n")

    def test_range_left_inf(self):
        output, _ = self._rargs(input=r"echo -e '1,2,3,4,5,6'",
                                args='-d ,',
                                command='{} -e X{{..3}}X'.format(self._echo()))
        self.assertEqual(output, "X1 2 3X\n")

        output, _ = self._rargs(input=r"echo -e '1,2,3,4,5,6'",
                                args='-d ,',
                                command='{} -e X{{..-2}}X'.format(self._echo()))
        self.assertEqual(output, "X1 2 3 4 5X\n")

        output, _ = self._rargs(input=r"echo -e '1,2,3,4,5,6'",
                                args='-d ,',
                                command='{} -e X{{..0}}X'.format(self._echo()))
        self.assertEqual(output, "XX\n")

    def test_range_right_inf(self):
        output, _ = self._rargs(input=r"echo -e '1,2,3,4,5,6'",
                                args='-d ,',
                                command='{} -e X{{3..}}X'.format(self._echo()))
        self.assertEqual(output, "X3 4 5 6X\n")

        output, _ = self._rargs(input=r"echo -e '1,2,3,4,5,6'",
                                args='-d ,',
                                command='{} -e X{{-2..}}X'.format(self._echo()))
        self.assertEqual(output, "X5 6X\n")

        output, _ = self._rargs(input=r"echo -e '1,2,3,4,5,6'",
                                args='-d ,',
                                command='{} -e X{{7..}}X'.format(self._echo()))
        self.assertEqual(output, "XX\n")

    def test_range_both(self):
        output, _ = self._rargs(input=r"echo -e '1,2,3,4,5,6'",
                                args='-d ,',
                                command='{} -e "X{{3..3}}X"'.format(self._echo()))
        self.assertEqual(output, "X3X\n")

        output, _ = self._rargs(input=r"echo -e '1,2,3,4,5,6'",
                                args='-d ,',
                                command='{} -e "X{{3..4}}X"'.format(self._echo()))
        self.assertEqual(output, "X3 4X\n")

        output, _ = self._rargs(input=r"echo -e '1,2,3,4,5,6'",
                                args='-d ,',
                                command='{} -e "X{{3..7}}X"'.format(self._echo()))
        self.assertEqual(output, "X3 4 5 6X\n")

        output, _ = self._rargs(input=r"echo -e '1,2,3,4,5,6'",
                                args='-d ,',
                                command='{} -e "X{{4..3}}X"'.format(self._echo()))
        self.assertEqual(output, "XX\n")

    def test_field_separator(self):
        output, _ = self._rargs(input=r"echo -e '1,2,3,4,5,6'",
                                args='-d ,',
                                command='{} -e "X{{3..4:_}}X"'.format(self._echo()))
        self.assertEqual(output, "X3_4X\n")

        output, _ = self._rargs(input=r"echo -e '1,2,3,4,5,6'",
                                args='-d ,',
                                command='{} -e "X{{3..4:-}}X"'.format(self._echo()))
        self.assertEqual(output, "X3-4X\n")

    def test_global_field_separator(self):
        output, _ = self._rargs(input=r"echo -e '1,2,3,4,5,6'",
                                args='-d , -s /',
                                command='{} -e "X{{3..4}}X"'.format(self._echo()))
        self.assertEqual(output, "X3/4X\n")

        output, _ = self._rargs(input=r"echo -e '1,2,3,4,5,6'",
                                args='-d , -s /',
                                command='{} -e "X{{3..4:,}}X"'.format(self._echo()))
        self.assertEqual(output, "X3,4X\n")

if __name__ == '__main__':
    unittest.main()
