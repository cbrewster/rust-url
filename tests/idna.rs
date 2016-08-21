extern crate url;

use std::char;
use url::idna;

#[test]
fn test_uts46() {
    // http://www.unicode.org/Public/idna/latest/IdnaTest.txt
    for line in include_str!("IdnaTest.txt").lines() {
        if line == "" || line.starts_with("#") {
            continue
        }
        // Remove comments
        let mut line = match line.find("#") {
            Some(index) => &line[0..index],
            None => line
        };

        let mut expected_failure = false;
        if line.starts_with("XFAIL") {
            expected_failure = true;
            line = &line[5..line.len()];
        };

        let mut pieces = line.split(';').map(|x| x.trim()).collect::<Vec<&str>>();

        let test_type = pieces.remove(0);
        let original = pieces.remove(0);
        let source = unescape(original);
        let to_unicode = pieces.remove(0);
        let to_ascii = pieces.remove(0);
        let _nv8 = if pieces.len() > 0 { pieces.remove(0) } else { "" };

        if expected_failure {
            continue;
        }

        let result = idna::uts46_to_ascii(&source, idna::Uts46Flags {
            use_std3_ascii_rules: true,
            transitional_processing: test_type == "T",
            verify_dns_length: true,
        });

        if to_ascii.starts_with("[") {
            if to_ascii.starts_with("[C") {
                // http://unicode.org/reports/tr46/#Deviations
                // applications that perform IDNA2008 lookup are not required to check for these contexts
                continue;
            }
            let res = result.ok();
            assert!(res == None, "Expected error. result: {} | original: {} | source: {}", res.unwrap(), original, source);
            continue;
        }

        let to_ascii = if to_ascii.len() > 0 {
            to_ascii.to_string()
        } else {
            if to_unicode.len() > 0 {
                to_unicode.to_string()
            } else {
                source.clone()
            }
        };

        if _nv8 == "NV8" {
            // This result isn't valid under IDNA2008. Skip it
            continue;
        }

        assert!(result.is_ok(), "Couldn't parse {} | original: {} | error: {:?}", source, original, result.err());
        let output = result.ok().unwrap();
        assert!(output == to_ascii, "result: {} | expected: {} | original: {} | source: {}", output, to_ascii, original, source);
    }
}

fn unescape(input: &str) -> String {
    let mut output = String::new();
    let mut chars = input.chars();
    loop {
        match chars.next() {
            None => return output,
            Some(c) =>
                if c == '\\' {
                    match chars.next().unwrap() {
                        '\\' => output.push('\\'),
                        'u' => {
                            let c1 = chars.next().unwrap().to_digit(16).unwrap();
                            let c2 = chars.next().unwrap().to_digit(16).unwrap();
                            let c3 = chars.next().unwrap().to_digit(16).unwrap();
                            let c4 = chars.next().unwrap().to_digit(16).unwrap();
                            match char::from_u32((((c1 * 16 + c2) * 16 + c3) * 16 + c4))
                            {
                                Some(c) => output.push(c),
                                None => { output.push_str(&format!("\\u{:X}{:X}{:X}{:X}",c1,c2,c3,c4)); }
                            };
                        }
                        _ => panic!("Invalid test data input"),
                    }
                } else {
                    output.push(c);
                }
        }
    }
}