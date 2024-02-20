//! Handles the loading of 3D models from files.

use std::io::{self, BufRead};
use std::ops::Range;

pub mod fbx;
pub mod gltf;
pub mod obj;

/// Implementors of this trait may be converted into actual models.
pub trait RawModelData {}

type LineRange = Range<usize>;

/// Iterates over lines in a [buffered reader][BufRead], merging consecutive lines when one ends with a backslash.
///
/// Returned items are given in the form `(line number, text, raw line range)`.
fn lines_escaped<R: BufRead>(reader: R) -> impl Iterator<Item = io::Result<(LineRange, String)>> {
    let mut raw_lines = reader.lines();
    let mut raw_line_num = 0; // Actual line number we're at (as if we'd called `enumerate`)
    let mut num_escaped = 0; // Subtracted from actual line count at the end to determine logical line range

    std::iter::from_fn(move || {
        // Loop and gather lines as long as the previous one ends with a `\`, "processing" them into the final string
        // for this given line.
        let mut processed: Option<String> = None;

        let escaped_line = loop {
            raw_line_num += 1; // Fine to increment first; line numbers start at 1.

            let raw_line = match raw_lines.next() {
                Some(Ok(text)) => text,
                // If we hit an error, short-circuit this function. Can be caught by the consumer.
                Some(Err(err)) => return Some(Err(err)),
                None => {
                    // If the last line of the file has a backslash on it, we might run out of lines while we're
                    // currently holding onto processed line(s). We
                    match processed {
                        Some(text) => break text,
                        None => return None,
                    };
                },
            };

            // If we have any processed text currently pending, append this line. Otherwise, our new pending text is
            // just the freshly read value on its own.
            let mut pending = match processed {
                Some(prev) => prev + &raw_line,
                None => raw_line,
            };

            // Then, check if the newly appended-to
            if pending.ends_with('\\') {
                // If our escaped text still ends with a backslash, trim off that backslash and keep looping. We are
                // safe to truncate by -1 because we now the last character is a plain ASCII backslash.
                pending.truncate(pending.len() - 1);
                processed = Some(pending);
                num_escaped += 1;
            } else {
                // Otherwise, what we've got is the current line.
                break pending;
            }
        };

        // If we're at line 86 of the file, but we've escaped 3 so far, then this is "line 83". Our in-file range is
        // then `83..87`.
        let line_range = (raw_line_num - num_escaped)..(raw_line_num + 1);
        Some(Ok((line_range, escaped_line)))
    })
}


/// Formats a range of line numbers as either `line N` or `lines N to M` if the range is longer than one line.
fn fmt_line_range(range: &LineRange) -> String {
    if range.start == range.end {
        format!("line {}", range.start)
    } else {
        format!("lines {} to {}", range.start, range.end)
    }
}
