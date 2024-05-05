use rand::prelude::*;

/// Generate a random number between `min` and `max`.
pub fn rand_num<T>(min: T, max: T) -> T
    where T: rand::distributions::uniform::SampleUniform + PartialOrd {
    if min > max {
        thread_rng().gen_range(max..min)
    } else {
        thread_rng().gen_range(min..max)
    }
}

/// A string of emojis.
pub static CHARACTERS: &str = "🚨👮🚔🚓";

/// Generate a random string of emojis.
pub fn call_police_string() -> String {
    let times = rand_num(8, 96);
    let mut rng = thread_rng();
    let mut s = String::with_capacity(times); // pre-allocate memory

    for _ in 0..times {
        s.push(CHARACTERS.chars().choose(&mut rng).unwrap());
    }

    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rand_num() {
        let min = 0;
        let max = 10;
        let n = rand_num(min, max);
        assert!(n >= min && n < max);
    }

    #[test]
    fn test_call_police_string() {
        let s = call_police_string();
        assert!(!s.is_empty());

        let len = s.chars().count();
        assert!(len >= 8 && len < 96);
    }
}
