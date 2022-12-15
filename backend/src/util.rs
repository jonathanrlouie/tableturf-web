use tracing::warn;
use thiserror::Error;

#[derive(Error, Debug)]
#[error("Retry failed")]
pub struct RetryFailed;

pub fn retry<T, E, F>(mut times: u32, f: F) -> Result<T, RetryFailed>
where F: Fn() -> Result<T, E>, E: std::error::Error + std::fmt::Display {
    let mut result = f();
    while let Err(err) = result {
        if times == 0 {
            warn!("No more retry attempts. Error: {}", err);
            return Err(RetryFailed)
        }
        warn!("Retry triggered. Error: {}", err);
        result = f();
        times -= 1;
    };
    Ok(result.unwrap())
}
