//! The `ResultPropagation` portable capability.

/// Marker for explicit early propagation without target exceptions.
#[derive(Clone, Copy, Debug)]
pub enum ResultPropagation {}
