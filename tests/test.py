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

    def _rargs(self, input, pattern, command, args=''):
        cmd = f"{input} | {RARGS} {args} '{pattern}' {command}"
        return self._execute(cmd)

    def test_echo(self):
        # echo -e 'a\\nb\nc\\nd' | rargs pattern echo -e {}
        # => a\nb\nc\nd\n

        echo = 'gecho' if sys.platform == 'darwin' else 'echo'
        output, _ = self._rargs(r"echo -e 'a\\nb\nc\\nd'", 'pattern', '{} -e {{}}'.format(echo))
        self.assertEqual(output, 'a\nb\nc\nd\n')

    def test_regex_should_match(self):
        # echo '2018-01-20' | rargs '^(?P<year>\d{4})-(\d{2})-(\d{2})$' echo -e {1} {2} {3}
        # => a\nb\nc\nd\n

        echo = 'gecho' if sys.platform == 'darwin' else 'echo'
        output, _ = self._rargs(r"echo '2018-01-20'",
                                '^(?P<year>\d{4})-(\d{2})-(\d{2})$',
                                '{} -e {{1}} {{2}} {{3}}'.format(echo))
        self.assertEqual(output, '2018 01 20\n')

    def test_regex_group_name_should_match(self):
        # echo '2018-01-20' | rargs '^(?P<year>\d{4})-(\d{2})-(\d{2})$' echo -e {year} {2} {3}
        # => a\nb\nc\nd\n

        echo = 'gecho' if sys.platform == 'darwin' else 'echo'
        output, _ = self._rargs(r"echo '2018-01-20'",
                                '^(?P<year>\d{4})-(\d{2})-(\d{2})$',
                                '{} -e {{year}} {{2}} {{3}}'.format(echo))
        self.assertEqual(output, '2018 01 20\n')

    def test_negtive_regex_group_should_work(self):
        # echo '2018-01-20' | rargs '^(?P<year>\d{4})-(\d{2})-(\d{2})$' echo -e {-3} {-2} {-1}
        # => a\nb\nc\nd\n

        echo = 'gecho' if sys.platform == 'darwin' else 'echo'
        output, _ = self._rargs(r"echo '2018-01-20'",
                                '^(?P<year>\d{4})-(\d{2})-(\d{2})$',
                                '{} -e {{-3}} {{-2}} {{-1}}'.format(echo))
        self.assertEqual(output, '2018 01 20\n')

    def test_read0_short(self):
        echo = 'gecho' if sys.platform == 'darwin' else 'echo'
        output, _ = self._rargs(r"echo -e 'a\0b'", 'pattern', '{} -e X{{}}X'.format(echo), args='-0')
        self.assertEqual(output, 'XaX\nXbX\n')

    def test_read0_long(self):
        echo = 'gecho' if sys.platform == 'darwin' else 'echo'
        output, _ = self._rargs(r"echo -e 'a\0b'", 'pattern', '{} -e X{{}}X'.format(echo), args='--read0')
        self.assertEqual(output, 'XaX\nXbX\n')

    def test_no_read0(self):
        echo = 'gecho' if sys.platform == 'darwin' else 'echo'
        _, err = self._rargs(r"echo -e 'a\0b'", 'pattern', '{} -e X{{}}X'.format(echo))
        self.assertNotEqual(err, None)

if __name__ == '__main__':
    unittest.main()
