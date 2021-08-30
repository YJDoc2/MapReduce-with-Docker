pub fn get_hash(v: &str, lim: u8) -> u8 {
    let mut sum: u8 = 0;
    for (i, l) in v.chars().enumerate() {
        sum += (((i + 1) * (i + 1) * l as u8 as usize) % lim as usize) as u8;
    }
    return (sum % lim) + 1;
}
