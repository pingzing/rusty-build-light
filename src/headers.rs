// requests left in window
header! {(XRateLimitRemaining, "X-RateLimit-Remaining") => [u32]}

// when window will reset in epoch seconds
header! {(XRateLimitReset, "X-RateLimit-Reset") => [u64]} 