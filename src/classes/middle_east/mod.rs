use crate::classes::{Course, CourseWeek};

pub mod israel_palestine_1;
pub mod israel_palestine_2;

pub const WEEKS: &[CourseWeek] = &[
    CourseWeek {
        number: 1,
        title: "Ancient roots",
        summary: "Build a historical timeline of the region and examine why place, memory, and sacred sites matter.",
        topics: &[
            "Ancient communities and place names",
            "Jerusalem and its historical significance",
            "How history is remembered by different communities",
        ],
        slide_deck: "israel_palestine_1",
    },
    CourseWeek {
        number: 2,
        title: "The Ottoman Empire",
        summary: "Explore the late Ottoman period and the political transitions that shaped the modern era.",
        topics: &[
            "Ottoman rule and regional communities",
            "The end of imperial rule",
            "The transition to the British Mandate",
        ],
        slide_deck: "israel_palestine_1",
    },
    CourseWeek {
        number: 3,
        title: "Modern conflicts",
        summary: "Trace the 1947 partition proposal, statehood, regional wars, political movements, and diplomacy.",
        topics: &[
            "The 1947 UN partition proposal",
            "Statehood, war, and displacement",
            "Political movements and peace efforts",
        ],
        slide_deck: "israel_palestine_2",
    },
];

pub const COURSE: Course = Course {
    id: 1,
    title: "Middle East History",
    notes: include_str!("notes.md"),
    syllabus: include_str!("syllabus.md"),
    content: include_str!("content.md"),
    video_intro: include_str!("video_intro.md"),
    slide_decks: &["israel_palestine_1", "israel_palestine_2"],
    weeks: WEEKS,
};

pub const ALL_COURSES: &[Course] = &[COURSE];
