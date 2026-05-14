#[derive(Debug)]
pub(super) struct PageInfo {
    pub(super) module_name: String,
    pub(super) module_path: String,
    pub(super) route_path: String,
    pub(super) route_segments: Vec<RouteSegment>,
    pub(super) path_params: Option<Vec<PathParamField>>,
    pub(super) search_params: Option<Vec<SearchParamField>>,
    pub(super) is_redirect_only: bool,
    pub(super) is_api: bool,
}

#[derive(Debug, Clone)]
pub(super) enum RouteSegment {
    Static(String),
    Dynamic(String), // [id] -> "id"
}

#[derive(Debug)]
pub(super) struct SearchParamField {
    pub(super) name: String,
    pub(super) is_optional: bool,
    pub(super) inner_type: String,
}

#[derive(Debug)]
pub(super) struct PathParamField {
    pub(super) name: String,
    pub(super) inner_type: String,
}

#[derive(Debug)]
pub(super) struct HookInfo {
    pub(super) name: String,
    pub(super) module_name: String,
    pub(super) module_path: String,
}

#[derive(Debug)]
pub(super) struct ActionInfo {
    pub(super) name: String,
}

#[derive(Debug)]
pub(super) struct QueueTaskInfo {
    pub(super) name: String,
}

#[derive(Debug)]
pub(super) struct AdminTaskInfo {
    pub(super) name: String,
}

pub(super) enum HandlerType {
    None,
    Props,
    Redirect,
}
