#[derive(Display, Debug, Clone, EnumString)]
#[strum(serialize_all = "kebab_case")]
pub enum Key {
    /* Client Backend */
    ApiServer,

    /* User Interface */
    DarkMode,
    WindowWidth,
    WindowHeight,
    ViewSorting,
    ViewOrder,

    /* Audio */
    RecorderSaveCount,
}
