#[macro_use]
extern crate log;

#[macro_use]
pub mod hash;
#[macro_use]
pub mod io;
#[macro_use]
pub mod hashio;
pub mod lazyio;
pub mod logger;
pub mod iolog;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
