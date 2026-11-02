# Copyright (c) 2026 CCP Games

import os
flavor = os.environ.get("BUILDFLAVOR", "release")

if flavor == 'release':
    from _blueexposuretest import *
elif flavor == 'debug':
    from _blueexposuretest_debug import *
elif flavor == 'trinitydev':
    from _blueexposuretest_trinitydev import *
elif flavor == 'internal':
    from _blueexposuretest_internal import *
else:
    raise RuntimeError("Unknown build flavor: {}".format(flavor))
