pub fn base62_encode(mut num: u64) -> String {
    let mut chars = String::new();
    let base = 62;
    let alphabet = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";

    if num == 0 {
        return "a".to_string();
    }

    while num != 0 {
        let remainder = num % base;
        chars.insert(0, alphabet.chars().nth(remainder as usize).unwrap());
        num /= base;
    }

    chars
}