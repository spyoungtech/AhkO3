use ahko3::prelude::*;


#[ahkfunction]
pub fn concatenate(a: String, b: String) -> String {
    format!("{}{}", a, b)
}

#[ahkfunction]
pub fn add(left: i64, right: i64) -> i64 {
    left + right
}

#[cfg(test)]
mod tests;