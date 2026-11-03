pub fn image<'d>(downloads: &'d super::Downloader, location: &str) -> Option<&'d [u8]> {
    downloads.images.get(location).map(Vec::as_slice)
}
