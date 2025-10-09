use super::state::State;

pub struct Metadata {
    name: String,
}

pub struct Task {
    uid: String,
    meta: Metadata,
    state: State,
}
