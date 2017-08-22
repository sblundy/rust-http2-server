use std::io::Write;

pub trait ContentManager<H: ContentHandle> {
    fn find_content(&self, url: &String, accepts_gzip: bool) -> Option<H>;
}

pub trait ContentHandle {
    fn is_gzipped(&self) -> bool;
    fn write_to(&mut self, writer: &mut Write);
}