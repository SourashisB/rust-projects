use std::env;

/// Returns true if the string `s` matches the pattern `p`.
fn is_match(s: &str, p: &str) -> bool {
    let s_bytes = s.as_bytes();
    let p_bytes = p.as_bytes();
    let m = s.len();
    let n = p.len();

    // dp[i][j] = does s[0..i] match p[0..j]
    let mut dp = vec![vec![false; n + 1]; m + 1];

    dp[0][0] = true;

    // Deals with patterns like a*, a*b*, a*b*c* matching empty string
    for j in 1..=n {
        if p_bytes[j - 1] == b'*' && j >= 2 {
            dp[0][j] = dp[0][j - 2];
        }
    }

    for i in 1..=m {
        for j in 1..=n {
            if p_bytes[j - 1] == b'.' || p_bytes[j - 1] == s_bytes[i - 1] {
                dp[i][j] = dp[i - 1][j - 1];
            } else if p_bytes[j - 1] == b'*' && j >= 2 {
                // zero occurrence of the char before '*'
                dp[i][j] = dp[i][j - 2];
                // one or more occurrence if matches
                if p_bytes[j - 2] == b'.' || p_bytes[j - 2] == s_bytes[i - 1] {
                    dp[i][j] = dp[i][j] || dp[i - 1][j];
                }
            }
        }
    }

    dp[m][n]
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        println!("Usage: {} <string> <pattern>", args[0]);
        println!("Example: {} \"aab\" \"c*a*b\"", args[0]);
        return;
    }

    let s = &args[1];
    let p = &args[2];

    let result = is_match(s, p);
    println!("{}", result);
}