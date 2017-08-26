use std::io::Write;
use chrono::{DateTime, TimeZone};
use chrono::offset::Utc;

pub trait ContentManager<H: ContentHandle> {
    fn find_content(&self, url: &String, accepts_gzip: bool) -> Option<H>;
}

pub trait ContentHandle {
    fn is_mod_since<TZ: TimeZone>(&self, other: &DateTime<TZ>) -> bool;
    fn mod_time(&self) -> &DateTime<Utc>;
    fn is_gzipped(&self) -> bool;
    fn write_to(&mut self, writer: &mut Write);
}