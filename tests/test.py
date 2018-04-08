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
        cmd = subprocess.run([self.SHELL, '-c', command], stdout=subprocess.PIPE)
        output = cmd.stdout
        return output.decode('utf8') if output is not None else None

    def _rargs(self, input, pattern, command):
        cmd = f'{input} | {RARGS} {pattern} {command}'
        return self._execute(cmd)

    def test_echo(self):
        echo = 'gecho' if sys.platform == 'darwin' else 'echo'
        output = self._rargs(r"echo -e 'a\\nb\nc\\nd'", 'pattern', '{} -e {{}}'.format(echo))
        self.assertEqual(output, 'a\nb\nc\nd\n')

if __name__ == '__main__':
    unittest.main()
