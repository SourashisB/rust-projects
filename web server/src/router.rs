use std::collections::HashMap;

pub struct Router {
    routes: HashMap<String, String>,
}

impl Router {
    pub fn new() -> Self {
        let mut routes = HashMap::new();
        routes.insert("/".to_string(), "static/index.html".to_string());
        routes.insert("/styles.css".to_string(), "static/styles.css".to_string());
        Router { routes }
    }

    pub fn get_route(&self, path: &str) -> Option<&String> {
        self.routes.get(path)
    }
}