use rand::prelude::*;

/// Generate a random number between `min` and `max`.
#[tracing::instrument]
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
#[tracing::instrument]
pub fn call_police_string() -> String {
    let times = rand_num(8, 96);
    let mut rng = thread_rng();
    let mut s = String::with_capacity(times); // pre-allocate memory

    for _ in 0..times {
        s.push(CHARACTERS.chars().choose(&mut rng).unwrap());
    }

    s
}

/// Get the value of an environment variable or a default value.
#[tracing::instrument]
pub fn env_or_default(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
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
