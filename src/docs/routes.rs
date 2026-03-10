use once_cell::sync::Lazy;
use std::sync::Mutex;

#[derive(Clone)]
pub struct RouteDoc {
    pub group: &'static str,
    pub path: &'static str,
    pub description: &'static str,
}

pub static ROUTES: Lazy<Mutex<Vec<RouteDoc>>> = Lazy::new(|| Mutex::new(Vec::new()));

pub fn register(group: &'static str, path: &'static str, description: &'static str) {
    ROUTES.lock().unwrap().push(RouteDoc {
        group,
        path,
        description,
    });
}
