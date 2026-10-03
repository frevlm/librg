//! ripgrep as a library: its command line, walk, matcher and binary
//! detection, unchanged, vendored from 14.1.1 in ../ripgrep. What it
//! gives back is not printed lines but every file it matched in, read once:
//! the bytes it searched and the lines it matched. stdin is one more file, read
//! when rg's own rules say so, and its bytes are the ones it searched.

#[macro_use]
#[allow(dead_code)]
#[path = "../ripgrep/messages.rs"]
mod messages;
#[allow(dead_code, unused_imports)]
#[path = "../ripgrep/flags/mod.rs"]
mod flags;
#[allow(dead_code)]
#[path = "../ripgrep/haystack.rs"]
mod haystack;
#[allow(dead_code)]
#[path = "../ripgrep/logger.rs"]
mod logger;
#[allow(dead_code)]
#[path = "../ripgrep/search.rs"]
mod search;

use std::{
    ffi::{c_char, c_int, c_void, CStr, OsString},
    io::{self, Read},
    sync::Mutex,
};

use grep::searcher::{Searcher, Sink, SinkMatch};
use ignore::WalkState;

use crate::{flags::HiArgs, haystack::Haystack, search::PatternMatcher};

/// A file rg matched in: its path as rg prints it, '/'-separated; the bytes it
/// searched; the lines it matched, 1-based and ascending.
struct Hit {
    path: String,
    bytes: Vec<u8>,
    lines: Vec<u64>,
}

/// The lines of each match. Binary data ends the search and drops the file,
/// as rg drops a binary file from its output.
#[derive(Default)]
struct Lines {
    lines: Vec<u64>,
    binary: bool,
}

impl Sink for Lines {
    type Error = io::Error;

    fn matched(&mut self, _: &Searcher, m: &SinkMatch<'_>) -> io::Result<bool> {
        let first = m.line_number().unwrap_or(0);
        self.lines.extend((first..).take(m.lines().count()));
        Ok(true)
    }

    fn binary_data(&mut self, _: &Searcher, _: u64) -> io::Result<bool> {
        self.binary = true;
        Ok(false)
    }
}

fn search(
    hi: &HiArgs,
    matcher: &PatternMatcher,
    searcher: &mut Searcher,
    haystack: &Haystack,
) -> io::Result<Option<Hit>> {
    let mut bytes = Vec::new();
    if haystack.is_stdin() {
        io::stdin().lock().read_to_end(&mut bytes)?;
    } else {
        bytes = std::fs::read(haystack.path())?;
    }
    searcher.set_binary_detection(hi.binary_detection(haystack.is_explicit()));
    let mut lines = Lines::default();
    match matcher {
        PatternMatcher::RustRegex(m) => searcher.search_slice(m, &bytes, &mut lines)?,
    }
    if lines.binary || lines.lines.is_empty() {
        return Ok(None);
    }
    let path = haystack.path().to_string_lossy();
    let path = if cfg!(windows) { path.replace('\\', "/") } else { path.into_owned() };
    Ok(Some(Hit { path, bytes, lines: lines.lines }))
}

/// rg with `args` (no "rg" in front), every file it matched in, by path.
fn run(mut args: Vec<OsString>) -> anyhow::Result<Vec<Hit>> {
    // The lines are what the caller is after.
    args.insert(0, "--line-number".into());
    let hi = match flags::parse_args(args) {
        flags::ParseResult::Ok(hi) => hi,
        flags::ParseResult::Err(err) => return Err(err),
        flags::ParseResult::Special(_) => anyhow::bail!("--help and --version are not taken here"),
    };
    if !matches!(hi.mode(), flags::Mode::Search(_)) {
        anyhow::bail!("only a search is taken here: not --files, --type-list or the like");
    }
    if !hi.matches_possible() {
        return Ok(Vec::new());
    }
    let matcher = hi.matcher()?;
    let searcher = hi.searcher()?;
    let haystacks = hi.haystack_builder();
    let found = Mutex::new(Vec::new());
    hi.walk_builder()?.build_parallel().run(|| {
        let (hi, matcher, haystacks, found) = (&hi, &matcher, &haystacks, &found);
        let mut searcher = searcher.clone();
        Box::new(move |result| {
            let Some(haystack) = haystacks.build_from_result(result) else {
                return WalkState::Continue;
            };
            match search(hi, matcher, &mut searcher, &haystack) {
                Ok(Some(hit)) => found.lock().unwrap().push(hit),
                Ok(None) => {}
                Err(err) => {
                    err_message!("{}: {}", haystack.path().display(), err);
                }
            }
            WalkState::Continue
        })
    });
    let mut found = found.into_inner().unwrap();
    found.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(found)
}

#[cfg(unix)]
fn os(arg: &CStr) -> OsString {
    use std::os::unix::ffi::OsStrExt;
    std::ffi::OsStr::from_bytes(arg.to_bytes()).to_os_string()
}

#[cfg(not(unix))]
fn os(arg: &CStr) -> OsString {
    arg.to_string_lossy().into_owned().into()
}

type Found = extern "C" fn(*mut c_void, *const c_char, usize, *const u8, usize, *const u64, usize);
type Failed = extern "C" fn(*mut c_void, *const c_char, usize);

/// See include/rg.h.
#[no_mangle]
pub extern "C" fn rg_search(
    argv: *const *const c_char,
    argc: usize,
    context: *mut c_void,
    found: Found,
    failed: Failed,
) -> c_int {
    let args: Vec<OsString> = (0..argc).map(|i| os(unsafe { CStr::from_ptr(*argv.add(i)) })).collect();
    let fail = |message: String| {
        failed(context, message.as_ptr().cast(), message.len());
        2
    };
    match std::panic::catch_unwind(|| run(args)) {
        Ok(Ok(hits)) => {
            for h in &hits {
                found(context, h.path.as_ptr().cast(), h.path.len(), h.bytes.as_ptr(), h.bytes.len(),
                      h.lines.as_ptr(), h.lines.len());
            }
            if hits.is_empty() { 1 } else { 0 }
        }
        Ok(Err(err)) => fail(format!("rg: {err:#}")),
        Err(_) => fail("rg: panicked".into()),
    }
}
