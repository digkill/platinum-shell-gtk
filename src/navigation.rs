#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppSurface {
    Home,
    Apps,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppRoute {
    Surface(AppSurface),
    App(&'static str),
}
