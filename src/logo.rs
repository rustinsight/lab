pub static LEARN: AppInfo = AppInfo {
    line: "  Ri!  ",
    name: "  Lab  ",
    bg: Color(255, 61, 0),
    link: "https://api.github.com/repos/rustinsight/lab/releases",
};

// https://api.github.com/repos/rustinsight/rustinsight/releases

/*
pub static STACK: AppInfo = AppInfo {
    line: "  Ri!  ",
    name: " Stack ",
    bg: Color(215, 164, 35),
    link: "https://api.github.com/repos/rustinsight/stack/releases",
};
*/

#[derive(Debug, Clone)]
pub struct AppInfo {
    pub line: &'static str,
    pub name: &'static str,
    pub bg: Color,
    pub link: &'static str,
}

#[derive(Debug, Clone)]
pub struct Color(pub u8, pub u8, pub u8);
