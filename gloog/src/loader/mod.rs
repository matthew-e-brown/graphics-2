//! Handles the loading of 3D models from files.

use std::io::{self, BufRead};
use std::ops::Range;

pub mod dae;
pub mod fbx;
pub mod gltf;
pub mod obj;


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
            raw_line_num += 1; // Fine to increment before grabbing from `raw_lines` iterator; line numbers start at 1.

            let raw_line = match raw_lines.next() {
                // Next line in file is just regular text.
                Some(Ok(text)) => text,
                // Failed to read next line; throw the error up to the consumer.
                Some(Err(err)) => return Some(Err(err)),
                // There are no more lines in the file. If `processed` is still Some, that means that the previous line
                // ended with a backslash, and so we _should_ have found another line here. Otherwise, we can just
                // return `None` and stop iteration.
                None if processed.is_some() => return Some(Err(io::ErrorKind::UnexpectedEof.into())),
                None => return None,
            };

            // If we have any processed text currently pending, append this line to it. Otherwise, our new pending text
            // is just the freshly read value on its own.
            let mut pending = match processed {
                Some(prev) => prev + &raw_line,
                None => raw_line,
            };

            // If our collection of lines still ends with a backslash, trim it off and keep looping.
            if pending.ends_with('\\') {
                // Safe to truncate by 1 byte since we know the last character is a plain ASCII backslash
                pending.truncate(pending.len() - 1);
                processed = Some(pending);
                num_escaped += 1;
            } else {
                // Otherwise, what we've got is the final line.
                break pending;
            }
        };

        // If we're at line 86 of the file, but we've escaped 3 so far, then this is "line 83". Our in-file range is
        // then `83..87`.
        let line_range = (raw_line_num - num_escaped)..(raw_line_num + 1);
        Some(Ok((line_range, escaped_line)))
    })
}


/// Grab everything in a line up to the first line-comment character(s). Also trims the start and end of the string.
fn trim_line_comment<'a>(line: &'a str, comment: &str) -> &'a str {
    match line.find(comment) {
        Some(i) => line[0..i].trim(),
        None => line[0..].trim(),
    }
}


/// Formats a range of line numbers as either `line N` or `lines N to M` if the range is longer than one line.
fn fmt_line_range(range: &LineRange) -> String {
    // Range is exclusive, hence - 1
    if range.start == range.end - 1 {
        format!("line {}", range.start)
    } else {
        format!("lines {} to {}", range.start, range.end)
    }
}
