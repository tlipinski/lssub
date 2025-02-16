pub mod guess;
pub mod login;
pub mod values;
pub mod user_info;

pub fn addd(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = addd(2, 2);
        assert_eq!(result, 4);
    }
}
