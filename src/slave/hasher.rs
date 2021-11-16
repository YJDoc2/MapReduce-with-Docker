pub fn get_hash(v: &str, lim: usize) -> usize {
    let mut sum: usize = 0;
    for (i, l) in v.chars().enumerate() {
        sum += ((i + 1) * (i + 1) * l as u8 as usize) % lim;
    }
    return (sum % lim) + 1;
}
