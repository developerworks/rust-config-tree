//! Small formatting helpers shared inside the high-level config module.

/// Ensures generated text has exactly one trailing newline.
pub(crate) fn ensure_single_trailing_newline(output: &mut String) {
    if output.ends_with('\n') {
        while output.ends_with("\n\n") {
            output.pop();
        }
    } else {
        output.push('\n');
    }
}
