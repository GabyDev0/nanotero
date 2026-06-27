#[doc(hidden)]
pub fn unescape_string(string: &str) -> String {
    let mut iter = string.chars();
    let mut r = String::new();
    while let Some(ch) = iter.next() {
        if ch == '\\' {
            // The logical ‘one’ escape system always has a counterpart
            r.push(match unsafe { iter.next().unwrap_unchecked() } {
                't' => '\t',
                'r' => '\r',
                '0' => '\0',
                'n' => '\n',
                chn => chn,
            });
        } else {
            r.push(ch);
        }
    }
    r
}
