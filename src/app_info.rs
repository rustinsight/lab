pub static LEARN: AppInfo = AppInfo {
    name: "ri-lab",
    line_1: "  Ri!  ",
    line_2: "  Lab  ",
    bg: Color(255, 61, 0),
    link: "https://api.github.com/repos/rustinsight/lab/releases",
};

// https://api.github.com/repos/rustinsight/rustinsight/releases

/*
pub static STACK: AppInfo = AppInfo {
    line_1: "  Ri!  ",
    line_2: " Stack ",
    bg: Color(215, 164, 35),
    link: "https://api.github.com/repos/rustinsight/stack/releases",
};
*/

#[derive(Debug, Clone)]
pub struct AppInfo {
    pub name: &'static str,
    pub line_1: &'static str,
    pub line_2: &'static str,
    pub bg: Color,
    pub link: &'static str,
}

#[derive(Debug, Clone)]
pub struct Color(pub u8, pub u8, pub u8);
