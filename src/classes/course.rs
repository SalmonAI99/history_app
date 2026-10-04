#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Course {
    pub id: i64,
    pub title: &'static str,
    pub notes: &'static str,
    pub syllabus: &'static str,
    pub content: &'static str,
    pub video_intro: &'static str,
    pub slide_decks: &'static [&'static str],
    pub weeks: &'static [CourseWeek],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CourseWeek {
    pub number: usize,
    pub title: &'static str,
    pub summary: &'static str,
    pub topics: &'static [&'static str],
    pub slide_deck: &'static str,
}
