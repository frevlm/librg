# librg

ripgrep 14.1.1 as a static library with a C interface. The caller passes rg
its arguments as a user would write them; rg parses them, walks the files,
reads each once and matches it, and hands back every file it matched in: its
path, the bytes it searched, the lines it matched. stdin is one more file, read
when rg's own rules say so. Nothing is printed.

```c
int rg_search(const char* const* argv, size_t argc, void* context,
              void (*found)(...), void (*failed)(...));
```

See [`include/rg.h`](include/rg.h).

## What is ripgrep's

`ripgrep/` is `crates/core` of the ripgrep 14.1.1 release, unchanged but for
three patches, each marked `librg:`:

- `flags/parse.rs`: `parse_args`, which parses the arguments it is given
  rather than the process's, reads no config file and leaves the logger alone
- `flags/mod.rs`: exports `parse_args`
- `flags/hiargs.rs`: `HiArgs::binary_detection`, the binary detection rg's
  search worker picks for an explicit or an implicit file

Its crates (`grep`, `ignore` and the rest) come from crates.io at the versions
14.1.1 releases them with, pinned with `=` in `Cargo.toml`; `Cargo.lock`
started from 14.1.1's. `src/lib.rs` is the interface.

To move to another ripgrep: copy its `crates/core` (without `main.rs`) over
`ripgrep/`, apply the three patches again, set the `=` versions and the
lockfile from that release.

## Build and release

```bash
./package.sh aarch64-apple-darwin    # dist/librg-<version>-<target>.tar.gz and .sha256
```

The archive holds `include/rg.h`, `lib/librg.a` (`lib/rg.lib` on Windows),
`LICENSE` and ripgrep's `LICENSE-ripgrep`. The library needs from the system
what Rust's standard library does: `iconv` on macOS; `ws2_32 userenv ntdll
bcrypt advapi32 kernel32` on Windows; `pthread dl m` on Linux.

A tag `v<version>`, the version in `Cargo.toml`, builds
`aarch64-apple-darwin`, `x86_64-unknown-linux-musl` and
`x86_64-pc-windows-msvc` and publishes them as a GitHub release, each with its
checksum.

## License

MIT: [`LICENSE`](LICENSE). `ripgrep/` is ripgrep's code, which its authors
offer under the Unlicense or MIT; it is used here under MIT, with its notice in
[`ripgrep/LICENSE-MIT`](ripgrep/LICENSE-MIT).
