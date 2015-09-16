// Grab the value inside an Option<T>.
// If the Option is None, return.
macro_rules! some_or_return {
    ($e:expr) => {
        match $e {
            Some(it) => it,
            None => return,
        }
    }
}

// Grab the user out of an Option<Prefix>.
// If it's None or not a User, it will return.
macro_rules! user_or_return {
    ($e:expr) => {
        match $e {
            Some(ref prefix) => {
                match *prefix {
                    Prefix::User(ref user) => user,
                    _ => return,
                }
            }
            _ => return,
        }
    }
}
