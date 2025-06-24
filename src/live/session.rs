/// A live session, this is the representation sent to clients
/// when listing all sessions or after session creation
pub struct Session {
    /// An arbitrary name defined by the leader to help followers choose the correct sessions
    /// among the multiple live sessions at the same time on the same group_id
    /// We imagine it could be named like "Course name - Teacher fullname"
    pub name: String,
    /// The group id is a way to group related sessions together.
    /// This can be an arbitrary string chosen by leader clients when creating a session.
    /// Listing available sessions can only be done via this group_id to filter the list
    /// By default, PLX clients will send the Git HTTPS link
    pub group_id: String,
}

/// This manager is running in it's own tokio task and is responsible for
/// 1. forwarding messages to leaders or to all clients of the session
/// 2. manage joining
struct SessionManager {
    session: Session,
    /// The leader id of the leader that created this session, this is the only client that is
    /// authorized to close a session, the potential other leaders cannot do that.
    leader_client_id: String,
}

impl Session {
    async fn run() {}
}
