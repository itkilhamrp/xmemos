#!/usr/bin/env python3
# Copyright (C) 2026 Xmemos Contributors
#
# This file is part of Xmemos.
#
# Xmemos is free software: you can redistribute it and/or modify
# it under the terms of the GNU Affero General Public License as published by
# the Free Software Foundation, either version 3 of the License, or
# (at your option) any later version.
#
# Xmemos is distributed in the hope that it will be useful,
# but WITHOUT ANY WARRANTY; without even the implied warranty of
# MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
# GNU Affero General Public License for more details.
#
# You should have received a copy of the GNU Affero General Public License
# along with Xmemos. If not, see https://www.gnu.org/licenses/.
#
# SPDX-License-Identifier: AGPL-3.0-or-later

"""Deterministic Windows 10/11 Portable ZIP packager respecting SOURCE_DATE_EPOCH."""

import os
import time
import zipfile
from pathlib import Path


def main() -> None:
    epoch = int(os.environ.get("SOURCE_DATE_EPOCH", "1700000000"))
    version = os.environ["APP_VERSION"]
    dt = time.gmtime(epoch)
    zip_dt = (max(dt.tm_year, 1980), dt.tm_mon, dt.tm_mday, dt.tm_hour, dt.tm_min, dt.tm_sec)

    src_root = Path(f"dist-portable/Xmemos-v{version}-portable")
    out_zip = Path(f"dist-release/Xmemos-v{version}-windows-x64-portable.zip")
    out_zip.parent.mkdir(parents=True, exist_ok=True)

    with zipfile.ZipFile(out_zip, "w", compression=zipfile.ZIP_DEFLATED, compresslevel=9) as zf:
        for path in sorted(src_root.rglob("*")):
            if path.is_file():
                rel = path.relative_to(src_root.parent).as_posix()
                info = zipfile.ZipInfo(rel, date_time=zip_dt)
                info.compress_type = zipfile.ZIP_DEFLATED
                info.external_attr = 0o100644 << 16
                zf.writestr(info, path.read_bytes())

    print(f"Created deterministic portable archive: {out_zip} ({out_zip.stat().st_size} bytes)")


if __name__ == "__main__":
    main()
