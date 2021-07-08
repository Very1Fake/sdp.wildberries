pub fn email(val: &String) -> bool {
    let mut local = 0;
    let mut domain = 0;
    let mut root = 0;
    let mut step = 0;

    let valid = val.chars().all(|c| {
        let common = c.is_alphabetic() || c.is_digit(10);

        match step {
            0 if c == '@' => step = 1,
            0 if common || c == '-' || c == '_' || c == '.' => local += 1,
            1 if c == '.' => step = 2,
            1 if common => domain += 1,
            2 if common => root += 1,
            _ => return false,
        }

        true
    });

    valid && (local > 2) && (domain > 2) && (root > 1)
}

pub fn phone(val: &String) -> bool {
    val.len() == 16
        && val.chars().enumerate().all(|(i, c)| match i {
            0 if c == '+' => true,
            1 if c == '7' => true,
            2 if c == '(' => true,
            6 if c == ')' => true,
            10 | 13 if c == '-' => true,
            3..=5 | 7..=9 | 11 | 12 | 14 | 15 if c.is_digit(10) => true,
            _ => false,
        })
}
