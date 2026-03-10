#[macro_export]
macro_rules! droute {
    ($router:expr, $group:expr, $path:expr, $handler:expr, $desc:expr) => {{
        $crate::docs::routes::register($group, $path, $desc);
        $router.route($path, axum::routing::get($handler))
    }};
}
