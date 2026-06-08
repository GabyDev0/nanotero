pub(crate) fn unescape_string(string: &str) -> String {
    let mut iter = string[0..string.len() - 1].chars();
    let mut r = String::new();
    while let Some(ch) = iter.next() {
        if ch == '\\' {
            // El sistema de escapeo por logica un \ siempre tiene su par
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
