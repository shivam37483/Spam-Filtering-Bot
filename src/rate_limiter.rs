// // rate_limiter.rs
// use std::collections::HashMap;
// use std::time::{Duration, Instant};
// use teloxide::types::UserId;

// pub struct RateLimiter {
//     user_messages: HashMap<UserId, Vec<Instant>>,
//     max_messages: usize,
//     time_window: Duration,
// }

// impl RateLimiter {
//     pub fn new(max_messages: usize, time_window: Duration) -> Self {
//         Self {
//             user_messages: HashMap::new(),
//             max_messages,
//             time_window,
//         }
//     }

//     pub fn check(&mut self, user_id: UserId) -> bool {
//         let now = Instant::now();
//         let messages = self.user_messages.entry(user_id).or_insert_with(Vec::new);
//         messages.retain(|&t| now.duration_since(t) < self.time_window);
//         messages.push(now);
//         messages.len() > self.max_messages
//     }
// }