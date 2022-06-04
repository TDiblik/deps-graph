use std::env;

lazy_static! {
    pub static ref USER_AGENT_IDENTIFIER: String = env::var("USER_AGENT_IDENTIFIER").expect(
        "USER_AGENT_IDENTIFIER header is strictlly required! This API is using third party APIs, we MUST let authors of these APIs know that requests sent using this are not done by users."
    );
}
