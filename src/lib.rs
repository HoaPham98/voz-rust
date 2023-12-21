mod core;
use core::models::*;
use serde_json::*;
pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let test: VozResponse<String> = VozResponse::Failed { message: "Hello".to_string() };
        println!("{}", to_string_pretty(&test).unwrap());
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
