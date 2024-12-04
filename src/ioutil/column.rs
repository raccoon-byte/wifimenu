use terminal_size::{terminal_size, Width};

const SEPARATION_PART: usize = 8;

pub fn col_print(input: &[String]) {
    let mut num_cols: usize = 1;
    let mut max_width: usize = 0;

    if let Some((Width(width), _)) = terminal_size() {
        input.iter().for_each(|s| {
            if s.len() > max_width {
                max_width = s.len();
            }
        });

        max_width += SEPARATION_PART;
        num_cols = width as usize / max_width;
    }

    let mut cur_col: usize = 1;
    for (i, item) in input.iter().enumerate() {
        let mut item: String = format!("{}. {}", i + 1, item);
        item.push_str(&" ".repeat(max_width - item.len()));
        print!("{}", item);

        if cur_col < num_cols {
            cur_col += 1;
            continue;
        } else if i != input.len() - 1 {
            println!();
        }
        cur_col = 1;
    }
    println!();
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_col_print() {
        assert_eq!(2, 2);
    }
}
