#[derive(Debug, Clone)]
pub struct ComposeError {
    err: String,
}

impl ComposeError {
    pub fn new(err: &str) -> ComposeError {
        ComposeError {
            err: err.to_string(),
        }
    }
}

impl std::fmt::Display for ComposeError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.err)
    }
}

impl std::error::Error for ComposeError {
    fn description(&self) -> &str {
        &self.err
    }

    fn cause(&self) -> Option<&dyn std::error::Error> {
        // 泛型错误。没有记录其内部原因。
        None
    }
}

pub fn ba_error(error: &str) -> Box<dyn std::error::Error> {
    Box::new(ComposeError::new(error))
}

pub type BDError = Box<dyn std::error::Error>;
pub type BDEResult<T> = Result<T, BDError>;

pub fn print_n(print_str: &String, n: i32) {
    for _ in 0..n {
        print!("{}", print_str);
    }
    println!("");
}

pub fn is_yes(promt: &str) -> bool {
    println!("{}(y/n):", promt);
    let mut input_string = String::new();
    std::io::stdin().read_line(&mut input_string).unwrap();
    input_string.trim() == "y" || input_string.trim() == "yes"
}
