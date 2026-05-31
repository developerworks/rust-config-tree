//! Small formatting helpers shared inside the high-level config module.

/// Ensures generated text has exactly one trailing newline.
///
/// # Arguments
///
/// - `output`: Mutable generated text to normalize in place.
///
/// # Returns
///
/// Returns no value; `output` is updated directly.
///
/// # Examples
///
/// ```no_run
/// let _ = ();
/// ```
pub(crate) fn ensure_single_trailing_newline(output: &mut String) {
    if output.ends_with('\n') {
        while output.ends_with("\n\n") {
            output.pop();
        }
    } else {
        output.push('\n');
    }
}
