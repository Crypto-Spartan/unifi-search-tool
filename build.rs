use std::io;

fn main() -> io::Result<()> {
    #[cfg(windows)] {
        embed_resource::compile("icon.rc", embed_resource::NONE);
    }
    Ok(())
}