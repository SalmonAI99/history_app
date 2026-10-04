#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MediaType {
    Image,
    Video,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MediaItem {
    pub media_type: MediaType,
    pub src: &'static str,
    pub poster: Option<&'static str>,
    pub caption: Option<&'static str>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Slide {
    pub num: usize,
    pub title: &'static str,
    pub paragraphs: &'static [&'static str],
    pub media: &'static [MediaItem],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SlideDeck {
    pub id: &'static str,
    pub title: &'static str,
    pub slide_count: usize,
    pub slides: &'static [Slide],
}
