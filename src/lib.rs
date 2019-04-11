#[cfg(windows)]
pub mod colorable_windows;
#[cfg(windows)]
pub use colorable_windows as colorable;

#[cfg(unix)]
pub mod colorable_unix;
#[cfg(unix)]
pub use colorable_unix as colorable;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
