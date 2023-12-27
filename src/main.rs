use anyhow::Result;
extern crate openssl;
#[allow(unused_imports)]
#[macro_use]
extern crate diesel;

#[tokio::main]
async fn main() -> Result<()> {
    nautilus::server::start().await?;

    Ok(())
}
