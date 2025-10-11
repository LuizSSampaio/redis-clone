pub fn parse(source: &[u8]) -> Vec<String> {
    let source = String::from_utf8_lossy(source).to_string();
    let mut res = Vec::new();

    source.split("\r\n").for_each(|line| {
        if line.starts_with('*') || line.starts_with('$') || line.is_empty() {
            return;
        }
        if line.starts_with('+') || line.starts_with(':') {
            res.push(line[1..].to_string());
            return;
        }
        res.push(line.to_string());
    });

    res
}
