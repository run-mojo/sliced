
/// MO.XADD
pub struct AddCommand;

/// MO.XDEL
pub struct DelCommand;

/// MO.XTRIM
pub struct TrimCommand;

/// MO.XCLAIM
pub struct ClaimCommand;

/// MO.XACK
pub struct AckCommand;

/// MO.GROUP
pub struct GroupCommand;

/// MO.STREAM
pub struct StreamCommand;

/// MO.STATS
pub struct StatsCommand;

/// MO.IO
///
///
pub struct IOCommand;

/// MO.X
///
/// Internal commands used for AOF and Replication.
pub struct InternalCommand;

/// MO.XCOPY
///
/// Copies entries to a Redis Stream
pub struct CopyToCommand;

/// MO.XCOPYTRIM
///
/// Copies entries from a Redis Stream and trims them from the Redis
/// Stream after persisting successfully in slice/d. This provides a
/// way to persist Redis Streams.
pub struct CopyTrimCommand;